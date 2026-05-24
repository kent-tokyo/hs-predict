//! LLM-based HS code classification — **trait hook** (v0.4).
//!
//! `hs-predict` deliberately does **not** ship a concrete LLM API client.
//! Instead it defines the [`LlmClassifier`] trait, which you implement with
//! whatever HTTP transport, model, and prompt customisation your application
//! requires.
//!
//! The library provides:
//! - [`LlmPrompt`] — pre-built system + user text (EN/JA) ready to send
//! - [`LlmResponse`] — the expected return value from your implementation
//! - [`parse_llm_json`] — helper that strips markdown fences and deserialises
//!   the LLM's JSON reply into an [`LlmResponse`]
//! - [`MockLlmClassifier`] — deterministic stub for unit tests (`mock` feature)
//!
//! Requires the **`llm`** Cargo feature.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "llm")]
//! # mod example {
//! use hs_predict::llm::{LlmClassifier, LlmPrompt, LlmResponse, parse_llm_json};
//! use futures::future::BoxFuture;
//!
//! struct MyClient { api_key: String }
//!
//! impl LlmClassifier for MyClient {
//!     fn classify<'a>(&'a self, prompt: &'a LlmPrompt) -> BoxFuture<'a, hs_predict::Result<LlmResponse>> {
//!         Box::pin(async move {
//!             // 1. Call your LLM API using prompt.system_text / prompt.user_text
//!             let raw_json: String = todo!("send HTTP request, receive text");
//!             // 2. Parse and return
//!             parse_llm_json(&raw_json)
//!         })
//!     }
//! }
//! # }
//! ```

pub mod mock;
pub mod prompt;

#[cfg(feature = "mock")]
pub use mock::MockLlmClassifier;
pub use prompt::PromptBuilder;

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// LlmPrompt
// ─────────────────────────────────────────────────────────────────────────────

/// Input passed to [`LlmClassifier::classify`].
///
/// Contains pre-built prompt text as well as structured SMILES analysis
/// for implementations that want to build a custom prompt.
#[derive(Debug, Clone)]
pub struct LlmPrompt {
    /// Pre-built system prompt (role + format instructions + confidence guide).
    pub system_text: String,

    /// Pre-built user message (all product identifiers, physical description,
    /// and SMILES functional-group hints if available).
    pub user_text: String,

    /// SMILES-based pre-classification, if a SMILES string was available.
    /// Useful for building custom prompts or for post-call chapter validation.
    pub smiles_analysis: Option<crate::smiles::SmilesClassification>,
}

// ─────────────────────────────────────────────────────────────────────────────
// LlmResponse
// ─────────────────────────────────────────────────────────────────────────────

/// Response that [`LlmClassifier::classify`] must return.
///
/// All fields map directly to fields of [`HsPrediction`](crate::types::HsPrediction).
///
/// # JSON schema expected from the LLM
/// ```json
/// {
///   "hs_code":    "291511",
///   "confidence": 0.85,
///   "rationale":  "Acetic acid → heading 29.15 (saturated acyclic carboxylic acid).",
///   "alternatives": [
///     { "hs_code": "291519", "confidence": 0.10, "reason": "If purity threshold not met." }
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Six-digit HS 2022 code, no punctuation (e.g. `"291511"`).
    ///
    /// The pipeline validates this is exactly 6 ASCII digits before accepting it.
    pub hs_code: String,

    /// Confidence score in [0.0, 1.0].
    pub confidence: f32,

    /// Natural-language rationale (1–3 sentences).
    pub rationale: String,

    /// Alternative HS codes with lower confidence. May be empty.
    #[serde(default)]
    pub alternatives: Vec<LlmAlternative>,
}

/// An alternative HS code suggestion returned by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAlternative {
    /// Six-digit HS 2022 code.
    pub hs_code: String,
    /// Confidence for this alternative, in [0.0, 1.0].
    pub confidence: f32,
    /// Why this alternative applies.
    pub reason: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// LlmClassifier trait
// ─────────────────────────────────────────────────────────────────────────────

