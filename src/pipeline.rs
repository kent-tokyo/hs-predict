//! Main classification pipeline.
//!
//! Runs classification in priority order:
//! 1. User-provided CAS → HS mappings (confidence = 1.0)
//! 2. Embedded static rule table (CAS + shape + purity)
//! 3. SMILES-based rule engine (v0.3)
//! 4. LLM fallback via [`LlmClassifier`] trait hook (v0.4, `llm` feature)

use std::collections::HashMap;
#[cfg(feature = "llm")]
use std::sync::Arc;

use crate::error::{HsPredictError, Result};
use crate::rules::jp_table::{find_jp_rule, JP_TARIFF_YEAR};
use crate::rules::matcher::find_best_rule;
use crate::types::{
    GrayZone, HsPrediction, OrganicInorganic, PhysicalForm, ProductDescription,
    PredictionSource, RecommendedAction,
};

/// Configuration for the classification pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Confidence threshold above which a result is returned directly
    /// without asking for LLM confirmation.
    pub confidence_threshold_direct: f32,

    /// Confidence threshold below which LLM is required.
    /// Between `confidence_threshold_llm_required` and `confidence_threshold_direct`
    /// the result is returned with `RecommendedAction::VerifyWithLlm`.
    pub confidence_threshold_llm_required: f32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            confidence_threshold_direct: 0.85,
            confidence_threshold_llm_required: 0.50,
        }
    }
}

/// Main HS code classification pipeline.
///
/// # Example — direct (sync)
/// ```rust,no_run
/// use hs_predict::pipeline::HsPipeline;
/// use hs_predict::types::{ProductDescription, SubstanceIdentifier, PhysicalForm};
///
/// let pipeline = HsPipeline::new();
///
/// let product = ProductDescription {
///     identifier: SubstanceIdentifier::from_cas("1310-73-2"),
///     physical_form: Some(PhysicalForm::Solid),
///     purity_pct: None,
///     purity_type: None,
///     mixture_components: None,
///     intended_use: None,
///     additional_context: None,
/// };
///
/// let prediction = pipeline.classify(&product).unwrap();
/// assert_eq!(&prediction.hs_code, "281511");
/// ```
///
/// # Example — with PubChem enrichment (async, `pubchem` feature)
/// ```rust,no_run
/// # #[cfg(feature = "pubchem")]
/// # async fn example() -> hs_predict::Result<()> {
/// use hs_predict::pipeline::HsPipeline;
/// use hs_predict::pubchem::PubChemClient;
/// use hs_predict::types::{ProductDescription, SubstanceIdentifier, PhysicalForm};
///
/// let pipeline = HsPipeline::new().with_pubchem(PubChemClient::new());
///
/// let mut product = ProductDescription {
///     identifier: SubstanceIdentifier::from_cas("1310-73-2"),
///     physical_form: Some(PhysicalForm::Solid),
///     purity_pct: None,
///     purity_type: None,
///     mixture_components: None,
///     intended_use: None,
///     additional_context: None,
/// };
///
/// pipeline.enrich(&mut product).await?;   // fills SMILES, InChI, IUPAC name …
/// let prediction = pipeline.classify(&product)?;
/// println!("{}", prediction.display());   // "28.15.11"
/// # Ok(())
/// # }
/// ```
///
/// # Example — with LLM fallback (async, `llm` feature)
/// ```rust,no_run
/// # #[cfg(feature = "llm")]
/// # async fn example() -> hs_predict::Result<()> {
/// use hs_predict::pipeline::HsPipeline;
/// use hs_predict::llm::{LlmClassifier, LlmPrompt, LlmResponse};
/// use futures::future::BoxFuture;
///
/// struct MyClient;
/// impl LlmClassifier for MyClient {
///     fn classify<'a>(&'a self, prompt: &'a LlmPrompt) -> BoxFuture<'a, hs_predict::Result<LlmResponse>> {
///         Box::pin(async move { todo!() })
///     }
/// }
///
/// let pipeline = HsPipeline::new().with_llm(MyClient);
/// use hs_predict::types::{ProductDescription, SubstanceIdentifier};
/// let product = ProductDescription {
///     identifier: SubstanceIdentifier::from_cas("12-34-5"),
///     physical_form: None, purity_pct: None, purity_type: None,
///     mixture_components: None, intended_use: None, additional_context: None,
/// };
/// let prediction = pipeline.classify_with_llm(&product).await?;
/// println!("{}", prediction.display());
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct HsPipeline {
    /// User-supplied CAS → HS code overrides. Highest priority.
    user_mappings: HashMap<String, String>,

    config: PipelineConfig,

    /// PubChem client for identifier enrichment (v0.2, `pubchem` feature).
    #[cfg(feature = "pubchem")]
    pubchem: Option<std::sync::Arc<crate::pubchem::PubChemClient>>,

    /// LLM classifier hook (v0.4, `llm` feature).
    #[cfg(feature = "llm")]
    llm: Option<Arc<dyn crate::llm::LlmClassifier>>,
}

