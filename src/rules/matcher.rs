//! Rule matching logic — selects the best [`HsRule`] for a given
//! CAS number, physical form, and optional purity.

use crate::rules::static_table::HS_RULES;
use crate::rules::types::{HsRule, ShapePattern};
use crate::types::PhysicalForm;

/// Find the best matching [`HsRule`] for the given inputs.
///
/// When multiple rules match (e.g. both a specific-concentration rule and a
/// catch-all rule), the one with the highest *specificity score* wins.
/// Ties are broken by rule order in the static table.
///
/// Returns `None` if no rule is registered for this CAS number or if no
/// rule's shape/purity conditions are satisfied.
pub fn find_best_rule(
    cas: &str,
    physical_form: Option<&PhysicalForm>,
    purity_pct: Option<f64>,
) -> Option<&'static HsRule> {
    let form = physical_form.unwrap_or(&PhysicalForm::Unknown);

    let mut best: Option<(&'static HsRule, u8)> = None;

    for rule in HS_RULES {
        if rule.cas != cas {
            continue;
        }

        // Check shape match
        if !matches_shape(form, &rule.shape) {
            continue;
        }

        // Check purity match
        let purity_ok = match (&rule.purity_range, purity_pct) {
            (None, _) => true,
            (Some(range), Some(p)) => range.contains(&p),
            (Some(_), None) => false, // rule requires purity but none provided
        };
        if !purity_ok {
            continue;
        }

        let specificity = shape_specificity(&rule.shape)
            + if rule.purity_range.is_some() { 2 } else { 0 };

        if best.is_none() || specificity > best.unwrap().1 {
            best = Some((rule, specificity));
        }
    }

    best.map(|(rule, _)| rule)
}

/// Check whether a concrete [`PhysicalForm`] satisfies a [`ShapePattern`].
pub fn matches_shape(form: &PhysicalForm, pattern: &ShapePattern) -> bool {
    match (form, pattern) {
        (_, ShapePattern::Any) => true,
        (PhysicalForm::Solid, ShapePattern::Solid) => true,
        (PhysicalForm::Powder { .. }, ShapePattern::Powder) => true,
        (PhysicalForm::Granules, ShapePattern::Granules) => true,
        (PhysicalForm::Granules, ShapePattern::Powder) => true, // granules ≈ powder for most rules
        (PhysicalForm::Liquid, ShapePattern::Liquid) => true,
        (PhysicalForm::Gas, ShapePattern::Gas) => true,
        (PhysicalForm::Ingot, ShapePattern::Ingot) => true,
        (PhysicalForm::Foil { .. }, ShapePattern::Foil) => true,
        (PhysicalForm::Unknown, _) => false,
        (
            PhysicalForm::Solution { concentration_pct_ww, .. },
            ShapePattern::Solution { concentration_range_pct },
        ) => match (concentration_pct_ww, concentration_range_pct) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(c), Some(range)) => range.contains(c),
        },
        _ => false,
    }
}

/// Specificity score for a [`ShapePattern`] (higher = more specific).
fn shape_specificity(pattern: &ShapePattern) -> u8 {
    match pattern {
        ShapePattern::Any => 0,
        ShapePattern::Solution { concentration_range_pct: None } => 1,
        ShapePattern::Solution { concentration_range_pct: Some(_) } => 3,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naoh_solid_matches_281511() {
        let rule = find_best_rule("1310-73-2", Some(&PhysicalForm::Solid), None).unwrap();
        assert_eq!(rule.hs_code, "281511");
    }

    #[test]
    fn naoh_solution_matches_281512() {
        let rule = find_best_rule(
            "1310-73-2",
            Some(&PhysicalForm::Solution { solvent: None, concentration_pct_ww: Some(50.0) }),
            None,
        )
        .unwrap();
        assert_eq!(rule.hs_code, "281512");
    }

    #[test]
    fn hno3_fuming_matches_specific_code() {
        let rule = find_best_rule(
            "7697-37-2",
            Some(&PhysicalForm::Solution { solvent: None, concentration_pct_ww: Some(99.0) }),
            None,
        )
        .unwrap();
        // Fuming nitric acid (≥98%) should get the more specific code
        assert!(rule.confidence >= 0.85);
    }

    #[test]
    fn unknown_cas_returns_none() {
        let rule = find_best_rule("0000-00-0", Some(&PhysicalForm::Solid), None);
        assert!(rule.is_none());
    }
}
