//! Question decision tree.
//!
//! Reads the current [`ClassificationState`] and returns the next
//! [`Question`] + [`QuestionStep`] pair to present to the user.
//! Pure functions; no side-effects.

use crate::session::messages as msg;
use crate::session::question::{Question, QuestionStep};
use crate::session::state::ClassificationState;
use crate::types::{Language, OrganicInorganic, PhysicalForm};

/// Return the next question to ask (and its step tag), or `None` when all
/// information has been collected and the pipeline can be invoked.
pub fn next_question(
    state: &ClassificationState,
    lang: Language,
) -> Option<(Question, QuestionStep)> {
    // ── Step 1: identifier ───────────────────────────────────────────
    if !state.has_identifier() {
        let (prompt, example) = msg::q_identifier(lang);
        return Some((
            Question::Text { prompt, example: Some(example) },
            QuestionStep::Identifier,
        ));
    }

    // ── Step 2: is it a mixture? ────────────────────────────────────
    if state.is_mixture.is_none() {
        return Some((
            Question::YesNo { prompt: msg::q_is_mixture(lang) },
            QuestionStep::IsMixture,
        ));
    }

    // ── Step 3a: mixture branch ─────────────────────────────────────
    if state.is_mixture == Some(true) {
        if state.component_count.is_none() {
            let (prompt, unit) = msg::q_component_count(lang);
            return Some((
                Question::Number { prompt, unit, min: 2.0, max: 20.0 },
                QuestionStep::ComponentCount,
            ));
        }

        let expected = state.component_count.unwrap_or(0);
        let idx = state.current_component_index;

        // Current component — identifier missing
        if idx < expected && !state.current_component_has_identifier() {
            let (prompt, example) = msg::q_component_identifier(lang, idx + 1);
            return Some((
                Question::Text { prompt, example: Some(example) },
                QuestionStep::ComponentIdentifier,
            ));
        }

        // Current component — weight fraction missing
        if idx < expected && !state.current_component_has_fraction() {
            let name = state
                .components
                .get(idx)
                .and_then(|c| c.identifier.iupac_name.clone())
                .or_else(|| state.components.get(idx).and_then(|c| c.identifier.cas.clone()))
                .unwrap_or_else(|| format!("component {}", idx + 1));
            let (prompt, unit) = msg::q_component_fraction(lang, &name);
            return Some((
                Question::Number { prompt, unit, min: 0.0, max: 100.0 },
                QuestionStep::ComponentFraction,
            ));
        }

        // Still more components to collect
        if state.components.len() < expected {
            let next_idx = state.components.len();
            let (prompt, example) = msg::q_next_component_identifier(lang, next_idx + 1);
            return Some((
                Question::Text { prompt, example: Some(example) },
                QuestionStep::ComponentIdentifier,
            ));
        }

        // All components entered → hand off to pipeline
        return None;
    }

    // ── Step 3b: pure substance branch ──────────────────────────────

    // Physical form
    if state.physical_form.is_none() {
        let (prompt, options) = msg::q_physical_form(lang);
        return Some((
            Question::Choice { prompt, options },
            QuestionStep::PhysicalForm,
        ));
    }

    // Solution concentration (when form is Solution with unknown concentration)
    if let Some(PhysicalForm::Solution { concentration_pct_ww: None, .. }) = &state.physical_form {
        let (prompt, unit) = msg::q_solution_concentration(lang);
        return Some((
            Question::Number { prompt, unit, min: 0.0, max: 100.0 },
            QuestionStep::SolutionConcentration,
        ));
    }

    // Intended use
    if state.intended_use.is_none() {
        let (prompt, options) = msg::q_intended_use(lang);
        return Some((
            Question::Choice { prompt, options },
            QuestionStep::IntendedUse,
        ));
    }

    // Organic / inorganic (only when SMILES is unknown)
    if state.organic_inorganic.is_none() && state.identifier.smiles.is_none() {
        let (prompt, options) = msg::q_organic_inorganic(lang);
        return Some((
            Question::Choice { prompt, options },
            QuestionStep::OrganicInorganic,
        ));
    }

    // Functional groups (only for organic compounds without SMILES)
    if state.identifier.smiles.is_none()
        && matches!(state.organic_inorganic, Some(OrganicInorganic::Organic))
        && state.detected_functional_groups.is_empty()
        && state.chapter_hint.is_none()
    {
        let (prompt, options) = msg::q_functional_groups(lang);
        return Some((
            Question::MultiChoice { prompt, options, include_unknown: true },
            QuestionStep::FunctionalGroups,
        ));
    }

    // Nothing more to ask → pipeline
    None
}

// ─── Index-to-value converters ────────────────────────────────────────────────
// These are language-independent; indices map to the option order defined in
// messages.rs (both EN and JA share the same ordering).

/// Map physical-form choice index to [`PhysicalForm`].
pub fn choice_index_to_physical_form(index: usize) -> PhysicalForm {
    match index {
        0 => PhysicalForm::Solid,
        1 => PhysicalForm::Powder { particle_size_um: None },
        2 => PhysicalForm::Granules,
        3 => PhysicalForm::Liquid,
        4 => PhysicalForm::Solution { solvent: None, concentration_pct_ww: None },
        5 => PhysicalForm::Gas,
        6 => PhysicalForm::Foil { thickness_mm: None },
        7 => PhysicalForm::Ingot,
        _ => PhysicalForm::Unknown,
    }
}

/// Map intended-use choice index to [`IntendedUse`](crate::types::IntendedUse).
pub fn choice_index_to_intended_use(index: usize) -> crate::types::IntendedUse {
    use crate::types::IntendedUse;
    match index {
        0 => IntendedUse::Industrial,
        1 => IntendedUse::Pharmaceutical,
        2 => IntendedUse::Agricultural,
        3 => IntendedUse::Food,
        4 => IntendedUse::Cosmetic,
        _ => IntendedUse::Other("unknown".to_string()),
    }
}

/// Map organic/inorganic choice index to [`OrganicInorganic`].
pub fn choice_index_to_organic_inorganic(index: usize) -> OrganicInorganic {
    match index {
        0 => OrganicInorganic::Organic,
        1 => OrganicInorganic::Inorganic,
        _ => OrganicInorganic::Unknown,
    }
}

/// Map functional-group multi-choice indices to string keys.
pub fn multi_choice_indices_to_functional_groups(indices: &[usize]) -> Vec<String> {
    const GROUPS: &[&str] = &[
        "carboxylic_acid",
        "alcohol",
        "phenol",
        "aldehyde",
        "ketone",
        "amine",
        "amide",
        "nitrile",
        "halide",
        "ester",
        "aromatic",
    ];
    indices
        .iter()
        .filter_map(|&i| GROUPS.get(i).map(|s| s.to_string()))
        .collect()
}