impl std::fmt::Debug for HsPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("HsPipeline");
        s.field("user_mappings", &self.user_mappings);
        s.field("config", &self.config);
        #[cfg(feature = "pubchem")]
        s.field("pubchem", &self.pubchem.as_ref().map(|_| "<PubChemClient>"));
        #[cfg(feature = "llm")]
        s.field("llm", &self.llm.as_ref().map(|_| "<dyn LlmClassifier>"));
        s.finish()
    }
}

impl HsPipeline {
    /// Create a pipeline with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a user-provided CAS → HS code mapping.
    ///
    /// These mappings override the embedded rule table with `confidence = 1.0`.
    ///
    /// The `hs_code` must be exactly 6 ASCII digits (e.g. `"281511"`).
    /// If the code does not satisfy this constraint the mapping is silently
    /// ignored and the pipeline is returned unchanged.
    pub fn with_mapping(mut self, cas: impl Into<String>, hs_code: impl Into<String>) -> Self {
        let hs_code = hs_code.into();
        let valid = hs_code.len() == 6 && hs_code.chars().all(|c| c.is_ascii_digit());
        if valid {
            self.user_mappings.insert(cas.into(), hs_code);
        }
        self
    }

    /// Override the default pipeline configuration.
    pub fn with_config(mut self, config: PipelineConfig) -> Self {
        self.config = config;
        self
    }

    /// Attach an [`LlmClassifier`](crate::llm::LlmClassifier) implementation to
    /// enable the LLM fallback (Priority 4).
    ///
    /// The LLM is called by [`classify_with_llm`](Self::classify_with_llm) when
    /// the rule-based pipeline returns a result with
    /// `recommended_action != Accept`, or returns
    /// [`LowConfidenceNoLlm`](crate::HsPredictError::LowConfidenceNoLlm).
    ///
    /// Requires the **`llm`** Cargo feature.
    #[cfg(feature = "llm")]
    pub fn with_llm(mut self, client: impl crate::llm::LlmClassifier + 'static) -> Self {
        self.llm = Some(Arc::new(client));
        self
    }

    /// Attach a [`PubChemClient`](crate::pubchem::PubChemClient) to enable
    /// automatic identifier enrichment before classification.
    ///
    /// Requires the **`pubchem`** Cargo feature.
    #[cfg(feature = "pubchem")]
    pub fn with_pubchem(mut self, client: crate::pubchem::PubChemClient) -> Self {
        self.pubchem = Some(std::sync::Arc::new(client));
        self
    }

    /// Enrich a [`ProductDescription`] with PubChem data.
    ///
    /// Fills in any missing fields of the main identifier and each mixture
    /// component's identifier (SMILES, InChI, InChIKey, IUPAC name, CID).
    ///
    /// This is a **best-effort** operation:
    /// - "Not found" and "no usable identifier" results are silently ignored.
    /// - Network / parse errors **are** propagated.
    /// - If no PubChem client is configured, returns `Ok(())` immediately.
    ///
    /// Requires the **`pubchem`** Cargo feature.
    #[cfg(feature = "pubchem")]
    pub async fn enrich(&self, product: &mut ProductDescription) -> Result<()> {
        let Some(ref client) = self.pubchem else {
            return Ok(());
        };

        client.enrich(&mut product.identifier).await?;

        if let Some(ref mut comps) = product.mixture_components {
            for comp in comps.iter_mut() {
                client.enrich(&mut comp.substance).await?;
            }
        }

        Ok(())
    }

