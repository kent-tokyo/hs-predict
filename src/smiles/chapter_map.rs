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

use crate::smiles::detector::{FunctionalGroup, StructuralFeatures};
use crate::types::OrganicInorganic;

// ─────────────────────────────────────────────────────────────────────────────
// HeadingHint
// ─────────────────────────────────────────────────────────────────────────────

/// HS chapter / heading hint derived from SMILES functional group analysis.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HeadingHint {
    /// HS chapter number (e.g. `28`, `29`).
    pub chapter: u8,

    /// Four-digit HS heading (e.g. `2914` for ketones).
    /// `None` when only the chapter can be determined.
    pub heading: Option<u16>,

    /// Six-digit HS subheading when structural features allow it
    /// (e.g. `"291411"` for acetone).  `None` when only the 4-digit
    /// heading can be determined.
    pub subheading: Option<String>,

    /// Human-readable rationale for the hint.
    pub rationale: &'static str,

    /// Confidence in [0.0, 1.0].
    /// Capped at 0.70 for heading-only results; up to 0.90 when a
    /// specific 6-digit subheading is identified.
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
            subheading: None,
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
            subheading: None,
            rationale: "Organometallic compound → HS 29.31",
            confidence: 0.62,
        };
    }

    // ── Organic: use priority table ──────────────────────────────────────
    for &(group, chapter, heading_code, rationale, confidence) in PRIORITY_MAP {
        if groups.contains(&group) {
            let heading = if heading_code == 0 { None } else { Some(heading_code) };
            return HeadingHint { chapter, heading, subheading: None, rationale, confidence };
        }
    }

    // ── Fallback: generic organic (Chapter 38 or unclassified Ch.29) ─────
    HeadingHint {
        chapter: 29,
        heading: None,
        subheading: None,
        rationale: "Organic compound with no detected functional groups → \
                    Chapter 29 (unsubstituted hydrocarbon) or Chapter 38",
        confidence: 0.35,
    }
}

/// Derive a 6-digit HS subheading when structural features permit.
///
/// Combines functional-group detection with atom counts and ring/bond
/// information.  Covers the three most common Chapter 29 groups in
/// chemical trade: **ketones** (29.14), **alcohols** (29.05 / 22.07),
/// and **carboxylic acids** (29.15 / 29.16).
///
/// When no 6-digit code can be determined, falls back to
/// [`map_to_heading`] (returns `subheading: None`).
pub fn map_to_subheading(
    groups: &[FunctionalGroup],
    organic_class: &OrganicInorganic,
    feat: &StructuralFeatures,
) -> HeadingHint {
    // Only applies to pure organic compounds.
    if !matches!(organic_class, OrganicInorganic::Organic) {
        return map_to_heading(groups, organic_class);
    }

    // ── Ketones (HS 29.14) ────────────────────────────────────────────────
    if groups.contains(&FunctionalGroup::Ketone) {
        return subheading_ketone(feat);
    }

    // ── Alcohols (HS 29.05 / 22.07) ──────────────────────────────────────
    if groups.contains(&FunctionalGroup::Alcohol) {
        return subheading_alcohol(feat);
    }

    // ── Carboxylic acids (HS 29.15 / 29.16) ──────────────────────────────
    if groups.contains(&FunctionalGroup::CarboxylicAcid) {
        return subheading_acid(feat);
    }

    // ── Aldehydes (HS 29.12) ──────────────────────────────────────────────
    if groups.contains(&FunctionalGroup::Aldehyde) {
        return subheading_aldehyde(feat);
    }

    // No specific subheading logic — fall back to heading-only.
    map_to_heading(groups, organic_class)
}

// ─────────────────────────────────────────────────────────────────────────────
// Subheading decision trees
// ─────────────────────────────────────────────────────────────────────────────

