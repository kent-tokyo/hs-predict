//! SMILES-based functional group detection and chapter-level HS classification.
//!
//! This module provides a pattern-matching engine that inspects a canonical
//! SMILES string and infers:
//!
//! 1. **Organic vs. inorganic** classification
//! 2. **Functional groups** present (up to 20 categories)
//! 3. **HS chapter / heading hint** (approximate, confidence ≤ 0.70)
//!
//! The engine is used as Priority 3 in [`HsPipeline::classify`] when the CAS
//! rule table (Priority 2) finds no match but a SMILES string is available.
//!
//! [`HsPipeline::classify`]: crate::pipeline::HsPipeline::classify
//!
//! # Example
//! ```rust
//! use hs_predict::smiles::classify_smiles;
//! use hs_predict::smiles::detector::FunctionalGroup;
//!
//! let result = classify_smiles("CC(C)=O").unwrap(); // acetone
//! assert_eq!(result.heading_hint.heading, Some(2914)); // ketone → 29.14
//! ```

pub mod chapter_map;
pub mod detector;

pub use chapter_map::HeadingHint;
pub use detector::{FunctionalGroup, StructuralFeatures};

use crate::types::OrganicInorganic;

// ─────────────────────────────────────────────────────────────────────────────
// SmilesClassification
// ─────────────────────────────────────────────────────────────────────────────

/// Result of SMILES-based functional group analysis and HS heading estimation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SmilesClassification {
    /// Whether the compound is organic, inorganic, or organometallic.
    pub organic_class: OrganicInorganic,

    /// Functional groups detected in the SMILES string.
    /// May be empty for simple hydrocarbons (alkanes, alkenes, etc.).
    pub functional_groups: Vec<FunctionalGroup>,

    /// Structural atom-count and connectivity properties.
    pub structural_features: StructuralFeatures,

    /// Best-guess HS chapter / heading (and 6-digit subheading when
    /// determinable) based on detected groups and structural features.
    pub heading_hint: HeadingHint,
}

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Analyse a SMILES string and return a chapter-level HS classification hint.
///
/// # Returns
/// - `Some(SmilesClassification)` — analysis result; use
///   [`SmilesClassification::heading_hint`] for the HS heading.
/// - `None` — the SMILES string is empty or whitespace-only.
///
/// # Notes
/// - Detection is based on substring matching against canonical SMILES
///   (as produced by PubChem). Non-canonical or hand-written SMILES may
///   yield reduced accuracy.
/// - Results carry confidence ≤ 0.70; always verify with a trade-compliance
///   expert before using in a customs declaration.
///
/// # Example
/// ```rust
/// use hs_predict::smiles::classify_smiles;
///
/// // Benzaldehyde → aldehyde → 29.12
/// let r = classify_smiles("O=Cc1ccccc1").unwrap();
/// assert_eq!(r.heading_hint.heading, Some(2912));
///
/// // Acetic acid → carboxylic acid → 29.15
/// let r = classify_smiles("CC(=O)O").unwrap();
/// assert_eq!(r.heading_hint.heading, Some(2915));
/// ```
/// Maximum accepted SMILES string length (bytes).
///
/// SMILES strings for real-world compounds are at most a few thousand
/// characters.  This limit prevents algorithmic-complexity denial of service
/// from excessively long inputs.
pub const MAX_SMILES_LEN: usize = 4096;

pub fn classify_smiles(smiles: &str) -> Option<SmilesClassification> {
    let smiles = smiles.trim();
    if smiles.is_empty() || smiles.len() > MAX_SMILES_LEN {
        return None;
    }

    let organic_class = detector::classify_organic(smiles);
    let functional_groups = detector::detect_functional_groups(smiles);
    let structural_features = detector::detect_structural_features(smiles);
    let heading_hint = chapter_map::map_to_subheading(
        &functional_groups,
        &organic_class,
        &structural_features,
    );

    Some(SmilesClassification {
        organic_class,
        functional_groups,
        structural_features,
        heading_hint,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_smiles_returns_none() {
        assert!(classify_smiles("").is_none());
        assert!(classify_smiles("   ").is_none());
    }

    #[test]
    fn acetone_ketone_heading() {
        // CC(C)=O — acetone (PubChem canonical)
        let r = classify_smiles("CC(C)=O").unwrap();
        assert_eq!(r.heading_hint.heading, Some(2914));
        assert!(r.functional_groups.contains(&FunctionalGroup::Ketone));
        assert!(matches!(r.organic_class, OrganicInorganic::Organic));
    }

    #[test]
    fn acetic_acid_heading() {
        // CC(=O)O — acetic acid
        let r = classify_smiles("CC(=O)O").unwrap();
        assert_eq!(r.heading_hint.heading, Some(2915));
        assert!(r.functional_groups.contains(&FunctionalGroup::CarboxylicAcid));
    }

    #[test]
    fn ethyl_acetate_heading() {
        // CCOC(C)=O — ethyl acetate
        let r = classify_smiles("CCOC(C)=O").unwrap();
        assert_eq!(r.heading_hint.heading, Some(2915));
        assert!(r.functional_groups.contains(&FunctionalGroup::Ester));
    }

    #[test]
    fn benzaldehyde_heading() {
        // O=Cc1ccccc1 — benzaldehyde
        let r = classify_smiles("O=Cc1ccccc1").unwrap();
        assert_eq!(r.heading_hint.heading, Some(2912));
        assert!(r.functional_groups.contains(&FunctionalGroup::Aldehyde));
    }

    #[test]
    fn ethanol_heading() {
        // CCO — ethanol: structural engine routes to HS 22.07 (ethyl alcohol),
        // not 29.05.  This is the correct WCO classification.
        let r = classify_smiles("CCO").unwrap();
        assert_eq!(r.heading_hint.chapter, 22);
        assert_eq!(r.heading_hint.heading, Some(2207));
        assert_eq!(r.heading_hint.subheading.as_deref(), Some("220710"));
        assert!(r.functional_groups.contains(&FunctionalGroup::Alcohol));
    }

    #[test]
    fn methylamine_heading() {
        // CN — methylamine
        let r = classify_smiles("CN").unwrap();
        assert_eq!(r.heading_hint.heading, Some(2921));
    }

    #[test]
    fn chlorobenzene_heading() {
        // Clc1ccccc1 — chlorobenzene
        let r = classify_smiles("Clc1ccccc1").unwrap();
        assert_eq!(r.heading_hint.heading, Some(2903));
        assert!(r.functional_groups.contains(&FunctionalGroup::Halide));
    }

    #[test]
    fn co2_is_inorganic_ch28() {
        let r = classify_smiles("O=C=O").unwrap();
        assert_eq!(r.heading_hint.chapter, 28);
        assert!(matches!(r.organic_class, OrganicInorganic::Inorganic));
    }

    #[test]
    fn epoxide_heading() {
        // C1CO1 — ethylene oxide
        let r = classify_smiles("C1CO1").unwrap();
        assert_eq!(r.heading_hint.heading, Some(2910));
    }

    #[test]
    fn phthalic_anhydride_heading() {
        // O=C1OC(=O)c2ccccc21
        let r = classify_smiles("O=C1OC(=O)c2ccccc21").unwrap();
        assert!(r.functional_groups.contains(&FunctionalGroup::Anhydride));
        assert_eq!(r.heading_hint.heading, Some(2915));
    }
}