    /// Classify a product and return an HS code prediction.
    ///
    /// Priority order:
    /// 0. Mixture branch (v0.5) — GRI 3a/3b/3c via [`crate::mixture`]
    /// 1. User-provided mapping
    /// 2. Embedded static rule table
    /// 3. (v0.3) SMILES rule engine
    /// 4. (v0.4) LLM fallback
    pub fn classify(&self, product: &ProductDescription) -> Result<HsPrediction> {
        // ── Priority 0: Mixture branch (v0.5) ────────────────────────────
        if product.is_mixture() {
            return crate::mixture::classify_mixture(product, |comp| self.classify(comp));
        }

        // ── Priority 1: User-provided mappings ────────────────────────
        if let Some(ref cas) = product.identifier.cas {
            if let Some(hs_code) = self.user_mappings.get(cas.as_str()) {
                let jp = find_jp_rule(hs_code);
                return Ok(HsPrediction {
                    hs_code: hs_code.clone(),
                    heading_description: String::new(),
                    confidence: 1.0,
                    source: PredictionSource::UserMapping,
                    notes: vec!["From user-provided mapping".to_string()],
                    alternatives: vec![],
                    recommended_action: RecommendedAction::Accept,
                    gray_zone: None,
                    jp_tariff_code: jp.map(|r| r.jp_code.to_string()),
                    jp_tariff_year: jp.map(|_| JP_TARIFF_YEAR),
                });
            }
        }

        // ── Priority 2: Embedded static rule table ────────────────────
        if let Some(ref cas) = product.identifier.cas {
            if let Some(rule) = find_best_rule(
                cas,
                product.physical_form.as_ref(),
                product.purity_pct,
            ) {
                let gray_zone = self.detect_gray_zone(product, rule.hs_code, None);
                let action = self.recommended_action_with_gz(rule.confidence, gray_zone.as_ref());
                let jp = find_jp_rule(rule.hs_code);
                return Ok(HsPrediction {
                    hs_code: rule.hs_code.to_string(),
                    heading_description: rule.heading_description.to_string(),
                    confidence: rule.confidence,
                    source: PredictionSource::EmbeddedRule {
                        rule_id: format!("{}:{}", rule.cas, rule.hs_code),
                    },
                    notes: self.build_notes(product),
                    alternatives: vec![],
                    recommended_action: action,
                    gray_zone,
                    jp_tariff_code: jp.map(|r| r.jp_code.to_string()),
                    jp_tariff_year: jp.map(|_| JP_TARIFF_YEAR),
                });
            }
        }

        // ── Priority 3: SMILES-based rule engine ─────────────────────────
        if let Some(ref smiles) = product.identifier.smiles {
            if let Some(classification) = crate::smiles::classify_smiles(smiles) {
                let hint = &classification.heading_hint;

                // Prefer the 6-digit subheading when the structural engine
                // resolved it; otherwise pad the 4-digit heading with "00".
                let maybe_code: Option<(String, bool)> = hint
                    .subheading
                    .as_ref()
                    .map(|sub| (sub.clone(), true))
                    .or_else(|| {
                        hint.heading
                            .map(|heading| (format!("{:04}00", heading), false))
                    });

                if let Some((hs_code, is_6digit)) = maybe_code {
                    if hint.confidence >= self.config.confidence_threshold_llm_required {
                        let jp = find_jp_rule(&hs_code);

                        let gray_zone = self.detect_gray_zone(
                            product,
                            &hs_code,
                            Some(&classification.organic_class),
                        );
                        let action =
                            self.recommended_action_with_gz(hint.confidence, gray_zone.as_ref());

                        let mut notes = self.build_notes(product);
                        if is_6digit {
                            notes.push(
                                "6-digit subheading resolved from SMILES structural analysis \
                                 (carbon count, ring type, functional group). \
                                 Verify with product specification before declaration."
                                    .to_string(),
                            );
                        } else {
                            notes.push(
                                "Heading derived from SMILES functional-group analysis. \
                                 Sub-heading (last two digits) is a placeholder — \
                                 verify the exact 6-digit code with the product specification."
                                    .to_string(),
                            );
                        }

                        let matched_rules: Vec<String> = classification
                            .functional_groups
                            .iter()
                            .map(|g| g.label().to_string())
                            .collect();

                        return Ok(HsPrediction {
                            hs_code,
                            heading_description: hint.rationale.to_string(),
                            confidence: hint.confidence,
                            source: PredictionSource::RuleEngine { matched_rules },
                            notes,
                            alternatives: vec![],
                            recommended_action: action,
                            gray_zone,
                            jp_tariff_code: jp.map(|r| r.jp_code.to_string()),
                            jp_tariff_year: jp.map(|_| JP_TARIFF_YEAR),
                        });
                    }
                }
            }
        }

        // ── Priority 4: LLM fallback ─────────────────────────────────
        // (async path — use classify_with_llm for LLM support)
        Err(HsPredictError::LowConfidenceNoLlm {
            confidence: 0.0,
            threshold: self.config.confidence_threshold_llm_required,
        })
    }

