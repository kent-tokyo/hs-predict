//! Chapter 38 — Miscellaneous chemical products.
//!
//! Chapter 38 covers **prepared** and **mixed** chemical products that do not
//! fall neatly into Chapters 28 or 29.  Unlike Chapters 28/29, classification
//! here depends heavily on **intended use** and **presentation** rather than
//! chemical structure alone.
//!
//! ## Key headings
//! | Heading | Description |
//! |---------|-------------|
//! | 38.02 | Activated carbon; activated natural mineral products |
//! | 38.09 | Finishing agents, dye carriers for textiles, etc. |
//! | 38.11 | Anti-knock / anti-oxidant preparations for mineral oils |
//! | 38.12 | Prepared rubber/plastic stabilizers |
//! | 38.17 | Mixed alkylbenzenes / alkylnaphthalenes |
//! | 38.20 | Anti-freezing preparations and de-icing fluids |
//! | 38.22 | Diagnostic / laboratory reagents |
//! | 38.23 | Industrial fatty acids, acid oils, industrial fatty alcohols |
//! | 38.24 | Chemical preparations NEC (catch-all) |
//!
//! ## Classification logic
//! Use [`classify_by_intended_use`] for mixtures whose chapter depends primarily
//! on end-use.  Use [`CHAPTER38_CATCH_ALL_CODE`] when no other rule applies.

use crate::types::IntendedUse;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Six-digit HS 2022 catch-all code for chemical preparations NEC (not elsewhere
/// classified).  Used as the final GRI 3c fallback for mixtures.
pub(crate) const CHAPTER38_CATCH_ALL_CODE: &str = "382499";

/// Heading description paired with [`CHAPTER38_CATCH_ALL_CODE`].
pub(crate) const CHAPTER38_CATCH_ALL_DESC: &str =
    "Chemical preparations, not elsewhere specified or included (Ch. 38 NEC)";

// ─────────────────────────────────────────────────────────────────────────────
// Intended-use based classification
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `(hs_code, heading_description, confidence)` for a mixture product
/// whose HS chapter is determined by its intended use.
///
/// Returns `None` when the intended use does not uniquely determine a Chapter 38
/// heading (e.g. `Industrial` is too broad).
///
/// Called by the mixture classifier **before** GRI 3a/3b/3c evaluation.
pub(crate) fn classify_by_intended_use(
    intended_use: &IntendedUse,
) -> Option<(&'static str, &'static str, f32)> {
    match intended_use {
        // Agricultural pesticide preparations → 38.08
        IntendedUse::Agricultural => Some((
            "380800",
            "Insecticides, rodenticides, fungicides, herbicides, \
             anti-sprouting products and plant-growth regulators, \
             disinfectants and similar products (Ch. 38.08)",
            0.75,
        )),

        // Pharmaceutical formulations → Ch. 30
        // (not Ch.38, handled separately by the pipeline)
        IntendedUse::Pharmaceutical => None,

        // Cosmetic formulations → Ch. 33
        // (not Ch.38, handled separately by the pipeline)
        IntendedUse::Cosmetic => None,

        // Food-grade preparations → Ch. 21
        // (not Ch.38, handled separately by the pipeline)
        IntendedUse::Food => None,

        // General industrial use: too broad to determine chapter automatically.
        IntendedUse::Industrial | IntendedUse::Other(_) => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Use-case-to-chapter mapping for non-chemical chapters
// ─────────────────────────────────────────────────────────────────────────────

/// For special-use products that must be classified outside Ch.28/29/38,
/// returns the chapter they belong to.
///
/// Returns `None` when standard pipeline logic applies.
pub(crate) fn special_chapter_by_use(
    intended_use: &IntendedUse,
) -> Option<(&'static str, &'static str, f32)> {
    match intended_use {
        IntendedUse::Pharmaceutical => Some((
            "300490",
            "Medicaments — other mixtures/preparations for therapeutic use (Ch. 30)",
            0.70,
        )),
        IntendedUse::Cosmetic => Some((
            "330499",
            "Beauty/cosmetic preparations — other (Ch. 33)",
            0.65,
        )),
        IntendedUse::Food => Some((
            "210690",
            "Food preparations, not elsewhere specified (Ch. 21)",
            0.65,
        )),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agricultural_gives_3808() {
        let (code, _, confidence) =
            classify_by_intended_use(&IntendedUse::Agricultural).unwrap();
        assert_eq!(&code[..2], "38");
        assert!(confidence > 0.5);
    }

    #[test]
    fn pharmaceutical_returns_none_from_ch38_function() {
        // Pharmaceuticals are handled by special_chapter_by_use, not classify_by_intended_use
        assert!(classify_by_intended_use(&IntendedUse::Pharmaceutical).is_none());
    }

    #[test]
    fn pharmaceutical_gives_ch30_via_special() {
        let (code, _, _) = special_chapter_by_use(&IntendedUse::Pharmaceutical).unwrap();
        assert_eq!(&code[..2], "30");
    }

    #[test]
    fn cosmetic_gives_ch33_via_special() {
        let (code, _, _) = special_chapter_by_use(&IntendedUse::Cosmetic).unwrap();
        assert_eq!(&code[..2], "33");
    }

    #[test]
    fn food_gives_ch21_via_special() {
        let (code, _, _) = special_chapter_by_use(&IntendedUse::Food).unwrap();
        assert_eq!(&code[..2], "21");
    }

    #[test]
    fn industrial_returns_none() {
        assert!(classify_by_intended_use(&IntendedUse::Industrial).is_none());
    }

    #[test]
    fn catch_all_is_6_digits() {
        assert_eq!(CHAPTER38_CATCH_ALL_CODE.len(), 6);
        assert!(CHAPTER38_CATCH_ALL_CODE.chars().all(|c| c.is_ascii_digit()));
    }
}
