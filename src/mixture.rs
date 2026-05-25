//! Mixture classification using the WCO General Rules for Interpretation (GRI).
//!
//! Mixtures and preparations of chemical substances must be classified according
//! to the General Rules for the Interpretation of the Harmonized System (GRIs):
//!
//! | GRI | Principle |
//! |-----|-----------|
//! | 3a  | Most specific description (when all components share the same chapter, use the most specific heading) |
//! | 3b  | Essential character (classify by the component that gives the mixture its essential character, typically the dominant component by weight ≥ 50 % w/w) |
//! | 3c  | Last heading numerically (when 3b cannot determine essential character, use the heading that occurs last among those equally worthy of consideration) |
//!
//! ## Chapter priority (special cases handled before GRI 3)
//! Before applying GRI 3, the intended use is checked. Some uses are so
//! determinative that they override structural analysis:
//!
//! | Intended use | Chapter |
//! |---|---|
//! | Pharmaceutical | Ch. 30 |
//! | Agricultural (pesticide formulation) | Ch. 38.08 |
//! | Cosmetic / beauty | Ch. 33 |
//! | Food additive / food preparation | Ch. 21 |

use crate::error::{HsPredictError, Result};
use crate::rules::chapter38::{
    classify_by_intended_use, special_chapter_by_use, CHAPTER38_CATCH_ALL_CODE,
    CHAPTER38_CATCH_ALL_DESC,
};
use crate::rules::jp_table::{find_jp_rule, JP_TARIFF_YEAR};
use crate::types::{
    GrayZone, HsPrediction, IntendedUse, MixtureComponent, ProductDescription,
    PredictionSource, RecommendedAction,
};

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Classify a mixture product using GRI 3 rules.
///
/// This function is called by [`HsPipeline::classify`](crate::pipeline::HsPipeline::classify)
/// when `product.is_mixture()` is `true`.
///
/// ## Classification steps
/// 1. Special use check (Ch. 30 / 33 / 21 / 38.08).
/// 2. Classify each component individually via `classify_component`.
/// 3. GRI 3a: all components in the same chapter → most specific heading.
/// 4. GRI 3b: dominant component (>50% w/w) → that component's classification.
/// 5. GRI 3c fallback: last heading by number (low confidence, gray zone set).
pub(crate) fn classify_mixture(
    product: &ProductDescription,
    classify_component: impl Fn(&ProductDescription) -> Result<HsPrediction>,
) -> Result<HsPrediction> {
    let components = product
        .mixture_components
        .as_ref()
        .filter(|v| !v.is_empty())
        .ok_or(HsPredictError::MissingIdentifier)?;

    // ── Step 0: Intended-use special chapters ─────────────────────────────
    if let Some(ref intended_use) = product.intended_use {
        // Non-Ch.38 special chapters (pharma, cosmetic, food)
        if let Some((hs_code, desc, confidence)) = special_chapter_by_use(intended_use) {
            return Ok(PredictionBuilder {
                hs_code: hs_code.to_string(),
                heading_description: desc.to_string(),
                confidence,
                source: PredictionSource::EmbeddedRule {
                    rule_id: "chapter38::special_use".to_string(),
                },
                notes: vec![format!(
                    "Mixture classified by intended use ({}); verify with Chapter Notes.",
                    intended_use_label(intended_use)
                )],
                gray_zone: None,
                recommended_action: RecommendedAction::VerifyWithLlm,
            }
            .build());
        }

        // Ch. 38 agricultural preparations
        if let Some((hs_code, desc, confidence)) = classify_by_intended_use(intended_use) {
            return Ok(PredictionBuilder {
                hs_code: hs_code.to_string(),
                heading_description: desc.to_string(),
                confidence,
                source: PredictionSource::EmbeddedRule {
                    rule_id: "chapter38::agricultural".to_string(),
                },
                notes: vec![
                    "Mixture classified by agricultural intended use → Ch. 38.08.".to_string(),
                    "Verify: active ingredient type and concentration may shift the sub-heading."
                        .to_string(),
                ],
                gray_zone: Some(GrayZone::Chapter29vs38),
                recommended_action: RecommendedAction::PriorConsultation,
            }
            .build());
        }
    }

    // ── Step 1: Classify each component individually ──────────────────────
    let mut component_preds: Vec<(Option<f64>, HsPrediction)> = Vec::new();
    let mut unclassified_count = 0usize;

    for comp in components {
        let comp_product = component_to_product(comp);
        match classify_component(&comp_product) {
            Ok(pred) => {
                component_preds.push((comp.weight_fraction_pct, pred));
            }
            Err(_) => {
                // Could not classify this component — track but continue
                unclassified_count += 1;
            }
        }
    }

    // If we couldn't classify any component, fall back to Ch.38 catch-all
    if component_preds.is_empty() {
        return Ok(ch38_catch_all(
            vec![
                "No components could be individually classified.".to_string(),
                "Review each component's CAS/SMILES and consult a trade-compliance expert."
                    .to_string(),
            ],
            0.35,
        ));
    }

    // ── Step 2: GRI 3a — all components in the same chapter ──────────────
    if let Some(gri3a_result) = try_gri3a(&component_preds) {
        return Ok(gri3a_result);
    }

    // ── Step 3: GRI 3b — dominant component (> 50 % w/w) ─────────────────
    if let Some(gri3b_result) = try_gri3b(&component_preds) {
        return Ok(gri3b_result);
    }

    // ── Step 4: GRI 3c — last heading numerically ─────────────────────────
    Ok(gri3c(&component_preds, unclassified_count))
}

