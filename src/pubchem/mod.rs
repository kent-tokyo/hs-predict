//! PubChem REST API client.
//!
//! Requires the **`pubchem`** Cargo feature.
//!
//! # Purpose
//!
//! Enriches a [`SubstanceIdentifier`] with structural data fetched from the
//! [PubChem PUG REST API](https://pubchem.ncbi.nlm.nih.gov/docs/pug-rest):
//! - CAS number → SMILES, InChI, InChIKey, IUPAC name, CID
//! - IUPAC name → SMILES, CID, …
//! - SMILES / InChIKey / InChI → CID + remaining fields
//!
//! # Usage
//!
//! ```rust,no_run
//! # #[cfg(feature = "pubchem")]
//! # async fn example() -> hs_predict::Result<()> {
//! use hs_predict::pipeline::HsPipeline;
//! use hs_predict::pubchem::PubChemClient;
//! use hs_predict::types::{ProductDescription, SubstanceIdentifier};
//!
//! let pipeline = HsPipeline::new()
//!     .with_pubchem(PubChemClient::new());
//!
//! let mut product = ProductDescription {
//!     identifier: SubstanceIdentifier::from_cas("1310-73-2"),
//!     physical_form: None,
//!     purity_pct: None,
//!     purity_type: None,
//!     mixture_components: None,
//!     intended_use: None,
//!     additional_context: None,
//! };
//!
//! // Enrich: CAS 1310-73-2 → SMILES "[Na+].[OH-]", IUPAC "sodium hydroxide", …
//! pipeline.enrich(&mut product).await?;
//!
//! // Classify as normal (SMILES now available → better matching)
//! let prediction = pipeline.classify(&product)?;
//! println!("{}", prediction.display());
//! # Ok(())
//! # }
//! ```
//!
//! # Rate limiting
//!
//! PubChem allows up to **5 requests / second** without an API key.
//! [`PubChemClient`] enforces this automatically via an internal token-bucket
//! rate limiter ([`governor`]).
//!
//! # Caching
//!
//! Responses are cached by PubChem CID using [`moka`] with a 24-hour TTL and
//! a 1 000-entry capacity. The same compound looked up by different identifiers
//! (CAS vs. InChIKey) is cached once after the first fetch.

mod error;

pub use error::PubChemError;

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use moka::future::Cache;
use serde::Deserialize;
use urlencoding::encode;

use crate::error::{HsPredictError, Result};
use crate::types::SubstanceIdentifier;

// ─── PubChem API constants ────────────────────────────────────────────────────

const BASE_URL: &str = "https://pubchem.ncbi.nlm.nih.gov/rest/pug";

/// Properties fetched in each request (comma-separated PubChem field names).
const PROPERTIES: &str =
    "IUPACName,CanonicalSMILES,InChIKey,InChI,MolecularFormula,MolecularWeight";

// ─── Result type ─────────────────────────────────────────────────────────────

/// Compound data returned from a successful PubChem lookup.
#[derive(Debug, Clone)]
pub struct PubChemCompound {
    /// PubChem Compound ID.
    pub cid: u64,
    /// Preferred IUPAC name as assigned by PubChem.
    pub iupac_name: Option<String>,
    /// Canonical SMILES string.
    pub canonical_smiles: Option<String>,
    /// Standard InChI string.
    pub inchi: Option<String>,
    /// 27-character InChIKey.
    pub inchi_key: Option<String>,
    /// Hill-notation molecular formula.
    pub molecular_formula: Option<String>,
    /// Molecular weight in g/mol.
    pub molecular_weight: Option<f64>,
}

impl PubChemCompound {
    /// Copy fields from this compound into `id`, filling only the **missing** fields.
    ///
    /// The CID is always set. Other fields are only written if the identifier
    /// field is currently `None`.
    pub fn apply_to(&self, id: &mut SubstanceIdentifier) {
        id.cid = Some(self.cid);
        if id.smiles.is_none() {
            id.smiles = self.canonical_smiles.clone();
        }
        if id.iupac_name.is_none() {
            id.iupac_name = self.iupac_name.clone();
        }
        if id.inchi.is_none() {
            id.inchi = self.inchi.clone();
        }
        if id.inchi_key.is_none() {
            id.inchi_key = self.inchi_key.clone();
        }
    }
}