/// HS 29.14 — ketones and quinones.
fn subheading_ketone(f: &StructuralFeatures) -> HeadingHint {
    let (code, rationale, conf) = if f.has_aromatic_ring {
        // Aromatic ketones
        if f.carbon_count == 8 && f.carbonyl_count == 1 {
            ("291431", "Phenyl methyl ketone (acetophenone) → HS 29.14.31", 0.82_f32)
        } else {
            ("291439", "Other aromatic ketone → HS 29.14.39", 0.65)
        }
    } else if f.has_ring {
        // Cycloaliphatic ketones
        match f.carbon_count {
            10 => ("291421", "Camphor (cyclic C10 ketone) → HS 29.14.21", 0.78),
            6  => ("291422", "Cyclohexanone → HS 29.14.22", 0.85),
            7  => ("291423", "Methylcyclohexanone → HS 29.14.23", 0.78),
            _  => ("291429", "Other cycloaliphatic/cycloterpenic ketone → HS 29.14.29", 0.65),
        }
    } else {
        // Acyclic ketones
        if f.has_halogen {
            ("291479", "Halogenated ketone derivative → HS 29.14.79", 0.68)
        } else if f.hydroxyl_count > 0 {
            ("291440", "Ketone-alcohol or ketone-aldehyde → HS 29.14.40", 0.70)
        } else {
            match f.carbon_count {
                3 => ("291411", "Acetone (3C acyclic ketone) → HS 29.14.11", 0.87),
                4 => ("291412", "Butanone / MEK (4C acyclic ketone) → HS 29.14.12", 0.83),
                6 => ("291413",
                      "4-Methylpentan-2-one / MIBK candidate (6C acyclic ketone) → HS 29.14.13; \
                       verify branching pattern",
                      0.72),
                _ => ("291419", "Other acyclic ketone without other O → HS 29.14.19", 0.68),
            }
        }
    };
    HeadingHint {
        chapter: 29,
        heading: Some(2914),
        subheading: Some(code.to_string()),
        rationale,
        confidence: conf,
    }
}

