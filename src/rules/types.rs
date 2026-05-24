//! Types for the static HS rule table.

use std::ops::RangeInclusive;

/// Physical shape pattern used as a condition in an [`HsRule`].
#[derive(Debug)]
pub enum ShapePattern {
    /// Matches any physical form.
    Any,
    /// Solid (bulk, pellets, flakes, rods, etc.).
    Solid,
    /// Powder (fine-grained).
    Powder,
    /// Granules (coarser than powder).
    Granules,
    /// Pure liquid (not a solution).
    Liquid,
    /// Solution, optionally constrained to a concentration range (w/w%).
    Solution {
        /// `None` = no concentration constraint.
        concentration_range_pct: Option<RangeInclusive<f64>>,
    },
    /// Gas or vapour.
    Gas,
    /// Metal foil.
    Foil,
    /// Cast metal (ingot, billet, slab, etc.).
    Ingot,
}

/// A single HS classification rule for a known CAS number.
///
/// Rules are stored in a `&'static [HsRule]` slice embedded at compile time,
/// so zero heap allocation is needed at runtime.
#[derive(Debug)]
pub struct HsRule {
    /// CAS registry number (e.g. `"1310-73-2"`).
    pub cas: &'static str,

    /// Required physical form for this rule to match.
    pub shape: ShapePattern,

    /// Optional purity constraint (w/w%). `None` = no constraint.
    pub purity_range: Option<RangeInclusive<f64>>,

    /// Six-digit HS 2022 code (no dots, e.g. `"281511"`).
    pub hs_code: &'static str,

    /// Human-readable description of the HS heading.
    pub heading_description: &'static str,

    /// Confidence score (0.0–1.0) for this rule.
    pub confidence: f32,
}
