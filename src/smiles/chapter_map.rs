//! Maps detected functional groups to HS chapter / heading hints.
//!
//! The mapping is intentionally approximate; predictions from this module
//! carry confidence ≤ 0.70 and are tagged with
//! [`PredictionSource::RuleEngine`](crate::types::PredictionSource::RuleEngine).
//!
//! # Priority order
//! More specific functional groups take precedence (e.g. anhydride >
//! carboxylic acid > alcohol). The first matching rule wins.
//!
//! # HS structure used
//! - Chapter 28 — inorganic chemicals (when `organic_class` is `Inorganic`)
//! - Chapter 29 — organic chemicals (sub-headings by functional group)
//! - Chapter 38 — misc. chemical preparations (default organic fallback)

use crate::smiles::detector::FunctionalGroup;
use crate::types::OrganicInorganic;

// ─────────────────────────────────────────────────────────────────────────────
// HeadingHint
// ─────────────────────────────────────────────────────────────────────────────

/// HS chapter / heading hint derived from SMILES functional group analysis.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HeadingHint {
    /// HS chapter number (e.g. `28`, `29`).
    pub chapter: u8,

    /// Four-digit HS heading (e.g. `2912` for aldehydes).
    /// `None` when only the chapter can be determined.
    pub heading: Option<u16>,

    /// Human-readable rationale for the hint.
    pub rationale: &'static str,

    /// Confidence in [0.0, 1.0].
    /// Capped at 0.70 because SMILES pattern matching is approximate.
    pub confidence: f32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Mapping table (priority-ordered)
// ─────────────────────────────────────────────────────────────────────────────