    /// Classify a batch of products concurrently.
    ///
    /// Returns one `Result<HsPrediction>` per input, in the same order.
    /// Uses synchronous [`classify`](Self::classify) internally — for LLM-backed
    /// batch classification see `classify_batch_with_llm` (future work).
    pub fn classify_batch(&self, products: &[ProductDescription]) -> Vec<Result<HsPrediction>> {
        products.iter().map(|p| self.classify(p)).collect()
    }

    /// Classify a batch of products using the async LLM path.
    ///
    /// Each product is classified via [`classify_with_llm`](Self::classify_with_llm).
    /// All requests are issued concurrently.
    ///
    /// Requires the **`llm`** Cargo feature.
    #[cfg(feature = "llm")]
    pub async fn classify_batch_with_llm(
        &self,
        products: &[ProductDescription],
    ) -> Vec<Result<HsPrediction>> {
        use futures::future::join_all;
        let futures: Vec<_> = products.iter().map(|p| self.classify_with_llm(p)).collect();
        join_all(futures).await
    }

    /// Classify a product, falling back to the configured LLM when the
    /// rule-based pipeline returns a low-confidence or uncertain result.
    ///
    /// # Priority order (same as [`classify`](Self::classify) + LLM)
    ///
    /// 1. User-provided mapping → `Accept` → return immediately.
    /// 2. Embedded static rule table → `Accept` → return immediately.
    /// 3. SMILES rule engine → `Accept` → return immediately.
    /// 4. Any result with `recommended_action != Accept`, or
    ///    `LowConfidenceNoLlm` → forward to LLM.
    ///
    /// If no LLM client has been configured via [`with_llm`](Self::with_llm),
    /// returns [`HsPredictError::LlmNotConfigured`].
    ///
    /// # Validation
    /// The LLM's `hs_code` must be exactly 6 ASCII digits; otherwise
    /// [`HsPredictError::ValidationFailed`] is returned.
    ///
    /// # Chapter consistency
    /// If the LLM chapter differs from the SMILES engine's chapter hint, a
    /// warning note is appended — this is **not** a hard error.
    ///
    /// Requires the **`llm`** Cargo feature.
    #[cfg(feature = "llm")]
    pub async fn classify_with_llm(
        &self,
        product: &ProductDescription,
    ) -> Result<HsPrediction> {
        use crate::llm::PromptBuilder;
        use crate::types::AlternativePrediction;

        // First try the synchronous rule-based pipeline.
        let needs_llm = match self.classify(product) {
            Ok(pred) if pred.recommended_action == RecommendedAction::Accept => {
                return Ok(pred);
            }
            Ok(_pred) => true,  // low-confidence result → try LLM
            Err(HsPredictError::LowConfidenceNoLlm { .. }) => true,
            Err(e) => return Err(e),
        };

        debug_assert!(needs_llm);

        // Require a configured LLM client.
        let llm = self
            .llm
            .as_ref()
            .ok_or(HsPredictError::LlmNotConfigured)?;

        // Build prompt and call the LLM.
        let prompt = PromptBuilder::new().build(product);
        let resp = llm.classify(&prompt).await?;

        // Validate: must be exactly 6 ASCII digits.
        if resp.hs_code.len() != 6 || !resp.hs_code.chars().all(|c| c.is_ascii_digit()) {
            return Err(HsPredictError::ValidationFailed { code: resp.hs_code });
        }

        // Chapter consistency check (warning only).
        let mut notes = self.build_notes(product);
        if let Some(ref analysis) = prompt.smiles_analysis {
            let llm_chapter = &resp.hs_code[..2];
            let expected_chapter = format!("{:02}", analysis.heading_hint.chapter);
            if llm_chapter != expected_chapter {
                notes.push(format!(
                    "Chapter mismatch: LLM returned Chapter {} but SMILES engine \
                     suggested Chapter {}. Verify with Chapter Notes.",
                    llm_chapter, expected_chapter
                ));
            }
        }

        notes.push(format!("LLM rationale: {}", resp.rationale));

        let jp = find_jp_rule(&resp.hs_code);
        let action = self.recommended_action(resp.confidence);

        // Only include alternatives whose hs_code passes the same 6-digit
        // format check applied to the primary result.
        let alternatives = resp
            .alternatives
            .into_iter()
            .filter(|a| a.hs_code.len() == 6 && a.hs_code.chars().all(|c| c.is_ascii_digit()))
            .map(|a| AlternativePrediction {
                hs_code: a.hs_code,
                confidence: a.confidence,
                reason: a.reason,
            })
            .collect();

        Ok(HsPrediction {
            hs_code: resp.hs_code,
            heading_description: String::new(),
            confidence: resp.confidence,
            source: PredictionSource::LlmApi { model: String::new() },
            notes,
            alternatives,
            recommended_action: action,
            gray_zone: None, // LLM response does not carry gray-zone information
            jp_tariff_code: jp.map(|r| r.jp_code.to_string()),
            jp_tariff_year: jp.map(|_| JP_TARIFF_YEAR),
        })
    }