// ─── Client ──────────────────────────────────────────────────────────────────

/// PubChem REST API client with built-in rate limiting and in-memory caching.
///
/// Cheap to clone — all internal state is reference-counted.
#[derive(Clone)]
pub struct PubChemClient {
    http: reqwest::Client,
    /// CID → compound (24 h TTL, capacity 1 000).
    cache: Cache<u64, Arc<PubChemCompound>>,
    limiter: Arc<DefaultDirectRateLimiter>,
    /// Configurable base URL (override for testing).
    base_url: String,
}

impl std::fmt::Debug for PubChemClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PubChemClient")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

impl Default for PubChemClient {
    fn default() -> Self {
        Self::new()
    }
}

impl PubChemClient {
    /// Create a client with PubChem's default rate limit (5 req/s).
    ///
    /// # Panics
    /// Panics if the TLS backend cannot be initialised (extremely rare;
    /// indicates a broken system environment).
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Start building a customised client.
    pub fn builder() -> PubChemClientBuilder {
        PubChemClientBuilder::default()
    }

    // ── Core lookup ──────────────────────────────────────────────────

    /// Look up a compound by the best available identifier.
    ///
    /// Priority: CAS number → InChIKey → InChI → SMILES → IUPAC name.
    ///
    /// Results are cached by CID, so repeated calls for the same compound
    /// are free after the first network request.
    ///
    /// # Errors
    /// - [`PubChemError::NotFound`] — no compound matched.
    /// - [`PubChemError::NoUsableIdentifier`] — the identifier has no usable field.
    /// - [`PubChemError::Http`] — network or server error.
    /// - [`PubChemError::RateLimitExceeded`] — PubChem returned HTTP 429.
    pub async fn lookup(&self, id: &SubstanceIdentifier) -> Result<PubChemCompound> {
        // Fast-path: if CID is already known, check cache first
        if let Some(cid) = id.cid {
            if let Some(cached) = self.cache.get(&cid).await {
                return Ok((*cached).clone());
            }
        }

        let (namespace, input) = Self::pick_namespace(id)
            .ok_or(PubChemError::NoUsableIdentifier)?;

        self.fetch(namespace, &input).await
    }

    /// Enrich `id` in place with PubChem data, filling any missing fields.
    ///
    /// On [`PubChemError::NotFound`] or [`PubChemError::NoUsableIdentifier`]
    /// this is a silent no-op (enrichment is best-effort).
    /// Other errors (network, parse) are propagated.
    pub async fn enrich(&self, id: &mut SubstanceIdentifier) -> Result<()> {
        match self.lookup(id).await {
            Ok(compound) => {
                compound.apply_to(id);
                Ok(())
            }
            Err(HsPredictError::PubChem(PubChemError::NotFound { .. }))
            | Err(HsPredictError::PubChem(PubChemError::NoUsableIdentifier)) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // ── Private helpers ───────────────────────────────────────────────

    /// Pick the best (namespace, input) pair for a PubChem URL.
    fn pick_namespace(id: &SubstanceIdentifier) -> Option<(&'static str, String)> {
        if let Some(ref cas) = id.cas {
            return Some(("name", cas.clone()));
        }
        if let Some(ref key) = id.inchi_key {
            return Some(("inchikey", key.clone()));
        }
        if let Some(ref inchi) = id.inchi {
            return Some(("inchi", inchi.clone()));
        }
        if let Some(ref smiles) = id.smiles {
            return Some(("smiles", smiles.clone()));
        }
        if let Some(ref name) = id.iupac_name {
            return Some(("name", name.clone()));
        }
        None
    }

    /// Fetch compound properties from PubChem and cache the result.
    async fn fetch(&self, namespace: &str, input: &str) -> Result<PubChemCompound> {
        // Honour the rate limit
        self.limiter.until_ready().await;

        let url = format!(
            "{base}/compound/{ns}/{enc}/property/{props}/JSON",
            base = self.base_url,
            ns   = namespace,
            enc  = encode(input),
            props = PROPERTIES,
        );

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| PubChemError::Http(e.to_string()))?;

        match resp.status().as_u16() {
            200 => {}
            404 => return Err(PubChemError::NotFound { input: input.to_string() }.into()),
            429 => return Err(PubChemError::RateLimitExceeded.into()),
            code => {
                return Err(PubChemError::Http(format!("HTTP {code}")).into());
            }
        }

        let body: PugPropertyResponse = resp
            .json()
            .await
            .map_err(|e| PubChemError::Parse(e.to_string()))?;

        let props = body
            .property_table
            .properties
            .into_iter()
            .next()
            .ok_or_else(|| PubChemError::NotFound { input: input.to_string() })?;

        let compound = Arc::new(PubChemCompound {
            cid: props.cid,
            iupac_name: props.iupac_name,
            canonical_smiles: props.canonical_smiles,
            inchi: props.in_chi,
            inchi_key: props.in_chi_key,
            molecular_formula: props.molecular_formula,
            molecular_weight: props.molecular_weight.as_deref().and_then(|s| s.parse().ok()),
        });

        // Cache by CID
        self.cache.insert(compound.cid, Arc::clone(&compound)).await;

        Ok((*compound).clone())
    }
}

// ─── Builder ─────────────────────────────────────────────────────────────────

/// Builder for [`PubChemClient`].
pub struct PubChemClientBuilder {
    requests_per_second: u32,
    cache_capacity: u64,
    cache_ttl: Duration,
    base_url: String,
    user_agent: String,
}

impl Default for PubChemClientBuilder {
    fn default() -> Self {
        Self {
            requests_per_second: 5,
            cache_capacity: 1_000,
            cache_ttl: Duration::from_secs(24 * 3600),
            base_url: BASE_URL.to_string(),
            user_agent: format!(
                "hs-predict/{} ({})",
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_REPOSITORY")
            ),
        }
    }
}

impl PubChemClientBuilder {
    /// Maximum HTTP requests per second (default: 5 — PubChem's published limit).
    pub fn requests_per_second(mut self, n: u32) -> Self {
        self.requests_per_second = n.max(1);
        self
    }

