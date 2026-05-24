//! Interactive Akinator-style classification session.
//!
//! Instead of requiring all product information upfront, [`ClassificationSession`]
//! asks targeted questions one at a time to progressively narrow down the HS code.
//!
//! # Example
//!
//! ```rust,no_run
//! use hs_predict::session::{ClassificationSession, Answer, SessionResult};
//! use hs_predict::types::Language;
//!
//! let mut session = ClassificationSession::new(); // English prompts
//!
//! let q1 = session.start();
//! println!("{}", q1.prompt());
//!
//! match session.answer(Answer::Text("1310-73-2".to_string())).unwrap() {
//!     SessionResult::NeedMoreInfo { next_question } => {
//!         println!("Next: {}", next_question.prompt());
//!     }
//!     SessionResult::Ready => {
//!         println!("Ready to classify!");
//!     }
//!     SessionResult::RequiresLlm => {
//!         println!("LLM needed");
//!     }
//! }
//! ```

pub mod flow;
pub mod messages;
pub mod question;
pub mod state;

use serde::{Deserialize, Serialize};

pub use question::{Answer, QAPair, Question, QuestionStep, SessionResult};
pub use state::{ClassificationState, PartialComponent};

use crate::error::{HsPredictError, Result};
use crate::session::flow::{
    choice_index_to_intended_use, choice_index_to_organic_inorganic,
    choice_index_to_physical_form, multi_choice_indices_to_functional_groups, next_question,
};
use crate::types::{Language, MixtureComponent, PhysicalForm, ProductDescription, SubstanceIdentifier};

/// Interactive HS code classification session.
///
/// Maintains state across multiple question-answer rounds and builds up a
/// [`ProductDescription`] that can be passed to the classification pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationSession {
    /// Accumulated classification state.
    state: ClassificationState,
    /// Full history of Q&A pairs (used for serialization / resume).
    history: Vec<QAPair>,
    /// The question currently pending an answer.
    current_question: Option<Question>,
    /// The logical step of `current_question` (language-independent).
    current_step: Option<QuestionStep>,
    /// Language used for question prompts.
    language: Language,
}

impl ClassificationSession {
    /// Create a new empty session with English prompts.
    pub fn new() -> Self {
        Self {
            state: ClassificationState::default(),
            history: Vec::new(),
            current_question: None,
            current_step: None,
            language: Language::En,
        }
    }

    /// Create a new empty session with Japanese prompts.
    pub fn new_ja() -> Self {
        Self::new().with_language(Language::Ja)
    }

    /// Set the language for question prompts (builder style).
    ///
    /// Must be called before [`start()`](Self::start).
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Return the first question and mark it as the active question.
    ///
    /// Must be called once before the first [`answer()`](Self::answer) call.
    pub fn start(&mut self) -> Question {
        let (q, step) = next_question(&self.state, self.language)
            .expect("new session should always have a first question");
        self.current_question = Some(q.clone());
        self.current_step = Some(step);
        q
    }

    /// Submit an answer to the current active question.
    ///
    /// Returns the next [`SessionResult`]:
    /// - [`SessionResult::NeedMoreInfo`] — more questions remain
    /// - [`SessionResult::Ready`] — call [`to_product_description()`](Self::to_product_description)
    /// - [`SessionResult::RequiresLlm`] — insufficient info for rule engine alone
    ///
    /// # Errors
    /// - [`HsPredictError::NoActiveQuestion`] — called before [`start()`](Self::start).
    /// - [`HsPredictError::AnswerTypeMismatch`] — answer type doesn't match the question.
    /// - [`HsPredictError::InvalidChoiceIndex`] — choice index out of range.
    /// - [`HsPredictError::NumberOutOfRange`] — number outside `[min, max]`.
    pub fn answer(&mut self, answer: Answer) -> Result<SessionResult> {
        let question = self
            .current_question
            .clone()
            .ok_or(HsPredictError::NoActiveQuestion)?;

        // Validate answer type and apply to state
        self.validate_and_apply(&question, &answer)?;

        // Record in history
        self.history.push(QAPair {
            question: question.clone(),
            answer,
        });

        // Try to resolve IUPAC name → SMILES
        self.try_resolve_smiles();

        // Determine next step
        match next_question(&self.state, self.language) {
            Some((q, step)) => {
                self.current_question = Some(q.clone());
                self.current_step = Some(step);
                Ok(SessionResult::NeedMoreInfo { next_question: q })
            }
            None => {
                self.state.is_complete = true;
                self.current_question = None;
                self.current_step = None;
                if self.state.confidence_estimate() < 0.25 {
                    Ok(SessionResult::RequiresLlm)
                } else {
                    Ok(SessionResult::Ready)
                }
            }
        }
    }

