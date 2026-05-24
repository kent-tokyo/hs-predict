//! Main classification pipeline.
//!
//! Runs classification in priority order:
//! 1. User-provided CAS → HS mappings (confidence = 1.0)
//! 2. Embedded static rule table (CAS + shape + purity)
//! 3. *(placeholder)* SMILES-based rule engine — v0.3
//! 4. *(placeholder)* LLM API — v0.4

use std::collections::HashMap;

use crate::error::{HsPredictError, Result};
use crate::rules::jp_table::{find_jp_rule, JP_TARIFF_YEAR};
use crate::rules::matcher::find_best_rule;
use crate::types::{HsPrediction, PhysicalForm, ProductDescription, PredictionSource, RecommendedAction};

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
/// # Example
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
#[derive(Debug, Default)]
pub struct HsPipeline {
    /// User-supplied CAS → HS code overrides. Highest priority.
    user_mappings: HashMap<String, String>,

    config: PipelineConfig,
}

impl HsPipeline {
    /// Create a pipeline with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a user-provided CAS → HS code mapping.
    ///
    /// These mappings override the embedded rule table with `confidence = 1.0`.
    pub fn with_mapping(mut self, cas: impl Into<String>, hs_code: impl Into<String>) -> Self {
        self.user_mappings.insert(cas.into(), hs_code.into());
        self
    }

    /// Override the default pipeline configuration.
    pub fn with_config(mut self, config: PipelineConfig) -> Self {
        self.config = config;
        self
    }

    /// Classify a product and return an HS code prediction.
    ///
    /// Priority order:
    /// 1. User-provided mapping
    /// 2. Embedded static rule table
    /// 3. (v0.3) SMILES rule engine
    /// 4. (v0.4) LLM fallback
    pub fn classify(&self, product: &ProductDescription) -> Result<HsPrediction> {
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
                let action = self.recommended_action(rule.confidence);
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
                    jp_tariff_code: jp.map(|r| r.jp_code.to_string()),
                    jp_tariff_year: jp.map(|_| JP_TARIFF_YEAR),
                });
            }
        }

        // ── Priority 3: SMILES-based rule engine (v0.3 placeholder) ──
        // TODO: implement SMILES organic/inorganic detection and
        //       functional group → chapter mapping in v0.3.

        // ── Priority 4: LLM fallback (v0.4 placeholder) ───────────────
        // TODO: implement LLM client in v0.4.

        // No rule matched — return low-confidence placeholder
        Err(HsPredictError::LowConfidenceNoLlm {
            confidence: 0.0,
            threshold: self.config.confidence_threshold_llm_required,
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