    /// In-memory cache capacity in number of entries (default: 1 000).
    pub fn cache_capacity(mut self, n: u64) -> Self {
        self.cache_capacity = n;
        self
    }

    /// Cache TTL (default: 24 hours).
    pub fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Override the PubChem base URL (useful for testing against a local mock server).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Build the [`PubChemClient`].
    ///
    /// # Panics
    /// Panics if the TLS backend cannot be initialised (extremely rare;
    /// indicates a broken system environment).  For an infallible path in
    /// constrained environments use [`try_build`](Self::try_build).
    pub fn build(self) -> PubChemClient {
        self.try_build()
            .expect("failed to build PubChemClient — TLS backend unavailable")
    }

    /// Build the [`PubChemClient`], returning an error instead of panicking if
    /// the underlying HTTP client cannot be initialised (e.g. TLS failure).
    ///
    /// Prefer this over [`build`](Self::build) in long-running servers and WASM
    /// environments where a panic is unacceptable.
    pub fn try_build(self) -> Result<PubChemClient> {
        // `requests_per_second` is always ≥ 1 because the setter clamps with
        // `.max(1)` and the default is 5, so `NonZeroU32::new` never returns
        // `None` here.
        let rps = NonZeroU32::new(self.requests_per_second.max(1))
            .expect("max(1) guarantees non-zero");
        let quota = Quota::per_second(rps);

        let http = reqwest::Client::builder()
            .user_agent(self.user_agent)
            .build()
            .map_err(|e| HsPredictError::Http(format!("failed to build HTTP client: {e}")))?;

        Ok(PubChemClient {
            http,
            cache: Cache::builder()
                .max_capacity(self.cache_capacity)
                .time_to_live(self.cache_ttl)
                .build(),
            limiter: Arc::new(RateLimiter::direct(quota)),
            base_url: self.base_url,
        })
    }
}

// ─── PubChem JSON response types (private) ───────────────────────────────────

#[derive(Deserialize)]
struct PugPropertyResponse {
    #[serde(rename = "PropertyTable")]
    property_table: PropertyTable,
}

#[derive(Deserialize)]
struct PropertyTable {
    #[serde(rename = "Properties")]
    properties: Vec<CompoundProperty>,
}

#[derive(Deserialize)]
struct CompoundProperty {
    #[serde(rename = "CID")]
    cid: u64,
    #[serde(rename = "IUPACName")]
    iupac_name: Option<String>,
    #[serde(rename = "CanonicalSMILES")]
    canonical_smiles: Option<String>,
    #[serde(rename = "InChI")]
    in_chi: Option<String>,
    #[serde(rename = "InChIKey")]
    in_chi_key: Option<String>,
    #[serde(rename = "MolecularFormula")]
    molecular_formula: Option<String>,
    /// PubChem returns molecular weight as a string (e.g. `"39.997"`).
    #[serde(rename = "MolecularWeight")]
    molecular_weight: Option<String>,
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_builds_with_defaults() {
        let client = PubChemClient::new();
        assert_eq!(client.base_url, BASE_URL);
    }

