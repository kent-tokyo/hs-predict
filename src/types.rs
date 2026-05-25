use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────
// Language
// ─────────────────────────────────────────────

/// UI language for session question prompts.
///
/// Defaults to English. Pass [`Language::Ja`] to
/// [`ClassificationSession::with_language`](crate::session::ClassificationSession::with_language)
/// for Japanese prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// English (default)
    #[default]
    En,
    /// Japanese (日本語)
    Ja,
}

// ─────────────────────────────────────────────
// Substance identifier
// ─────────────────────────────────────────────

/// Set of identifiers for a single chemical compound.
///
/// Provide at least one field. When multiple fields are set, the pipeline
/// uses them in priority order: CAS → SMILES → InChIKey → InChI → IUPAC name.
///
/// **Important**: `iupac_name` must be an IUPAC systematic name.
/// Trade names and common aliases (e.g. "caustic soda") are not accepted
/// because they cannot be reliably resolved in PubChem.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubstanceIdentifier {
    /// CAS registry number (e.g. `"1310-73-2"`).
    pub cas: Option<String>,
    /// Canonical SMILES string (e.g. `"[Na+].[OH-]"`).
    pub smiles: Option<String>,
    /// IUPAC systematic name (e.g. `"sodium hydroxide"`).
    ///
    /// Do not use trade names or common aliases.
    pub iupac_name: Option<String>,
    /// InChI string (e.g. `"InChI=1S/Na.H2O/h;1H/q+1;/p-1"`).
    pub inchi: Option<String>,
    /// 27-character InChIKey (e.g. `"HEMHJVSKTPXQMS-UHFFFAOYSA-M"`).
    pub inchi_key: Option<String>,
    /// PubChem Compound ID — set automatically after a PubChem lookup.
    pub cid: Option<u64>,
}

impl SubstanceIdentifier {
    pub fn from_cas(cas: impl Into<String>) -> Self {
        Self { cas: Some(cas.into()), ..Default::default() }
    }

    pub fn from_smiles(smiles: impl Into<String>) -> Self {
        Self { smiles: Some(smiles.into()), ..Default::default() }
    }

    pub fn from_iupac_name(name: impl Into<String>) -> Self {
        Self { iupac_name: Some(name.into()), ..Default::default() }
    }

    /// Returns `true` when no identifier field has been set.
    pub fn is_empty(&self) -> bool {
        self.cas.is_none()
            && self.smiles.is_none()
            && self.iupac_name.is_none()
            && self.inchi.is_none()
            && self.inchi_key.is_none()
            && self.cid.is_none()
    }

    /// Short display string for logging and error messages.
    pub fn display_name(&self) -> String {
        if let Some(ref n) = self.iupac_name {
            return n.clone();
        }
        if let Some(ref cas) = self.cas {
            return format!("CAS:{}", cas);
        }
        if let Some(cid) = self.cid {
            return format!("CID:{}", cid);
        }
        if let Some(ref s) = self.smiles {
            // Use char-based slicing to avoid panicking on multi-byte UTF-8 sequences.
            let short: String = s.chars().take(20).collect();
            return format!("SMILES:{}", short);
        }
        "(unknown)".to_string()
    }
}

// ─────────────────────────────────────────────
// Physical form
// ─────────────────────────────────────────────

/// Physical state / form of the chemical product.
///
/// The same compound can have different HS subheadings depending on its form.
/// For example, sodium hydroxide solid → 2815.11, aqueous solution → 2815.12.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PhysicalForm {
    /// Solid bulk material (lumps, pellets, flakes, rods, …).
    Solid,
    /// Fine-grained powder.
    Powder {
        /// Median particle size in micrometres. `None` if unknown.
        particle_size_um: Option<f64>,
    },
    /// Coarser granulated product.
    Granules,
    /// Pure liquid (not a solution).
    Liquid,
    /// Solution of the substance in a solvent.
    Solution {
        /// Solvent IUPAC name. `None` implies water.
        solvent: Option<String>,
        /// Solute concentration in w/w%. `None` if unknown.
        concentration_pct_ww: Option<f64>,
    },
    /// Gas or vapour.
    Gas,
    /// Thin metal sheet.
    Foil {
        /// Thickness in millimetres. `None` if unknown.
        thickness_mm: Option<f64>,
    },
    /// Cast metal product (ingot, billet, slab, …).
    Ingot,
    /// Form not yet determined (initial session value).
    Unknown,
}

