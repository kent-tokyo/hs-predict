//! Mock LLM classifier for unit testing.
//!
//! Enabled by the **`mock`** Cargo feature (which implies `llm`).
//!
//! [`MockLlmClassifier`] is a deterministic stub that derives an HS code
//! directly from the SMILES pre-analysis embedded in the prompt, or falls back
//! to a configurable default.  It never makes a network call.
//!
//! # Example
//! ```rust
//! # #[cfg(all(feature = "llm", feature = "mock"))]
//! # {
//! use hs_predict::llm::{MockLlmClassifier, LlmClassifier, LlmPrompt};
//! use hs_predict::llm::PromptBuilder;
//! use hs_predict::types::{ProductDescription, SubstanceIdentifier, PhysicalForm};
//!
//! # tokio_test::block_on(async {
//! let product = ProductDescription {
//!     identifier: SubstanceIdentifier {
//!         cas: Some("64-19-7".to_string()),
//!         smiles: Some("CC(O)=O".to_string()),
//!         ..Default::default()
//!     },
//!     physical_form: Some(PhysicalForm::Liquid),
//!     purity_pct: Some(99.5),
//!     purity_type: None,
//!     mixture_components: None,
//!     intended_use: None,
//!     additional_context: None,
//! };
//!
//! let prompt = PromptBuilder::new().build(&product);
//! let mock = MockLlmClassifier::new();
//! let response = mock.classify(&prompt).await.unwrap();
//! assert_eq!(response.hs_code.len(), 6);
//! # });
//! # }
//! ```

#[cfg(feature = "mock")]
pub use inner::MockLlmClassifier;

#[cfg(feature = "mock")]
mod inner {
    use futures::future::BoxFuture;
    use crate::llm::{LlmAlternative, LlmClassifier, LlmPrompt, LlmResponse};

    /// Deterministic mock LLM classifier for unit tests.
    ///
    /// Resolution order:
    /// 1. If the prompt contains a SMILES analysis with a 4-digit heading hint,
    ///    return that heading padded to 6 digits (`XXXX00`) with confidence from
    ///    the SMILES engine.
    /// 2. Otherwise return the configured `default_hs_code` (default: `"999999"`).
    ///
    /// The mock never makes a network call and is fully `Send + Sync`.
    #[derive(Debug, Clone)]
    pub struct MockLlmClassifier {
        /// HS code returned when no SMILES heading hint is available.
        pub default_hs_code: String,
        /// Confidence for the default code.
        pub default_confidence: f32,
    }

    impl Default for MockLlmClassifier {
        fn default() -> Self {
            Self {
                default_hs_code: "999999".to_string(),
                default_confidence: 0.50,
            }
        }
    }

    impl MockLlmClassifier {
        /// Create a mock with the default fallback code `"999999"`.
        pub fn new() -> Self {
            Self::default()
        }

        /// Create a mock that always returns the specified HS code.
        pub fn with_default(hs_code: impl Into<String>, confidence: f32) -> Self {
            Self {
                default_hs_code: hs_code.into(),
                default_confidence: confidence,
            }
        }
    }