    #[test]
    fn builder_overrides_base_url() {
        let client = PubChemClient::builder()
            .base_url("http://localhost:8080")
            .build();
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn pick_namespace_cas_first() {
        let id = SubstanceIdentifier {
            cas: Some("1310-73-2".to_string()),
            smiles: Some("[Na+].[OH-]".to_string()),
            ..Default::default()
        };
        let (ns, inp) = PubChemClient::pick_namespace(&id).unwrap();
        assert_eq!(ns, "name");
        assert_eq!(inp, "1310-73-2");
    }

    #[test]
    fn pick_namespace_inchikey_when_no_cas() {
        let id = SubstanceIdentifier {
            inchi_key: Some("HEMHJVSKTPXQMS-UHFFFAOYSA-M".to_string()),
            ..Default::default()
        };
        let (ns, inp) = PubChemClient::pick_namespace(&id).unwrap();
        assert_eq!(ns, "inchikey");
        assert_eq!(inp, "HEMHJVSKTPXQMS-UHFFFAOYSA-M");
    }

    #[test]
    fn pick_namespace_returns_none_for_empty_id() {
        let id = SubstanceIdentifier::default();
        assert!(PubChemClient::pick_namespace(&id).is_none());
    }

    #[test]
    fn apply_to_fills_missing_fields_only() {
        let compound = PubChemCompound {
            cid: 14798,
            iupac_name: Some("sodium hydroxide".to_string()),
            canonical_smiles: Some("[Na+].[OH-]".to_string()),
            inchi: Some("InChI=1S/Na.H2O/h;1H/q+1;/p-1".to_string()),
            inchi_key: Some("HEMHJVSKTPXQMS-UHFFFAOYSA-M".to_string()),
            molecular_formula: Some("HNaO".to_string()),
            molecular_weight: Some(39.997),
        };

        let mut id = SubstanceIdentifier {
            cas: Some("1310-73-2".to_string()),
            smiles: Some("existing".to_string()), // should NOT be overwritten
            ..Default::default()
        };

        compound.apply_to(&mut id);

        assert_eq!(id.cid, Some(14798));
        assert_eq!(id.smiles.as_deref(), Some("existing")); // preserved
        assert_eq!(id.iupac_name.as_deref(), Some("sodium hydroxide")); // filled
        assert_eq!(id.inchi_key.as_deref(), Some("HEMHJVSKTPXQMS-UHFFFAOYSA-M")); // filled
    }

    /// Integration test: real PubChem network call.
    /// Run with `cargo test -- --ignored` (requires internet access).
    #[tokio::test]
    #[ignore = "requires internet access"]
    async fn integration_lookup_naoh_by_cas() {
        let client = PubChemClient::new();
        let id = SubstanceIdentifier::from_cas("1310-73-2");
        let compound = client.lookup(&id).await.unwrap();

        assert_eq!(compound.cid, 14798);
        assert_eq!(
            compound.canonical_smiles.as_deref(),
            Some("[Na+].[OH-]")
        );
        assert_eq!(
            compound.iupac_name.as_deref(),
            Some("sodium hydroxide")
        );
    }

    #[tokio::test]
    #[ignore = "requires internet access"]
    async fn integration_enrich_fills_smiles() {
        let client = PubChemClient::new();
        let mut id = SubstanceIdentifier::from_cas("67-64-1"); // acetone
        client.enrich(&mut id).await.unwrap();

        assert!(id.smiles.is_some());
        assert!(id.cid.is_some());
        assert!(id.iupac_name.is_some());
    }
}
