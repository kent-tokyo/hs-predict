use serde::{Deserialize, Serialize};
use crate::types::{IntendedUse, OrganicInorganic, PhysicalForm, SubstanceIdentifier};

/// Classification state accumulated across session Q&A rounds.
///
/// Each field starts as `None` and is filled in as the user answers questions.
/// The pipeline reads this state (via [`to_product_description`](super::ClassificationSession::to_product_description))
/// when the session is complete.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassificationState {
    // ── Identifier ───────────────────────────────────────────────
    /// Identifier entered by the user in the first question.
    pub identifier: SubstanceIdentifier,

    // ── Mixture ──────────────────────────────────────────────────
    /// Whether the product is a mixture. `None` = not yet answered.
    pub is_mixture: Option<bool>,

    /// Number of components (set when `is_mixture` = true).
    pub component_count: Option<usize>,

    /// Components collected so far (built up one at a time).
    pub components: Vec<PartialComponent>,

    /// Index of the component currently being entered.
    pub current_component_index: usize,

    // ── Physical form ─────────────────────────────────────────────
    pub physical_form: Option<PhysicalForm>,

    // ── Purity ────────────────────────────────────────────────────
    pub purity_pct: Option<f64>,

    // ── Chemistry ─────────────────────────────────────────────────
    /// Organic / inorganic classification. `None` = not yet answered.
    pub organic_inorganic: Option<OrganicInorganic>,

    /// HS chapter hint narrowed down by the rule engine (two-digit string, e.g. `"28"`).
    pub chapter_hint: Option<String>,

    // ── Intended use ──────────────────────────────────────────────
    pub intended_use: Option<IntendedUse>,

    // ── Functional groups (organic compounds without SMILES) ──────
    /// Functional group keys selected by the user (e.g. `"carboxylic_acid"`).
    pub detected_functional_groups: Vec<String>,

    // ── Completion ────────────────────────────────────────────────
    /// Set to `true` when `next_question()` returns `None`.
    pub is_complete: bool,
}

/// Partially-filled mixture component (accumulated during session).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartialComponent {
    pub identifier: SubstanceIdentifier,
    /// Weight fraction in w/w%.
    pub weight_fraction_pct: Option<f64>,
    pub is_solvent: bool,
}

impl ClassificationState {
    /// Returns `true` if at least one identifier field has been set.
    pub fn has_identifier(&self) -> bool {
        !self.identifier.is_empty()
    }

    /// Rough confidence estimate based on how many fields are known.
    ///
    /// Used to decide whether to emit `SessionResult::RequiresLlm`
    /// when all questions have been answered.
    pub fn confidence_estimate(&self) -> f32 {
        let mut score: f32 = 0.0;

        // Identifier quality
        if self.identifier.cas.is_some() {
            score += 0.40; // CAS is the most reliable identifier
        } else if self.identifier.smiles.is_some() {
            score += 0.30;
        } else if self.identifier.iupac_name.is_some()
            || self.identifier.inchi.is_some()
            || self.identifier.inchi_key.is_some()
        {
            score += 0.25;
        }

        // Physical form known
        if self.physical_form.is_some() {
            score += 0.15;
        }

        // Organic / inorganic known
        if self.organic_inorganic.is_some() {
            score += 0.15;
        }

        // HS chapter narrowed down
        if self.chapter_hint.is_some() {
            score += 0.15;
        }

        // Intended use known — pharmaceutical / agricultural constrain the chapter strongly
        if matches!(
            self.intended_use,
            Some(IntendedUse::Pharmaceutical) | Some(IntendedUse::Agricultural)
        ) {
            score += 0.10;
        }

        // All mixture components collected
        if self.is_mixture == Some(true) {
            let expected = self.component_count.unwrap_or(0);
            let filled = self.components.iter().filter(|c| !c.identifier.is_empty()).count();
            if expected > 0 && filled >= expected {
                score += 0.05;
            }
        }

        score.min(1.0)
    }

    /// Returns `true` if all expected mixture components have been entered.
    ///
    /// Always returns `true` for non-mixture products.
    pub fn all_components_filled(&self) -> bool {
        if self.is_mixture != Some(true) {
            return true;
        }
        let expected = self.component_count.unwrap_or(0);
        if expected == 0 {
            return false;
        }
        self.components.len() >= expected
            && self.components.iter().all(|c| !c.identifier.is_empty())
    }

    /// Returns `true` if the current mixture component has an identifier.
    pub fn current_component_has_identifier(&self) -> bool {
        self.components
            .get(self.current_component_index)
            .map(|c| !c.identifier.is_empty())
            .unwrap_or(false)
    }

    /// Returns `true` if the current mixture component has a weight fraction.
    pub fn current_component_has_fraction(&self) -> bool {
        self.components
            .get(self.current_component_index)
            .map(|c| c.weight_fraction_pct.is_some())
            .unwrap_or(false)
    }
}