/// Priority-ordered mapping: (FunctionalGroup, chapter, heading, rationale, confidence).
///
/// The first entry whose group is present in the detected set wins.
/// Groups higher in the list are more specific (e.g. anhydride before acid).
static PRIORITY_MAP: &[(FunctionalGroup, u8, u16, &str, f32)] = &[
    // ── High-specificity groups ─────────────────────────────────────────
    (
        FunctionalGroup::Anhydride,
        29, 2915,
        "Acid anhydride → HS 29.15–29.17 (acyclic/aromatic acid anhydrides); \
         use 29.17 for aromatic anhydrides",
        0.65,
    ),
    (
        FunctionalGroup::Isocyanate,
        29, 2929,
        "Isocyanate / carbodiimide → HS 29.29",
        0.70,
    ),
    (
        FunctionalGroup::Epoxide,
        29, 2910,
        "Epoxide → HS 29.10",
        0.70,
    ),
    (
        FunctionalGroup::SulphonicAcid,
        29, 2904,
        "Organo-sulphonic acid → HS 29.04 (sulphonated derivatives)",
        0.68,
    ),
    (
        FunctionalGroup::Nitrile,
        29, 2926,
        "Nitrile → HS 29.26",
        0.70,
    ),
    (
        FunctionalGroup::Phosphate,
        29, 2920,
        "Organophosphate / phosphonate ester → HS 29.20",
        0.62,
    ),
    // ── Carbonyl groups ─────────────────────────────────────────────────
    (
        FunctionalGroup::Amide,
        29, 2924,
        "Amide → HS 29.24 (amide-function compounds)",
        0.67,
    ),
    (
        FunctionalGroup::CarboxylicAcid,
        29, 2915,
        "Carboxylic acid → HS 29.15 (acyclic), 29.16 (cyclic), 29.17 (aromatic), \
         or 29.18 (other with additional functions); heading depends on chain length / ring",
        0.60,
    ),
    (
        FunctionalGroup::Ester,
        29, 2915,
        "Ester → HS 29.15–29.17 (depends on parent acid type and chain length)",
        0.55,
    ),
    (
        FunctionalGroup::Aldehyde,
        29, 2912,
        "Aldehyde → HS 29.12",
        0.67,
    ),
    (
        FunctionalGroup::Ketone,
        29, 2914,
        "Ketone / quinone → HS 29.14",
        0.67,
    ),
    // ── OH groups ───────────────────────────────────────────────────────
    (
        FunctionalGroup::Phenol,
        29, 2907,
        "Phenol → HS 29.07",
        0.67,
    ),
    (
        FunctionalGroup::Alcohol,
        29, 2905,
        "Alcohol → HS 29.05 (acyclic) or 29.06 (cyclic); \
         polyols may fall under 29.05 subheading",
        0.60,
    ),
    // ── Organo-sulphur ──────────────────────────────────────────────────
    (
        FunctionalGroup::Thiol,
        29, 2930,
        "Thiol (mercaptan) → HS 29.30 (organo-sulphur compounds)",
        0.65,
    ),
    (
        FunctionalGroup::Sulphide,
        29, 2930,
        "Thioether / sulphide → HS 29.30 (organo-sulphur compounds)",
        0.65,
    ),
    // ── N-function, O-function, halide ────────────────────────────────
    (
        FunctionalGroup::Amine,
        29, 2921,
        "Amine → HS 29.21",
        0.63,
    ),
    (
        FunctionalGroup::Nitro,
        29, 2904,
        "Nitro / nitroso compound → HS 29.04",
        0.60,
    ),
    (
        FunctionalGroup::Ether,
        29, 2909,
        "Ether → HS 29.09",
        0.63,
    ),
    (
        FunctionalGroup::Halide,
        29, 2903,
        "Organohalide → HS 29.03",
        0.65,
    ),
    // ── Aromatic (lowest organic priority) ──────────────────────────────
    (
        FunctionalGroup::AromaticRing,
        29, 0,    // heading unknown — only chapter hint
        "Aromatic compound → Chapter 29; heading depends on substituents",
        0.40,
    ),
];

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Derive an HS chapter / heading hint from functional group analysis.
///
/// # Arguments
/// - `groups` — functional groups detected by [`detect_functional_groups`](crate::smiles::detector::detect_functional_groups).
/// - `organic_class` — result of [`classify_organic`](crate::smiles::detector::classify_organic).
///
/// # Returns
/// The first matching entry in the priority table, or a Chapter-28/29 generic
/// fallback if no specific match is found.
pub fn map_to_heading(
    groups: &[FunctionalGroup],
    organic_class: &OrganicInorganic,
) -> HeadingHint {
    // ── Inorganic branch ─────────────────────────────────────────────────
    if matches!(organic_class, OrganicInorganic::Inorganic) {
        return HeadingHint {
            chapter: 28,
            heading: None,
            rationale: "Inorganic compound → Chapter 28; \
                        heading depends on element / salt type",
            confidence: 0.55,
        };
    }

    // ── Organometallic branch ────────────────────────────────────────────
    if matches!(organic_class, OrganicInorganic::Organometallic) {
        return HeadingHint {
            chapter: 29,
            heading: Some(2931),
            rationale: "Organometallic compound → HS 29.31",
            confidence: 0.62,
        };
    }

    // ── Organic: use priority table ──────────────────────────────────────
    for &(group, chapter, heading_code, rationale, confidence) in PRIORITY_MAP {
        if groups.contains(&group) {
            let heading = if heading_code == 0 { None } else { Some(heading_code) };
            return HeadingHint { chapter, heading, rationale, confidence };
        }
    }

    // ── Fallback: generic organic (Chapter 38 or unclassified Ch.29) ─────
    HeadingHint {
        chapter: 29,
        heading: None,
        rationale: "Organic compound with no detected functional groups → \
                    Chapter 29 (unsubstituted hydrocarbon) or Chapter 38",
        confidence: 0.35,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn hint(groups: &[FunctionalGroup]) -> HeadingHint {
        map_to_heading(groups, &OrganicInorganic::Organic)
    }

    #[test]
    fn inorganic_gives_ch28() {
        let h = map_to_heading(&[], &OrganicInorganic::Inorganic);
        assert_eq!(h.chapter, 28);
        assert!(h.heading.is_none());
    }

    #[test]
    fn organometallic_gives_2931() {
        let h = map_to_heading(&[], &OrganicInorganic::Organometallic);
        assert_eq!(h.heading, Some(2931));
    }

    #[test]
    fn anhydride_wins_over_acid() {
        let h = hint(&[FunctionalGroup::Anhydride, FunctionalGroup::CarboxylicAcid]);
        // Anhydride is higher priority → heading 2915, not a different one
        assert_eq!(h.heading, Some(2915));
        assert!(h.rationale.to_lowercase().contains("anhydride"));
    }

    #[test]
    fn aldehyde_maps_to_2912() {
        let h = hint(&[FunctionalGroup::Aldehyde]);
        assert_eq!(h.heading, Some(2912));
    }

    #[test]
    fn ketone_maps_to_2914() {
        let h = hint(&[FunctionalGroup::Ketone]);
        assert_eq!(h.heading, Some(2914));
    }

    #[test]
    fn alcohol_maps_to_2905() {
        let h = hint(&[FunctionalGroup::Alcohol]);
        assert_eq!(h.heading, Some(2905));
    }

    #[test]
    fn nitrile_maps_to_2926() {
        let h = hint(&[FunctionalGroup::Nitrile]);
        assert_eq!(h.heading, Some(2926));
    }

    #[test]
    fn amine_maps_to_2921() {
        let h = hint(&[FunctionalGroup::Amine]);
        assert_eq!(h.heading, Some(2921));
    }

    #[test]
    fn halide_maps_to_2903() {
        let h = hint(&[FunctionalGroup::Halide]);
        assert_eq!(h.heading, Some(2903));
    }

    #[test]
    fn no_groups_gives_low_confidence() {
        let h = hint(&[]);
        assert!(h.confidence < 0.50);
    }

    #[test]
    fn isocyanate_maps_to_2929() {
        let h = hint(&[FunctionalGroup::Isocyanate]);
        assert_eq!(h.heading, Some(2929));
    }

    #[test]
    fn epoxide_maps_to_2910() {
        let h = hint(&[FunctionalGroup::Epoxide]);
        assert_eq!(h.heading, Some(2910));
    }
}