    /// Convert the accumulated session state into a [`ProductDescription`].
    ///
    /// Call after receiving [`SessionResult::Ready`] or [`SessionResult::RequiresLlm`].
    pub fn to_product_description(&self) -> ProductDescription {
        let mixture_components = if self.state.is_mixture == Some(true) {
            Some(
                self.state
                    .components
                    .iter()
                    .map(|c| MixtureComponent {
                        substance: c.identifier.clone(),
                        weight_fraction_pct: c.weight_fraction_pct,
                        volume_fraction_pct: None,
                        is_solvent: c.is_solvent,
                    })
                    .collect(),
            )
        } else {
            None
        };

        ProductDescription {
            identifier: self.state.identifier.clone(),
            physical_form: self.state.physical_form.clone(),
            purity_pct: self.state.purity_pct,
            purity_type: None,
            mixture_components,
            intended_use: self.state.intended_use.clone(),
            additional_context: None,
        }
    }

    /// Current session state (read-only).
    pub fn state(&self) -> &ClassificationState {
        &self.state
    }

    /// Full Q&A history.
    pub fn history(&self) -> &[QAPair] {
        &self.history
    }

    /// Number of questions answered so far.
    pub fn question_count(&self) -> usize {
        self.history.len()
    }

    /// Whether the session has collected enough information.
    pub fn is_complete(&self) -> bool {
        self.state.is_complete
    }

    /// The language used for question prompts.
    pub fn language(&self) -> Language {
        self.language
    }

    /// The logical step of the current active question, if any.
    pub fn current_step(&self) -> Option<QuestionStep> {
        self.current_step
    }

    // ─── Private: validate & apply ────────────────────────────────────

    fn validate_and_apply(&mut self, question: &Question, answer: &Answer) -> Result<()> {
        match (question, answer) {
            // Text input
            (Question::Text { .. }, Answer::Text(text)) => {
                self.apply_identifier_input(text);
            }
            (Question::Text { .. }, Answer::Skip) => {
                if !self.state.has_identifier()
                    && self.current_step != Some(QuestionStep::ComponentIdentifier)
                {
                    return Err(HsPredictError::MissingIdentifier);
                }
            }

            // Yes/No
            (Question::YesNo { .. }, Answer::YesNo(val)) => {
                self.apply_yes_no(*val);
            }

            // Number
            (Question::Number { min, max, .. }, Answer::Number(val)) => {
                if *val < *min || *val > *max {
                    return Err(HsPredictError::NumberOutOfRange {
                        value: *val,
                        min: *min,
                        max: *max,
                    });
                }
                self.apply_number(*val);
            }

            // Single choice
            (Question::Choice { options, .. }, Answer::Choice(idx)) => {
                if *idx >= options.len() {
                    return Err(HsPredictError::InvalidChoiceIndex {
                        index: *idx,
                        max: options.len() - 1,
                    });
                }
                self.apply_choice(*idx);
            }

            // Multi choice
            (Question::MultiChoice { options, .. }, Answer::MultiChoice(indices)) => {
                for &idx in indices {
                    if idx >= options.len() {
                        return Err(HsPredictError::InvalidChoiceIndex {
                            index: idx,
                            max: options.len() - 1,
                        });
                    }
                }
                self.apply_multi_choice(indices);
            }

            // Type mismatch
            _ => {
                return Err(HsPredictError::AnswerTypeMismatch {
                    expected: question_kind_name(question),
                    got: answer.kind_name(),
                });
            }
        }
        Ok(())
    }