    // ─── Private helpers ──────────────────────────────────────────────

    fn recommended_action(&self, confidence: f32) -> RecommendedAction {
        if confidence >= self.config.confidence_threshold_direct {
            RecommendedAction::Accept
        } else if confidence >= self.config.confidence_threshold_llm_required {
            RecommendedAction::VerifyWithLlm
        } else {
            RecommendedAction::ExpertReview
        }
    }

    /// Like `recommended_action` but upgrades to `PriorConsultation` when a
    /// gray zone is present and the confidence does not reach the "direct" threshold.
    fn recommended_action_with_gz(
        &self,
        confidence: f32,
        gray_zone: Option<&GrayZone>,
    ) -> RecommendedAction {
        let base = self.recommended_action(confidence);
        if gray_zone.is_some() && base != RecommendedAction::Accept {
            // Gray zone identified → recommend an advance ruling (事前教示)
            RecommendedAction::PriorConsultation
        } else {
            base
        }
    }

    /// Detect whether a prediction falls in a well-known gray zone.
    ///
    /// When `organic_class` is `Some`, the supplied classification is used
    /// (e.g. when the SMILES engine has already analysed the structure);
    /// otherwise the classification is re-derived from
    /// `product.identifier.smiles` when available.
    fn detect_gray_zone(
        &self,
        product: &ProductDescription,
        hs_code: &str,
        organic_class: Option<&OrganicInorganic>,
    ) -> Option<GrayZone> {
        let chapter = &hs_code[..2];

        // Chapter 28 / 29 boundary: organometallic or borderline compound
        if chapter == "28" && self.is_organometallic(product, organic_class) {
            return Some(GrayZone::Chapter28vs29);
        }

        // Chapter 29 result but product is used industrially → Ch.29 vs Ch.38
        if chapter == "29" {
            use crate::types::IntendedUse;
            if let Some(IntendedUse::Industrial) = &product.intended_use {
                return Some(GrayZone::Chapter29vs38);
            }
        }

        None
    }

    /// Whether the product is an organometallic compound — either via the
    /// pre-computed `organic_class` (preferred) or by re-deriving from SMILES.
    fn is_organometallic(
        &self,
        product: &ProductDescription,
        organic_class: Option<&OrganicInorganic>,
    ) -> bool {
        match organic_class {
            Some(oc) => matches!(oc, OrganicInorganic::Organometallic),
            None => product.identifier.smiles.as_deref().is_some_and(|s| {
                matches!(
                    crate::smiles::detector::classify_organic(s),
                    OrganicInorganic::Organometallic,
                )
            }),
        }
    }