    impl LlmClassifier for MockLlmClassifier {
        fn classify<'a>(
            &'a self,
            prompt: &'a LlmPrompt,
        ) -> BoxFuture<'a, crate::Result<LlmResponse>> {
            Box::pin(async move {
                // Derive answer from SMILES analysis if available
                if let Some(ref analysis) = prompt.smiles_analysis {
                    let hint = &analysis.heading_hint;
                    if let Some(heading) = hint.heading {
                        let hs_code = format!("{:04}00", heading);
                        return Ok(LlmResponse {
                            hs_code,
                            confidence: hint.confidence,
                            rationale: format!(
                                "Mock: derived from SMILES analysis ({}). \
                                 Sub-heading is a placeholder.",
                                hint.rationale
                            ),
                            alternatives: vec![],
                        });
                    }
                    // Have analysis but no heading — use chapter
                    let hs_code = format!("{:02}0000", hint.chapter);
                    return Ok(LlmResponse {
                        hs_code,
                        confidence: hint.confidence * 0.8,
                        rationale: format!(
                            "Mock: chapter-level hint only (Ch.{:02}, {}).",
                            hint.chapter, hint.rationale
                        ),
                        alternatives: vec![],
                    });
                }

                // No analysis — return default
                Ok(LlmResponse {
                    hs_code: self.default_hs_code.clone(),
                    confidence: self.default_confidence,
                    rationale: "Mock classifier — no SMILES analysis available.".to_string(),
                    alternatives: vec![LlmAlternative {
                        hs_code: "000000".to_string(),
                        confidence: 0.0,
                        reason: "Placeholder alternative from mock.".to_string(),
                    }],
                })
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::MockLlmClassifier;
    use crate::llm::{LlmClassifier, PromptBuilder};
    use crate::types::{ProductDescription, SubstanceIdentifier, PhysicalForm};

    fn acetic_acid_product() -> ProductDescription {
        ProductDescription {
            identifier: SubstanceIdentifier {
                cas: Some("64-19-7".to_string()),
                smiles: Some("CC(O)=O".to_string()),
                iupac_name: None,
                inchi: None,
                inchi_key: None,
                cid: None,
            },
            physical_form: Some(PhysicalForm::Liquid),
            purity_pct: Some(99.5),
            purity_type: None,
            mixture_components: None,
            intended_use: None,
            additional_context: None,
        }
    }

    #[tokio::test]
    async fn mock_smiles_based_returns_6_digits() {
        let product = acetic_acid_product();
        let prompt = PromptBuilder::new().build(&product);
        let mock = MockLlmClassifier::new();
        let resp = mock.classify(&prompt).await.unwrap();
        assert_eq!(resp.hs_code.len(), 6, "hs_code must be 6 digits");
        assert!(resp.hs_code.chars().all(|c| c.is_ascii_digit()));
    }

    #[tokio::test]
    async fn mock_smiles_based_derives_chapter_29() {
        // Acetic acid → carboxylic acid → heading 2915 → "291500"
        let product = acetic_acid_product();
        let prompt = PromptBuilder::new().build(&product);
        let mock = MockLlmClassifier::new();
        let resp = mock.classify(&prompt).await.unwrap();
        assert!(
            resp.hs_code.starts_with("29"),
            "acetic acid should be Chapter 29, got {}",
            resp.hs_code
        );
    }

    #[tokio::test]
    async fn mock_no_smiles_returns_default() {
        let product = ProductDescription {
            identifier: SubstanceIdentifier::from_cas("64-19-7"),
            physical_form: None,
            purity_pct: None,
            purity_type: None,
            mixture_components: None,
            intended_use: None,
            additional_context: None,
        };
        let prompt = PromptBuilder::new().build(&product);
        let mock = MockLlmClassifier::new();
        let resp = mock.classify(&prompt).await.unwrap();
        assert_eq!(resp.hs_code, "999999");
    }

    #[tokio::test]
    async fn mock_custom_default_returned_when_no_smiles() {
        let product = ProductDescription {
            identifier: SubstanceIdentifier::from_cas("64-19-7"),
            physical_form: None,
            purity_pct: None,
            purity_type: None,
            mixture_components: None,
            intended_use: None,
            additional_context: None,
        };
        let prompt = PromptBuilder::new().build(&product);
        let mock = MockLlmClassifier::with_default("291511", 0.85);
        let resp = mock.classify(&prompt).await.unwrap();
        assert_eq!(resp.hs_code, "291511");
        assert!((resp.confidence - 0.85).abs() < 0.001);
    }

    #[tokio::test]
    async fn mock_confidence_nonzero_with_smiles() {
        let product = acetic_acid_product();
        let prompt = PromptBuilder::new().build(&product);
        let mock = MockLlmClassifier::new();
        let resp = mock.classify(&prompt).await.unwrap();
        assert!(resp.confidence > 0.0);
    }
}