    /// Parse and store an identifier string.
    fn apply_identifier_input(&mut self, input: &str) {
        let input = input.trim();

        let in_mixture = self.state.is_mixture == Some(true)
            && self.state.current_component_index < self.state.component_count.unwrap_or(0);

        if in_mixture {
            let idx = self.state.current_component_index;
            while self.state.components.len() <= idx {
                self.state.components.push(PartialComponent::default());
            }
            self.state.components[idx].identifier = parse_identifier(input);
        } else {
            self.state.identifier = parse_identifier(input);
        }
    }

    fn apply_yes_no(&mut self, val: bool) {
        match self.current_step {
            Some(QuestionStep::IsMixture) => {
                self.state.is_mixture = Some(val);
            }
            _ => {}
        }
    }

    fn apply_number(&mut self, val: f64) {
        match self.current_step {
            Some(QuestionStep::ComponentCount) => {
                self.state.component_count = Some(val as usize);
            }
            Some(QuestionStep::ComponentFraction) => {
                let idx = self.state.current_component_index;
                if idx < self.state.components.len() {
                    self.state.components[idx].weight_fraction_pct =
                        if val > 0.0 { Some(val) } else { None };
                    self.state.current_component_index += 1;
                }
            }
            Some(QuestionStep::SolutionConcentration) => {
                if let Some(PhysicalForm::Solution { concentration_pct_ww, .. }) =
                    &mut self.state.physical_form
                {
                    *concentration_pct_ww = if val > 0.0 { Some(val) } else { None };
                }
            }
            _ => {}
        }
    }

    fn apply_choice(&mut self, idx: usize) {
        match self.current_step {
            Some(QuestionStep::PhysicalForm) => {
                self.state.physical_form = Some(choice_index_to_physical_form(idx));
            }
            Some(QuestionStep::IntendedUse) => {
                self.state.intended_use = Some(choice_index_to_intended_use(idx));
            }
            Some(QuestionStep::OrganicInorganic) => {
                self.state.organic_inorganic = Some(choice_index_to_organic_inorganic(idx));
            }
            _ => {}
        }
    }

    fn apply_multi_choice(&mut self, indices: &[usize]) {
        self.state.detected_functional_groups = multi_choice_indices_to_functional_groups(indices);
    }

    /// Attempt to resolve IUPAC name → SMILES (silent on failure).
    fn try_resolve_smiles(&mut self) {
        if self.state.identifier.smiles.is_none() {
            if let Some(ref iupac) = self.state.identifier.iupac_name.clone() {
                if let Some(smiles) = resolve_iupac_to_smiles(iupac) {
                    self.state.identifier.smiles = Some(smiles);
                }
            }
        }
        for comp in &mut self.state.components {
            if comp.identifier.smiles.is_none() {
                if let Some(ref iupac) = comp.identifier.iupac_name.clone() {
                    if let Some(smiles) = resolve_iupac_to_smiles(iupac) {
                        comp.identifier.smiles = Some(smiles);
                    }
                }
            }
        }
    }
}