    /// Build supplementary notes about shape / purity caveats.
    fn build_notes(&self, product: &ProductDescription) -> Vec<String> {
        let mut notes = Vec::new();

        match &product.physical_form {
            None | Some(PhysicalForm::Unknown) => {
                notes.push(
                    "Physical form not specified — the HS subheading may differ \
                     (e.g. solid vs. solution).".to_string(),
                );
            }
            Some(PhysicalForm::Solution { concentration_pct_ww: None, .. }) => {
                notes.push(
                    "Solution concentration not specified — subheading may differ \
                     (e.g. fuming vs. standard grade).".to_string(),
                );
            }
            _ => {}
        }

        if product.purity_pct.is_none() {
            notes.push(
                "Purity not specified — some headings require a minimum purity threshold."
                    .to_string(),
            );
        }

        notes
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pipeline integration tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::*;
    use crate::llm::MockLlmClassifier;
    use crate::types::{SubstanceIdentifier};

    /// A product with no static rule and a SMILES → triggers LLM path.
    fn unknown_organic() -> ProductDescription {
        // Ethyl propanoate (ester): SMILES engine gives heading 2915 at conf 0.55
        // (VerifyWithLlm) because esters don't have a 6-digit structural decision
        // tree yet.  No CAS → Priority 2 miss.  Suitable for testing LLM paths.
        ProductDescription {
            identifier: SubstanceIdentifier {
                cas: None,
                smiles: Some("CCC(=O)OCC".to_string()),
                iupac_name: None,
                inchi: None,
                inchi_key: None,
                cid: None,
            },
            physical_form: None,
            purity_pct: None,
            purity_type: None,
            mixture_components: None,
            intended_use: None,
            additional_context: None,
        }
    }

    #[tokio::test]
    async fn classify_with_llm_mock_returns_6_digit_code() {
        let pipeline = HsPipeline::new().with_llm(MockLlmClassifier::new());
        let product = unknown_organic();
        let pred = pipeline.classify_with_llm(&product).await.unwrap();
        assert_eq!(pred.hs_code.len(), 6);
        assert!(pred.hs_code.chars().all(|c| c.is_ascii_digit()));
    }

    #[tokio::test]
    async fn classify_with_llm_mock_chapter_29_for_smiles_acid() {
        let pipeline = HsPipeline::new().with_llm(MockLlmClassifier::new());
        let product = unknown_organic();
        let pred = pipeline.classify_with_llm(&product).await.unwrap();
        assert!(
            pred.hs_code.starts_with("29"),
            "acetic acid SMILES should yield Chapter 29, got {}",
            pred.hs_code
        );
    }

    #[tokio::test]
    async fn classify_with_llm_no_client_returns_error() {
        let pipeline = HsPipeline::new(); // no LLM attached
        let product = unknown_organic();
        let err = pipeline.classify_with_llm(&product).await.unwrap_err();
        assert!(
            matches!(err, HsPredictError::LlmNotConfigured),
            "expected LlmNotConfigured, got {:?}",
            err
        );
    }

    #[tokio::test]
    async fn classify_with_llm_skips_llm_for_high_confidence_rule() {
        // NaOH solid → static rule, confidence = 1.0 → should NOT call LLM
        let pipeline = HsPipeline::new()
            .with_llm(MockLlmClassifier::with_default("999999", 0.1));
        let product = ProductDescription {
            identifier: SubstanceIdentifier::from_cas("1310-73-2"),
            physical_form: Some(crate::types::PhysicalForm::Solid),
            purity_pct: None,
            purity_type: None,
            mixture_components: None,
            intended_use: None,
            additional_context: None,
        };
        let pred = pipeline.classify_with_llm(&product).await.unwrap();
        // Should be the static rule result, not the mock's "999999"
        assert_eq!(pred.hs_code, "281511", "static rule should win over LLM");
    }

    #[tokio::test]
    async fn classify_with_llm_invalid_code_returns_validation_error() {
        // Mock returning an invalid code
        struct BadMock;
        impl crate::llm::LlmClassifier for BadMock {
            fn classify<'a>(
                &'a self,
                _prompt: &'a crate::llm::LlmPrompt,
            ) -> futures::future::BoxFuture<'a, crate::Result<crate::llm::LlmResponse>> {
                Box::pin(async {
                    Ok(crate::llm::LlmResponse {
                        hs_code: "BAD!!".to_string(),
                        confidence: 0.5,
                        rationale: "bad".to_string(),
                        alternatives: vec![],
                    })
                })
            }
        }
        let pipeline = HsPipeline::new().with_llm(BadMock);
        let product = unknown_organic();
        let err = pipeline.classify_with_llm(&product).await.unwrap_err();
        assert!(
            matches!(err, HsPredictError::ValidationFailed { .. }),
            "expected ValidationFailed, got {:?}",
            err
        );
    }
}