impl PhysicalForm {
    /// Returns `true` if this is a solution variant.
    pub fn is_solution(&self) -> bool {
        matches!(self, PhysicalForm::Solution { .. })
    }

    /// Returns the concentration (w/w%) if this is a solution with known concentration.
    pub fn concentration_pct(&self) -> Option<f64> {
        if let PhysicalForm::Solution { concentration_pct_ww, .. } = self {
            *concentration_pct_ww
        } else {
            None
        }
    }
}

// ─────────────────────────────────────────────
// Purity
// ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PurityType {
    ReagentGrade,
    TechnicalGrade,
    /// Pharmaceutical grade (PhEur / USP / JP, etc.).
    PharmaceuticalGrade { standard: Option<String> },
    FoodGrade,
    ElectronicsGrade,
    /// Numeric purity value in % (0.0–100.0).
    Specified(f64),
}

// ─────────────────────────────────────────────
// Mixture component
// ─────────────────────────────────────────────

/// A single component of a mixture product.
///
/// Set either `weight_fraction_pct` or `volume_fraction_pct`, not both.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixtureComponent {
    /// Identifier for this component substance.
    pub substance: SubstanceIdentifier,
    /// Weight fraction in w/w%. The sum of all components need not equal 100
    /// (remaining fraction may be unknown).
    pub weight_fraction_pct: Option<f64>,
    /// Volume fraction in v/v%. Mutually exclusive with `weight_fraction_pct`.
    pub volume_fraction_pct: Option<f64>,
    /// Marks this component as the solvent (for solution products).
    pub is_solvent: bool,
}

// ─────────────────────────────────────────────
// Product description (pipeline input)
// ─────────────────────────────────────────────

/// Complete description of a product for HS code classification.
///
/// Build this struct via [`ClassificationSession`](crate::session::ClassificationSession)
/// or fill it directly and pass it to
/// [`HsPipeline::classify`](crate::pipeline::HsPipeline::classify).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDescription {
    /// Primary identifier (CAS / SMILES / IUPAC name / InChI, etc.).
    pub identifier: SubstanceIdentifier,

    /// Physical form of the product. `None` means unknown.
    pub physical_form: Option<PhysicalForm>,

    /// Purity in % (0.0–100.0). `None` means unspecified.
    pub purity_pct: Option<f64>,

    /// Qualitative purity category.
    pub purity_type: Option<PurityType>,

    /// Component list for mixture products. `None` means pure substance.
    pub mixture_components: Option<Vec<MixtureComponent>>,

    /// Intended end-use of the product.
    pub intended_use: Option<IntendedUse>,

    /// Free-form additional context forwarded to the LLM prompt.
    pub additional_context: Option<String>,
}

/// Intended end-use category (influences chapter selection for special cases
/// such as pharmaceuticals → Ch. 30, fertilisers → Ch. 31).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntendedUse {
    Industrial,
    Pharmaceutical,
    Agricultural,
    Food,
    Cosmetic,
    Other(String),
}

impl ProductDescription {
    /// Returns `true` if the product has mixture components set.
    pub fn is_mixture(&self) -> bool {
        self.mixture_components
            .as_ref()
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
}

// ─────────────────────────────────────────────
// Prediction result
// ─────────────────────────────────────────────

/// HS code prediction result returned by the classification pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsPrediction {
    /// Six-digit HS 2022 code without punctuation (e.g. `"281511"`).
    pub hs_code: String,
    /// Official HS 2022 heading description for this code.
    pub heading_description: String,
    /// Confidence score in the range [0.0, 1.0].
    pub confidence: f32,
    /// Which part of the pipeline produced this prediction.
    pub source: PredictionSource,
    /// Supplementary notes (shape caveats, concentration notes, etc.).
    pub notes: Vec<String>,
    /// Alternative HS codes worth considering.
    pub alternatives: Vec<AlternativePrediction>,
    /// Recommended next action for the user.
    pub recommended_action: RecommendedAction,

    /// Classification boundary risk indicator (v0.5).
    ///
    /// `Some` when the prediction falls in a well-known misclassification gray
    /// zone (e.g. Chapter 29 vs 38 for organic preparations).  When present,
    /// consider requesting an advance ruling from customs authorities (事前教示).
    pub gray_zone: Option<GrayZone>,