// ─────────────────────────────────────────────────────────────────────────────
// GRI implementations
// ─────────────────────────────────────────────────────────────────────────────

/// GRI 3a: all components share the same 2-digit chapter → use the most
/// specific heading (the one with the highest confidence).
fn try_gri3a(
    component_preds: &[(Option<f64>, HsPrediction)],
) -> Option<HsPrediction> {
    if component_preds.is_empty() {
        return None;
    }

    let first_chapter = &component_preds[0].1.hs_code[..2];
    let all_same_chapter = component_preds
        .iter()
        .all(|(_, p)| &p.hs_code[..2] == first_chapter);

    if !all_same_chapter {
        return None;
    }

    // All in the same chapter → pick the most specific (highest confidence).
    // Use `unwrap_or(Ordering::Equal)` so that NaN confidences (which can arise
    // from user-constructed or LLM-supplied `HsPrediction`s) do not panic.
    let best = component_preds
        .iter()
        .max_by(|(_, a), (_, b)| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;

    let pred = &best.1;
    let confidence = (pred.confidence * 0.90).min(0.85); // slightly lower for mixture
    let recommended_action = if confidence >= 0.85 {
        RecommendedAction::Accept
    } else {
        RecommendedAction::VerifyWithLlm
    };

    Some(
        PredictionBuilder {
            hs_code: pred.hs_code.clone(),
            heading_description: pred.heading_description.clone(),
            confidence,
            source: PredictionSource::RuleEngine {
                matched_rules: vec!["GRI-3a: all components same chapter".to_string()],
            },
            notes: vec![format!(
                "GRI 3a applied: all {} component(s) are in Chapter {}. \
                 Most specific heading selected by confidence.",
                component_preds.len(),
                first_chapter
            )],
            gray_zone: None, // no gray zone when chapter is unambiguous
            recommended_action,
        }
        .build(),
    )
}

/// GRI 3b: if one component exceeds 50 % w/w, it determines the essential
/// character and the mixture is classified with that component's HS code.
fn try_gri3b(
    component_preds: &[(Option<f64>, HsPrediction)],
) -> Option<HsPrediction> {
    // Find a component with known weight fraction > 50 %
    let dominant = component_preds
        .iter()
        .find(|(frac, _)| frac.map(|f| f > 50.0).unwrap_or(false));

    let (frac, pred) = dominant?;
    let fraction = frac.unwrap();
    let confidence = (pred.confidence * 0.88).min(0.82); // slightly lower for mixture

    // A dominant organic component in Ch.29 still has Ch.29 vs Ch.38 risk
    let gray_zone = if pred.hs_code.starts_with("29") {
        Some(GrayZone::Chapter29vs38)
    } else {
        None
    };

    let recommended_action = match (&gray_zone, confidence >= 0.75) {
        (Some(_), _) => RecommendedAction::PriorConsultation,
        (None, true) => RecommendedAction::VerifyWithLlm,
        (None, false) => RecommendedAction::ExpertReview,
    };

    let mut notes = vec![format!(
        "GRI 3b applied: dominant component ({:.1}% w/w) determines essential character.",
        fraction
    )];
    if gray_zone.is_some() {
        notes.push(
            "Chapter 29 vs 38 boundary: verify whether this mixture is sold as a \
             pure substance (Ch. 29) or as a prepared formulation (Ch. 38)."
                .to_string(),
        );
    }

    Some(
        PredictionBuilder {
            hs_code: pred.hs_code.clone(),
            heading_description: pred.heading_description.clone(),
            confidence,
            source: PredictionSource::RuleEngine {
                matched_rules: vec![format!("GRI-3b: dominant component {:.1}% w/w", fraction)],
            },
            notes,
            gray_zone,
            recommended_action,
        }
        .build(),
    )
}

/// GRI 3c: when essential character cannot be determined, use the heading that
/// occurs last in numeric order among all candidate headings.
fn gri3c(
    component_preds: &[(Option<f64>, HsPrediction)],
    unclassified_count: usize,
) -> HsPrediction {
    // Sort by HS code string (lexicographic = numeric for 6-digit codes)
    let last = component_preds
        .iter()
        .max_by(|(_, a), (_, b)| a.hs_code.cmp(&b.hs_code));

    if let Some((_, pred)) = last {
        let mut notes = vec![
            "GRI 3c applied: essential character could not be determined (no dominant \
             component >50% w/w); last heading by numeric order was used."
                .to_string(),
            "Confidence is LOW. An advance ruling (事前教示) from customs is strongly \
             recommended before making a declaration."
                .to_string(),
        ];
        if unclassified_count > 0 {
            notes.push(format!(
                "{} component(s) could not be classified individually and were excluded.",
                unclassified_count
            ));
        }

        PredictionBuilder {
            hs_code: pred.hs_code.clone(),
            heading_description: pred.heading_description.clone(),
            confidence: 0.40,
            source: PredictionSource::RuleEngine {
                matched_rules: vec!["GRI-3c: last heading numerically".to_string()],
            },
            notes,
            gray_zone: Some(GrayZone::MixtureEssentialCharacterUnclear),
            recommended_action: RecommendedAction::PriorConsultation,
        }
        .build()
    } else {
        ch38_catch_all(
            vec![
                "GRI 3c could not be applied (no components classified).".to_string(),
                "Ch. 38 NEC catch-all used as last resort.".to_string(),
            ],
            0.30,
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build a Ch.38 catch-all prediction for unclassifiable mixtures.
fn ch38_catch_all(notes: Vec<String>, confidence: f32) -> HsPrediction {
    PredictionBuilder {
        hs_code: CHAPTER38_CATCH_ALL_CODE.to_string(),
        heading_description: CHAPTER38_CATCH_ALL_DESC.to_string(),
        confidence,
        source: PredictionSource::RuleEngine {
            matched_rules: vec!["chapter38::catch_all".to_string()],
        },
        notes,
        gray_zone: Some(GrayZone::Chapter29vs38),
        recommended_action: RecommendedAction::PriorConsultation,
    }
    .build()
}

/// Convert a [`MixtureComponent`] into a [`ProductDescription`] suitable for
/// single-substance classification.
fn component_to_product(comp: &MixtureComponent) -> ProductDescription {
    ProductDescription {
        identifier: comp.substance.clone(),
        physical_form: None, // form not relevant for component classification
        purity_pct: None,
        purity_type: None,
        mixture_components: None, // treat as pure substance
        intended_use: None,
        additional_context: None,
    }
}

/// Builder for [`HsPrediction`] used by mixture classification.
///
/// All fields use named-struct literal initialisation at the call site, which
/// avoids the readability cost of a 9-argument function. The JP tariff lookup
/// is performed centrally in [`PredictionBuilder::build`].
struct PredictionBuilder {
    hs_code: String,
    heading_description: String,
    confidence: f32,
    source: PredictionSource,
    notes: Vec<String>,
    gray_zone: Option<GrayZone>,
    recommended_action: RecommendedAction,
}

impl PredictionBuilder {
    /// Finalise the builder into an [`HsPrediction`], populating Japan-specific
    /// tariff fields from the embedded JP table.
    fn build(self) -> HsPrediction {
        let jp = find_jp_rule(&self.hs_code);
        HsPrediction {
            hs_code: self.hs_code,
            heading_description: self.heading_description,
            confidence: self.confidence,
            source: self.source,
            notes: self.notes,
            alternatives: vec![],
            recommended_action: self.recommended_action,
            gray_zone: self.gray_zone,
            jp_tariff_code: jp.map(|r| r.jp_code.to_string()),
            jp_tariff_year: jp.map(|_| JP_TARIFF_YEAR),
        }
    }
}

fn intended_use_label(use_: &IntendedUse) -> &'static str {
    match use_ {
        IntendedUse::Pharmaceutical => "pharmaceutical",
        IntendedUse::Agricultural => "agricultural",
        IntendedUse::Cosmetic => "cosmetic",
        IntendedUse::Food => "food",
        IntendedUse::Industrial => "industrial",
        IntendedUse::Other(_) => "other",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MixtureComponent, SubstanceIdentifier};

    fn make_pred(hs_code: &str, confidence: f32) -> HsPrediction {
        HsPrediction {
            hs_code: hs_code.to_string(),
            heading_description: format!("Test heading for {}", hs_code),
            confidence,
            source: PredictionSource::EmbeddedRule { rule_id: "test".to_string() },
            notes: vec![],
            alternatives: vec![],
            recommended_action: RecommendedAction::Accept,
            gray_zone: None,
            jp_tariff_code: None,
            jp_tariff_year: None,
        }
    }

    // Build a component with a given CAS and optional weight fraction
    fn comp(cas: &str, weight_pct: Option<f64>) -> MixtureComponent {
        MixtureComponent {
            substance: SubstanceIdentifier::from_cas(cas),
            weight_fraction_pct: weight_pct,
            volume_fraction_pct: None,
            is_solvent: false,
        }
    }

    #[test]
    fn gri3a_same_chapter_picks_highest_confidence() {
        let preds = vec![
            (Some(40.0), make_pred("290511", 0.97)), // methanol Ch.29
            (Some(60.0), make_pred("290531", 0.90)), // ethylene glycol Ch.29
        ];
        let result = try_gri3a(&preds).unwrap();
        assert_eq!(&result.hs_code, "290511"); // higher confidence wins
        assert!(result.gray_zone.is_none());
    }

    #[test]
    fn gri3a_different_chapters_returns_none() {
        let preds = vec![
            (Some(50.0), make_pred("281511", 0.97)), // NaOH Ch.28
            (Some(50.0), make_pred("290511", 0.97)), // methanol Ch.29
        ];
        assert!(try_gri3a(&preds).is_none());
    }

    #[test]
    fn gri3b_dominant_component_wins() {
        let preds = vec![
            (Some(70.0), make_pred("280700", 0.97)), // sulphuric acid 70%
            (Some(30.0), make_pred("290531", 0.97)), // ethylene glycol 30%
        ];
        let result = try_gri3b(&preds).unwrap();
        assert_eq!(&result.hs_code, "280700");
    }

    #[test]
    fn gri3b_no_dominant_returns_none() {
        let preds = vec![
            (Some(40.0), make_pred("280700", 0.97)),
            (Some(40.0), make_pred("290511", 0.97)),
        ];
        assert!(try_gri3b(&preds).is_none());
    }

    #[test]
    fn gri3b_ch29_sets_gray_zone() {
        let preds = vec![
            (Some(60.0), make_pred("290531", 0.97)), // ethylene glycol 60%
            (Some(40.0), make_pred("280700", 0.90)), // sulphuric acid 40%
        ];
        let result = try_gri3b(&preds).unwrap();
        assert_eq!(result.gray_zone, Some(GrayZone::Chapter29vs38));
        assert_eq!(result.recommended_action, RecommendedAction::PriorConsultation);
    }

    #[test]
    fn gri3c_picks_last_heading_numerically() {
        let preds = vec![
            (Some(35.0), make_pred("280700", 0.90)), // 280700
            (Some(35.0), make_pred("290511", 0.90)), // 290511 (higher)
            (Some(30.0), make_pred("280610", 0.90)), // 280610
        ];
        let result = gri3c(&preds, 0);
        assert_eq!(&result.hs_code, "290511");
        assert_eq!(result.gray_zone, Some(GrayZone::MixtureEssentialCharacterUnclear));
        assert_eq!(result.recommended_action, RecommendedAction::PriorConsultation);
        assert!(result.confidence <= 0.40);
    }

    #[test]
    fn pharmaceutical_use_gives_ch30() {
        let product = ProductDescription {
            identifier: SubstanceIdentifier::default(),
            physical_form: None,
            purity_pct: None,
            purity_type: None,
            mixture_components: Some(vec![comp("64-17-5", Some(50.0))]),
            intended_use: Some(IntendedUse::Pharmaceutical),
            additional_context: None,
        };
        let result = classify_mixture(&product, |_p| {
            Ok(make_pred("290511", 0.97))
        })
        .unwrap();
        assert_eq!(&result.hs_code[..2], "30");
    }

    #[test]
    fn agricultural_use_gives_ch38() {
        let product = ProductDescription {
            identifier: SubstanceIdentifier::default(),
            physical_form: None,
            purity_pct: None,
            purity_type: None,
            mixture_components: Some(vec![comp("64-17-5", Some(50.0))]),
            intended_use: Some(IntendedUse::Agricultural),
            additional_context: None,
        };
        let result = classify_mixture(&product, |_p| {
            Ok(make_pred("290511", 0.97))
        })
        .unwrap();
        assert_eq!(&result.hs_code[..2], "38");
        assert_eq!(result.recommended_action, RecommendedAction::PriorConsultation);
    }

    #[test]
    fn empty_components_returns_error() {
        let product = ProductDescription {
            identifier: SubstanceIdentifier::default(),
            physical_form: None,
            purity_pct: None,
            purity_type: None,
            mixture_components: Some(vec![]),
            intended_use: None,
            additional_context: None,
        };
        let result = classify_mixture(&product, |_p| Ok(make_pred("290511", 0.97)));
        assert!(result.is_err());
    }

    /// A mixture where all component weights are None should fall through to
    /// GRI 3c (no dominant component can be found → GRI 3b skipped).
    #[test]
    fn all_unknown_weights_falls_to_gri3c() {
        let preds = vec![
            (None, make_pred("280700", 0.90)), // sulphuric acid — no weight
            (None, make_pred("290511", 0.90)), // methanol — no weight, higher HS code
        ];
        // GRI 3b needs a weight fraction; with all None it should return None
        assert!(try_gri3b(&preds).is_none(), "GRI 3b must return None when all weights are unknown");
        // GRI 3c picks last heading numerically
        let result = gri3c(&preds, 0);
        assert_eq!(&result.hs_code, "290511");
        assert_eq!(result.gray_zone, Some(GrayZone::MixtureEssentialCharacterUnclear));
        assert_eq!(result.recommended_action, RecommendedAction::PriorConsultation);
    }

    /// A single-component mixture: GRI 3a applies (trivially, one chapter).
    #[test]
    fn single_component_mixture_classifies_via_gri3a() {
        let preds = vec![(Some(100.0), make_pred("290511", 0.97))];
        let result = try_gri3a(&preds);
        assert!(result.is_some(), "GRI 3a must succeed for a single-component mixture");
        let result = result.unwrap();
        assert_eq!(&result.hs_code, "290511");
    }

    /// GRI 3b boundary condition: exactly 50.0% w/w must NOT qualify as dominant
    /// (the threshold is strictly > 50.0).
    #[test]
    fn gri3b_exactly_50pct_is_not_dominant() {
        let preds = vec![
            (Some(50.0), make_pred("280700", 0.97)),
            (Some(50.0), make_pred("290511", 0.97)),
        ];
        assert!(
            try_gri3b(&preds).is_none(),
            "50.0% is not strictly > 50.0; GRI 3b must return None"
        );
    }

    /// NaN confidence must not cause a panic in GRI 3a's max_by comparator.
    #[test]
    fn gri3a_nan_confidence_does_not_panic() {
        let preds = vec![
            (Some(50.0), make_pred("290511", f32::NAN)),
            (Some(50.0), make_pred("290512", 0.80)),
        ];
        // Should not panic; result may be either code depending on ordering
        let _ = try_gri3a(&preds);
    }
}