/// Trait for LLM-based HS code classification.
///
/// Implement this with your preferred LLM provider (Anthropic Claude,
/// OpenAI GPT-4o, local Ollama, …) and attach it to the pipeline via
/// [`HsPipeline::with_llm`](crate::pipeline::HsPipeline::with_llm).
///
/// # Contract
/// - Must return an [`LlmResponse`] with `hs_code` that is exactly 6 ASCII
///   digits. The pipeline validates this and returns
///   [`HsPredictError::ValidationFailed`](crate::HsPredictError::ValidationFailed)
///   if the code is malformed.
/// - `confidence` should follow the guide in [`LlmPrompt::system_text`]:
///   ≥ 0.90 for certain sub-heading, ≥ 0.70 for certain heading.
/// - Must be `Send + Sync` (required for `Arc<dyn LlmClassifier>`).
///
/// # Minimal implementation
/// ```rust,no_run
/// # #[cfg(feature = "llm")]
/// # {
/// use hs_predict::llm::{LlmClassifier, LlmPrompt, LlmResponse, parse_llm_json};
/// use futures::future::BoxFuture;
///
/// struct MyClient;
///
/// impl LlmClassifier for MyClient {
///     fn classify<'a>(&'a self, prompt: &'a LlmPrompt) -> BoxFuture<'a, hs_predict::Result<LlmResponse>> {
///         Box::pin(async move {
///             let raw = String::from(r#"{"hs_code":"291511","confidence":0.85,"rationale":"...","alternatives":[]}"#);
///             parse_llm_json(&raw)
///         })
///     }
/// }
/// # }
/// ```
pub trait LlmClassifier: Send + Sync {
    /// Classify the product described in `prompt` and return an HS code prediction.
    fn classify<'a>(
        &'a self,
        prompt: &'a LlmPrompt,
    ) -> BoxFuture<'a, crate::Result<LlmResponse>>;
}

// ─────────────────────────────────────────────────────────────────────────────
// parse_llm_json helper
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a raw LLM API text response into an [`LlmResponse`].
///
/// Handles the most common formatting quirks LLMs exhibit:
/// - Plain JSON
/// - JSON wrapped in ` ```json … ``` ` markdown fences
/// - JSON wrapped in plain ` ``` … ``` ` fences
/// - Leading / trailing whitespace
///
/// # Errors
/// Returns [`HsPredictError::LlmResponseParseError`](crate::HsPredictError::LlmResponseParseError)
/// if the string cannot be deserialised as [`LlmResponse`].
///
/// # Example
/// ```rust
/// # #[cfg(feature = "llm")]
/// # {
/// use hs_predict::llm::{parse_llm_json, LlmResponse};
///
/// let raw = r#"```json
/// {"hs_code":"291511","confidence":0.85,"rationale":"Acetic acid.","alternatives":[]}
/// ```"#;
///
/// let r: LlmResponse = parse_llm_json(raw).unwrap();
/// assert_eq!(r.hs_code, "291511");
/// # }
/// ```
pub fn parse_llm_json(raw: &str) -> crate::Result<LlmResponse> {
    let json_str = strip_markdown_fences(raw);
    serde_json::from_str::<LlmResponse>(json_str.trim()).map_err(|e| {
        crate::HsPredictError::LlmResponseParseError {
            source: e,
            raw: raw.to_string(),
        }
    })
}

/// Strip ` ```json ` or ` ``` ` fences from LLM output.
fn strip_markdown_fences(s: &str) -> &str {
    let s = s.trim();
    // Try ```json first, then plain ```
    let inner = s
        .strip_prefix("```json")
        .or_else(|| s.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"));
    inner.map(str::trim).unwrap_or(s)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_json() {
        let raw = r#"{"hs_code":"291511","confidence":0.85,"rationale":"test","alternatives":[]}"#;
        let r = parse_llm_json(raw).unwrap();
        assert_eq!(r.hs_code, "291511");
        assert!((r.confidence - 0.85).abs() < 0.001);
    }

    #[test]
    fn parse_json_with_json_fence() {
        let raw = "```json\n{\"hs_code\":\"280511\",\"confidence\":0.9,\"rationale\":\"ok\",\"alternatives\":[]}\n```";
        let r = parse_llm_json(raw).unwrap();
        assert_eq!(r.hs_code, "280511");
    }

    #[test]
    fn parse_json_with_plain_fence() {
        let raw = "```\n{\"hs_code\":\"290900\",\"confidence\":0.6,\"rationale\":\"ether\",\"alternatives\":[]}\n```";
        let r = parse_llm_json(raw).unwrap();
        assert_eq!(r.hs_code, "290900");
    }

    #[test]
    fn parse_alternatives_populated() {
        let raw = r#"{
            "hs_code": "291511",
            "confidence": 0.75,
            "rationale": "likely acetic acid",
            "alternatives": [
                { "hs_code": "291519", "confidence": 0.15, "reason": "other acids" }
            ]
        }"#;
        let r = parse_llm_json(raw).unwrap();
        assert_eq!(r.alternatives.len(), 1);
        assert_eq!(r.alternatives[0].hs_code, "291519");
    }

    #[test]
    fn parse_missing_alternatives_defaults_to_empty() {
        let raw = r#"{"hs_code":"290900","confidence":0.7,"rationale":"ether"}"#;
        let r = parse_llm_json(raw).unwrap();
        assert!(r.alternatives.is_empty());
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        let result = parse_llm_json("not json at all");
        assert!(result.is_err());
    }
}