    /// Nine-digit Japan statistical item code (統計品目番号).
    ///
    /// Based on Japan Customs 実行関税率表. Updated annually; the year used
    /// is indicated by the `jp_tariff_year` field.
    /// `None` when no Japan-specific code is registered for this HS heading.
    pub jp_tariff_code: Option<String>,

    /// Tariff schedule year used for the `jp_tariff_code` field (e.g. `2026`).
    pub jp_tariff_year: Option<u16>,
}

impl HsPrediction {
    /// Two-digit chapter code (e.g. `"28"`).
    ///
    /// Returns `&self.hs_code` unchanged if the code is shorter than 2 ASCII
    /// digits (which should not happen for valid predictions).
    pub fn chapter(&self) -> &str {
        self.hs_code.get(..2).unwrap_or(&self.hs_code)
    }

    /// Four-digit heading code (e.g. `"2815"`).
    ///
    /// Returns `&self.hs_code` unchanged if the code is shorter than 4 ASCII
    /// digits (which should not happen for valid predictions).
    pub fn heading(&self) -> &str {
        self.hs_code.get(..4).unwrap_or(&self.hs_code)
    }

    /// Dot-separated display string (e.g. `"28.15.11"`).
    pub fn display(&self) -> String {
        let c = &self.hs_code;
        if c.len() == 6 {
            format!("{}.{}.{}", &c[..2], &c[2..4], &c[4..6])
        } else {
            c.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativePrediction {
    pub hs_code: String,
    pub confidence: f32,
    pub reason: String,
}

/// Which part of the pipeline produced the prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PredictionSource {
    /// From the user's own CAS → HS mapping (highest trust).
    UserMapping,
    /// From the embedded compile-time rule table.
    EmbeddedRule { rule_id: String },
    /// From the SMILES-based rule engine (v0.3).
    RuleEngine { matched_rules: Vec<String> },
    /// From an LLM API call (v0.4).
    LlmApi { model: String },
    /// Combined rule-engine pre-classification + LLM final decision.
    Hybrid { rule_id: String, model: String },
}

/// Recommended follow-up action for the customs practitioner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendedAction {
    /// High-confidence result — safe to use in a customs declaration.
    Accept,
    /// Moderate-confidence result — recommend LLM or manual review.
    VerifyWithLlm,
    /// A formal advance ruling (prior consultation / 事前教示) is recommended.
    ///
    /// Applied when a [`GrayZone`] boundary is detected or when mixture
    /// classification falls back to GRI 3c (last heading by number).
    /// Contact your local customs authority for a binding ruling before declaration.
    PriorConsultation,
    /// Low-confidence result — consult a qualified trade-compliance expert.
    ExpertReview,
}

/// Identifies a classification boundary where misclassification risk is elevated.
///
/// When present in [`HsPrediction::gray_zone`], consider requesting a formal
/// advance ruling (事前教示 / binding tariff information) from customs authorities
/// before making a customs declaration. Misclassification of chemicals can result
/// in retroactive duty assessments going back up to five years.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrayZone {
    /// Chapter 29 (pure organic chemicals) vs Chapter 38 (prepared/mixed products).
    ///
    /// Occurs when an organic compound is part of a formulation, has industrial
    /// use, or when purity / presentation may shift classification to Ch. 38.
    /// Multi-function additives and flame-retardant mixtures are common examples.
    Chapter29vs38,

    /// Chapter 28 (inorganic chemicals) vs Chapter 29 (organic chemicals).
    ///
    /// Occurs for organometallic compounds or borderline organic/inorganic cases
    /// where the presence of metal–carbon bonds determines the correct chapter.
    Chapter28vs29,

    /// Mixture where essential character (GRI 3b) is ambiguous.
    ///
    /// No single component exceeds 50 % w/w, so GRI 3c (last heading by number)
    /// was applied with low confidence. An expert or advance ruling is strongly
    /// recommended.
    MixtureEssentialCharacterUnclear,
}

// ─────────────────────────────────────────────
// Organic / inorganic classification
// ─────────────────────────────────────────────

/// Result of SMILES-based organic / inorganic detection (v0.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganicInorganic {
    Organic,
    Inorganic,
    /// Compound with a direct metal–carbon bond.
    Organometallic,
    Unknown,
}