/// HS 29.05 — acyclic alcohols; 22.07 — ethanol special case.
fn subheading_alcohol(f: &StructuralFeatures) -> HeadingHint {
    let oh = f.hydroxyl_count.max(1); // guard zero-count edge case

    let (code, chapter, heading, rationale, conf): (&str, u8, u16, &'static str, f32) =
        if oh >= 3 {
            match f.carbon_count {
                3 => ("290541", 29, 2905, "Glycerol (3C triol) → HS 29.05.41", 0.90),
                _ => ("290549", 29, 2905, "Other polyol → HS 29.05.49", 0.65),
            }
        } else if oh == 2 {
            match f.carbon_count {
                2 => ("290531", 29, 2905, "Ethylene glycol (2C diol) → HS 29.05.31", 0.88),
                3 => ("290532", 29, 2905, "Propylene glycol (3C diol) → HS 29.05.32", 0.85),
                _ => ("290539", 29, 2905, "Other diol → HS 29.05.39", 0.68),
            }
        } else {
            // Monohydric alcohol
            if f.has_cc_double_bond {
                ("290529", 29, 2905,
                 "Unsaturated monohydric acyclic alcohol → HS 29.05.29", 0.65)
            } else {
                match f.carbon_count {
                    1 => ("290511", 29, 2905,
                          "Methanol (1C) → HS 29.05.11", 0.90),
                    2 => ("220710", 22, 2207,
                          "Ethanol (2C) → HS 22.07.10 (undenatured ethyl alcohol ≥ 80 %); \
                           verify concentration — denatured → 22.07.20, dilute → 22.08",
                          0.85),
                    3 => ("290512", 29, 2905,
                          "Propan-1-ol (3C saturated monohydric) → HS 29.05.12", 0.82),
                    4 => ("290513", 29, 2905,
                          "Butan-1-ol (4C primary alcohol) → HS 29.05.13; \
                           other butanols → 29.05.14",
                          0.75),
                    8 => ("290516", 29, 2905,
                          "Octanol and isomers → HS 29.05.16", 0.78),
                    12 | 16 | 18 => ("290517", 29, 2905,
                                     "Dodecan-1-ol / hexadecan-1-ol / octadecan-1-ol \
                                      → HS 29.05.17",
                                     0.75),
                    _ => ("290519", 29, 2905,
                          "Other saturated monohydric acyclic alcohol → HS 29.05.19", 0.65),
                }
            }
        };

    HeadingHint {
        chapter,
        heading: Some(heading),
        subheading: Some(code.to_string()),
        rationale,
        confidence: conf,
    }
}

/// HS 29.15 (saturated aliphatic) / 29.16 (unsaturated or aromatic) acids.
fn subheading_acid(f: &StructuralFeatures) -> HeadingHint {
    let (code, heading, rationale, conf): (&str, u16, &'static str, f32) =
        if f.has_aromatic_ring {
            match f.carbon_count {
                7 => ("291631", 2916,
                      "Benzoic acid (7C aromatic acid) → HS 29.16.31", 0.85),
                8 => ("291634", 2916,
                      "Phenylacetic acid (8C aromatic acid) → HS 29.16.34", 0.78),
                _ => ("291639", 2916,
                      "Other aromatic monocarboxylic acid → HS 29.16.39", 0.65),
            }
        } else if f.has_cc_double_bond {
            // Unsaturated aliphatic → heading 2916
            match f.carbon_count {
                3 => ("291611", 2916,
                      "Acrylic acid (3C unsaturated) → HS 29.16.11", 0.87),
                4 => ("291613", 2916,
                      "Methacrylic acid (4C unsaturated, branch C=C) → HS 29.16.13; \
                       esters → 29.16.14",
                      0.82),
                _ => ("291619", 2916,
                      "Other unsaturated aliphatic monocarboxylic acid → HS 29.16.19", 0.65),
            }
        } else {
            // Saturated aliphatic → heading 2915
            // hydroxyl_count includes the acid –OH; extra OH means hydroxy-acid (2918)
            let extra_oh = f.hydroxyl_count.saturating_sub(1);
            if extra_oh >= 1 {
                ("291819", 2918,
                 "Carboxylic acid with additional oxygen function → HS 29.18.19", 0.65)
            } else {
                match f.carbon_count {
                    1  => ("291511", 2915, "Formic acid (1C) → HS 29.15.11", 0.90),
                    2  => ("291521", 2915, "Acetic acid (2C) → HS 29.15.21", 0.90),
                    3  => ("291550", 2915, "Propionic acid (3C) → HS 29.15.50", 0.87),
                    4  => ("291560", 2915,
                           "Butanoic / butyric acid (4C) → HS 29.15.60", 0.83),
                    16 | 18 => ("291570", 2915,
                                "Palmitic / stearic acid (C16/C18) → HS 29.15.70", 0.80),
                    _  => ("291590", 2915,
                           "Other saturated acyclic monocarboxylic acid → HS 29.15.90", 0.65),
                }
            }
        };

    HeadingHint {
        chapter: 29,
        heading: Some(heading),
        subheading: Some(code.to_string()),
        rationale,
        confidence: conf,
    }
}

/// HS 29.12 — aldehydes.
fn subheading_aldehyde(f: &StructuralFeatures) -> HeadingHint {
    let (code, rationale, conf): (&str, &'static str, f32) = if f.has_aromatic_ring {
        match f.carbon_count {
            7 => ("291211", "Benzaldehyde (7C aromatic aldehyde) → HS 29.12.11", 0.85),
            _ => ("291219", "Other aromatic aldehyde → HS 29.12.19", 0.65),
        }
    } else {
        match f.carbon_count {
            1 => ("291211", "Formaldehyde → HS 29.12.11", 0.82),
            2 => ("291212", "Acetaldehyde (2C) → HS 29.12.12", 0.85),
            3 => ("291219", "Propanal / acrolein candidate (3C) → HS 29.12.19", 0.72),
            _ => ("291219", "Other aliphatic aldehyde → HS 29.12.19", 0.65),
        }
    };
    HeadingHint {
        chapter: 29,
        heading: Some(2912),
        subheading: Some(code.to_string()),
        rationale,
        confidence: conf,
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

    // ── map_to_subheading ─────────────────────────────────────────────────

    fn feat(carbon: u32, oh: u32, co: u32, ring: bool, arom: bool, cc: bool, hal: bool)
        -> StructuralFeatures
    {
        StructuralFeatures {
            carbon_count: carbon,
            hydroxyl_count: oh,
            carbonyl_count: co,
            has_ring: ring,
            has_aromatic_ring: arom,
            has_cc_double_bond: cc,
            has_halogen: hal,
        }
    }

    #[test]
    fn acetone_subheading_291411() {
        let f = feat(3, 0, 1, false, false, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::Ketone], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("291411"));
        assert!(h.confidence >= 0.85);
    }

    #[test]
    fn mek_subheading_291412() {
        let f = feat(4, 0, 1, false, false, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::Ketone], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("291412"));
    }

    #[test]
    fn cyclohexanone_subheading_291422() {
        let f = feat(6, 0, 1, true, false, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::Ketone], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("291422"));
        assert!(h.confidence >= 0.80);
    }

    #[test]
    fn methanol_subheading_290511() {
        let f = feat(1, 1, 0, false, false, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::Alcohol], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("290511"));
    }

    #[test]
    fn ethanol_subheading_220710() {
        let f = feat(2, 1, 0, false, false, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::Alcohol], &OrganicInorganic::Organic, &f,
        );
        // Ethanol goes to Ch. 22, not Ch. 29
        assert_eq!(h.subheading.as_deref(), Some("220710"));
        assert_eq!(h.chapter, 22);
    }

    #[test]
    fn ethylene_glycol_subheading_290531() {
        let f = feat(2, 2, 0, false, false, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::Alcohol], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("290531"));
    }

    #[test]
    fn glycerol_subheading_290541() {
        let f = feat(3, 3, 0, false, false, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::Alcohol], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("290541"));
    }

    #[test]
    fn acetic_acid_subheading_291521() {
        let f = feat(2, 1, 1, false, false, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::CarboxylicAcid], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("291521"));
    }

    #[test]
    fn formic_acid_subheading_291511() {
        let f = feat(1, 1, 1, false, false, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::CarboxylicAcid], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("291511"));
    }

    #[test]
    fn acrylic_acid_subheading_291611() {
        let f = feat(3, 1, 1, false, false, true, false);
        let h = map_to_subheading(
            &[FunctionalGroup::CarboxylicAcid], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("291611"));
        assert_eq!(h.heading, Some(2916));
    }

    #[test]
    fn methacrylic_acid_subheading_291613() {
        let f = feat(4, 1, 1, false, false, true, false);
        let h = map_to_subheading(
            &[FunctionalGroup::CarboxylicAcid], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("291613"));
    }

    #[test]
    fn benzoic_acid_subheading_291631() {
        let f = feat(7, 1, 1, true, true, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::CarboxylicAcid], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("291631"));
    }

    #[test]
    fn benzaldehyde_subheading_291211() {
        let f = feat(7, 0, 1, true, true, false, false);
        let h = map_to_subheading(
            &[FunctionalGroup::Aldehyde], &OrganicInorganic::Organic, &f,
        );
        assert_eq!(h.subheading.as_deref(), Some("291211"));
    }

    #[test]
    fn inorganic_subheading_falls_back_to_heading_only() {
        let f = feat(0, 0, 0, false, false, false, false);
        let h = map_to_subheading(&[], &OrganicInorganic::Inorganic, &f);
        assert!(h.subheading.is_none());
        assert_eq!(h.chapter, 28);
    }
}