impl Default for ClassificationSession {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Free functions ────────────────────────────────────────────────────────────

fn parse_identifier(input: &str) -> SubstanceIdentifier {
    let s = input.trim();

    if is_cas_format(s) {
        return SubstanceIdentifier::from_cas(s);
    }
    if is_inchi_key_format(s) {
        return SubstanceIdentifier {
            inchi_key: Some(s.to_string()),
            ..Default::default()
        };
    }
    if s.starts_with("InChI=") {
        return SubstanceIdentifier {
            inchi: Some(s.to_string()),
            ..Default::default()
        };
    }
    if !s.contains(' ')
        && s.chars()
            .any(|c| matches!(c, '(' | ')' | '=' | '#' | '[' | ']' | '+' | '-'))
    {
        return SubstanceIdentifier::from_smiles(s);
    }
    SubstanceIdentifier::from_iupac_name(s)
}

fn is_cas_format(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 3
        && parts[0].len() >= 2
        && parts[0].chars().all(|c| c.is_ascii_digit())
        && parts[1].len() == 2
        && parts[1].chars().all(|c| c.is_ascii_digit())
        && parts[2].len() == 1
        && parts[2].chars().all(|c| c.is_ascii_digit())
}

fn is_inchi_key_format(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 3
        && parts[0].len() == 14
        && parts[1].len() == 10
        && parts[2].len() == 1
        && s.chars().all(|c| c.is_ascii_uppercase() || c == '-')
}

fn resolve_iupac_to_smiles(iupac_name: &str) -> Option<String> {
    chem_name_resolver::resolve(iupac_name)
        .ok()
        .map(|r| r.smiles)
}

fn question_kind_name(q: &Question) -> &'static str {
    match q {
        Question::Text { .. } => "text",
        Question::Choice { .. } => "choice",
        Question::YesNo { .. } => "yes_no",
        Question::Number { .. } => "number",
        Question::MultiChoice { .. } => "multi_choice",
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{IntendedUse, OrganicInorganic, PhysicalForm};

    /// Helper: unwrap the next_question from SessionResult::NeedMoreInfo.
    fn next_q(result: SessionResult) -> Question {
        match result {
            SessionResult::NeedMoreInfo { next_question } => next_question,
            other => panic!("expected NeedMoreInfo, got {:?}", std::mem::discriminant(&other)),
        }
    }

    // ─── English session flow ─────────────────────────────────────────

    #[test]
    fn session_starts_with_identifier_question() {
        let mut session = ClassificationSession::new();
        let q = session.start();
        assert!(matches!(q, Question::Text { .. }));
        assert_eq!(session.current_step(), Some(QuestionStep::Identifier));
    }

    #[test]
    fn session_pure_cas_inorganic_full_flow() {
        // CAS input, pure substance, solid, industrial, inorganic → Ready
        let mut session = ClassificationSession::new();
        session.start();

        // Q1 identifier → Q2 is_mixture?
        let r = session.answer(Answer::Text("1310-73-2".to_string())).unwrap();
        assert!(matches!(next_q(r), Question::YesNo { .. }));
        assert_eq!(session.current_step(), Some(QuestionStep::IsMixture));

        // Q2 not mixture → Q3 physical form
        let r = session.answer(Answer::YesNo(false)).unwrap();
        assert!(matches!(next_q(r), Question::Choice { .. }));
        assert_eq!(session.current_step(), Some(QuestionStep::PhysicalForm));

        // Q3 solid (index 0) → Q4 intended use
        let r = session.answer(Answer::Choice(0)).unwrap();
        assert!(matches!(next_q(r), Question::Choice { .. }));
        assert_eq!(session.current_step(), Some(QuestionStep::IntendedUse));

        // Q4 industrial (index 0) → Q5 organic/inorganic (CAS has no SMILES)
        let r = session.answer(Answer::Choice(0)).unwrap();
        assert!(matches!(next_q(r), Question::Choice { .. }));
        assert_eq!(session.current_step(), Some(QuestionStep::OrganicInorganic));

        // Q5 inorganic (index 1) → Ready
        let r = session.answer(Answer::Choice(1)).unwrap();
        assert!(matches!(r, SessionResult::Ready));

        // Verify accumulated state
        let product = session.to_product_description();
        assert_eq!(product.identifier.cas.as_deref(), Some("1310-73-2"));
        assert!(matches!(product.physical_form, Some(PhysicalForm::Solid)));
        assert_eq!(product.intended_use, Some(IntendedUse::Industrial));
        assert_eq!(session.question_count(), 5);
        assert!(session.is_complete());
    }

    #[test]
    fn session_smiles_input_skips_organic_inorganic_question() {
        // SMILES input: organic/inorganic and functional-group questions are skipped.
        let mut session = ClassificationSession::new();
        session.start();

        // Identifier: SMILES string for NaOH → smiles is set
        let r = session.answer(Answer::Text("[Na+].[OH-]".to_string())).unwrap();
        assert!(matches!(next_q(r), Question::YesNo { .. }));

        // Not mixture
        let r = session.answer(Answer::YesNo(false)).unwrap();
        assert!(matches!(next_q(r), Question::Choice { .. })); // physical form

        // Liquid (index 3)
        let r = session.answer(Answer::Choice(3)).unwrap();
        assert!(matches!(next_q(r), Question::Choice { .. })); // intended use

        // Industrial
        let r = session.answer(Answer::Choice(0)).unwrap();
        // SMILES is set → organic/inorganic skipped → Ready
        assert!(matches!(r, SessionResult::Ready));
        assert_eq!(session.question_count(), 4);

        let product = session.to_product_description();
        assert!(product.identifier.smiles.is_some());
    }

    #[test]
    fn session_organic_cas_asks_functional_groups() {
        // CAS input + organic → functional-group question appears.
        let mut session = ClassificationSession::new();
        session.start();

        session.answer(Answer::Text("108-88-3".to_string())).unwrap(); // toluene CAS
        session.answer(Answer::YesNo(false)).unwrap();         // not mixture
        session.answer(Answer::Choice(0)).unwrap();            // solid
        session.answer(Answer::Choice(0)).unwrap();            // industrial
        let r = session.answer(Answer::Choice(0)).unwrap();    // organic (index 0)

        // Must be functional-group MultiChoice next
        let q = next_q(r);
        assert!(matches!(q, Question::MultiChoice { .. }));
        assert_eq!(session.current_step(), Some(QuestionStep::FunctionalGroups));

        // Select aromatic (index 10) and no others
        let r = session.answer(Answer::MultiChoice(vec![10])).unwrap();
        assert!(matches!(r, SessionResult::Ready));

        let state = session.state();
        assert_eq!(state.organic_inorganic, Some(OrganicInorganic::Organic));
        assert!(state.detected_functional_groups.contains(&"aromatic".to_string()));
    }

    #[test]
    fn session_solution_asks_concentration() {
        let mut session = ClassificationSession::new();
        session.start();

        session.answer(Answer::Text("7647-01-0".to_string())).unwrap(); // HCl CAS
        session.answer(Answer::YesNo(false)).unwrap();

        // Solution (index 4)
        let r = session.answer(Answer::Choice(4)).unwrap();
        let q = next_q(r);
        assert!(matches!(q, Question::Number { .. }));
        assert_eq!(session.current_step(), Some(QuestionStep::SolutionConcentration));

        // Concentration 35%
        let r = session.answer(Answer::Number(35.0)).unwrap();
        assert!(matches!(next_q(r), Question::Choice { .. })); // intended use

        session.answer(Answer::Choice(0)).unwrap(); // industrial
        let r = session.answer(Answer::Choice(1)).unwrap(); // inorganic → Ready
        assert!(matches!(r, SessionResult::Ready));

        let product = session.to_product_description();
        assert_eq!(
            product.physical_form,
            Some(PhysicalForm::Solution {
                solvent: None,
                concentration_pct_ww: Some(35.0),
            })
        );
    }

    // ─── Mixture flow ─────────────────────────────────────────────────

    #[test]
    fn session_mixture_two_components() {
        let mut session = ClassificationSession::new();
        session.start();

        // Step 1: main identifier
        session.answer(Answer::Text("7664-93-9".to_string())).unwrap(); // H2SO4

        // Step 2: is mixture → yes
        session.answer(Answer::YesNo(true)).unwrap();
        assert_eq!(session.current_step(), Some(QuestionStep::ComponentCount));

        // Step 3: 2 components
        session.answer(Answer::Number(2.0)).unwrap();
        assert_eq!(session.current_step(), Some(QuestionStep::ComponentIdentifier));

        // Component 1 identifier
        session.answer(Answer::Text("7664-93-9".to_string())).unwrap();
        assert_eq!(session.current_step(), Some(QuestionStep::ComponentFraction));

        // Component 1 fraction
        session.answer(Answer::Number(70.0)).unwrap();
        assert_eq!(session.current_step(), Some(QuestionStep::ComponentIdentifier));

        // Component 2 identifier
        session.answer(Answer::Text("7732-18-5".to_string())).unwrap(); // water
        assert_eq!(session.current_step(), Some(QuestionStep::ComponentFraction));

        // Component 2 fraction → done
        let r = session.answer(Answer::Number(30.0)).unwrap();
        assert!(matches!(r, SessionResult::Ready | SessionResult::RequiresLlm));

        let product = session.to_product_description();
        let comps = product.mixture_components.unwrap();
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].substance.cas.as_deref(), Some("7664-93-9"));
        assert_eq!(comps[0].weight_fraction_pct, Some(70.0));
        assert_eq!(comps[1].substance.cas.as_deref(), Some("7732-18-5"));
        assert_eq!(comps[1].weight_fraction_pct, Some(30.0));
    }

    // ─── Error handling ───────────────────────────────────────────────

    #[test]
    fn error_no_active_question_before_start() {
        let mut session = ClassificationSession::new();
        let err = session.answer(Answer::Text("1310-73-2".to_string())).unwrap_err();
        assert!(matches!(err, HsPredictError::NoActiveQuestion));
    }

    #[test]
    fn error_answer_type_mismatch() {
        let mut session = ClassificationSession::new();
        session.start(); // Q1 is a Text question
        let err = session.answer(Answer::YesNo(true)).unwrap_err();
        assert!(matches!(err, HsPredictError::AnswerTypeMismatch { .. }));
    }

    #[test]
    fn error_choice_index_out_of_range() {
        let mut session = ClassificationSession::new();
        session.start();
        session.answer(Answer::Text("1310-73-2".to_string())).unwrap();
        session.answer(Answer::YesNo(false)).unwrap(); // physical form question
        let err = session.answer(Answer::Choice(99)).unwrap_err();
        assert!(matches!(err, HsPredictError::InvalidChoiceIndex { .. }));
    }

    #[test]
    fn error_number_out_of_range() {
        let mut session = ClassificationSession::new();
        session.start();
        session.answer(Answer::Text("1310-73-2".to_string())).unwrap();
        session.answer(Answer::YesNo(true)).unwrap(); // component count question
        let err = session.answer(Answer::Number(1.0)).unwrap_err(); // min is 2
        assert!(matches!(err, HsPredictError::NumberOutOfRange { .. }));
    }

    // ─── Japanese language ────────────────────────────────────────────

    #[test]
    fn japanese_session_prompts_are_in_japanese() {
        let mut session = ClassificationSession::new_ja();
        let q = session.start();
        // The Japanese identifier prompt contains Japanese characters
        assert!(q.prompt().chars().any(|c| c as u32 > 0x7F));
    }

    #[test]
    fn japanese_session_completes_same_as_english() {
        // Logic is language-independent; only prompts differ.
        let mut session = ClassificationSession::new_ja();
        session.start();

        session.answer(Answer::Text("1310-73-2".to_string())).unwrap();
        session.answer(Answer::YesNo(false)).unwrap();
        session.answer(Answer::Choice(0)).unwrap(); // solid
        session.answer(Answer::Choice(0)).unwrap(); // industrial
        let r = session.answer(Answer::Choice(1)).unwrap(); // inorganic

        assert!(matches!(r, SessionResult::Ready));
        let product = session.to_product_description();
        assert_eq!(product.identifier.cas.as_deref(), Some("1310-73-2"));
    }

    // ─── Serialization round-trip ─────────────────────────────────────

    #[test]
    fn session_serializes_and_deserializes() {
        let mut session = ClassificationSession::new();
        session.start();
        session.answer(Answer::Text("1310-73-2".to_string())).unwrap();

        let json = serde_json::to_string(&session).unwrap();
        let restored: ClassificationSession = serde_json::from_str(&json).unwrap();

        assert_eq!(
            restored.state().identifier.cas.as_deref(),
            Some("1310-73-2")
        );
        assert_eq!(restored.language(), Language::En);
        assert_eq!(restored.current_step(), Some(QuestionStep::IsMixture));
    }
}
