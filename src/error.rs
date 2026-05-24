use thiserror::Error;

/// All errors produced by `hs-predict`.
#[derive(Debug, Error)]
pub enum HsPredictError {
    // ── Input validation ────────────────────────────────────────────
    #[error("Invalid HS code format: '{0}' (expected 6-digit number, e.g. \"281511\")")]
    InvalidHsCode(String),

    #[error("Invalid CAS number format: '{0}' (expected digits-digits-digit, e.g. \"1310-73-2\")")]
    InvalidCasNumber(String),

    #[error("At least one identifier (cas, smiles, iupac_name, or inchi) must be provided")]
    MissingIdentifier,

    #[error("Mixture component #{index} is missing an identifier")]
    MissingComponentIdentifier { index: usize },

    // ── Session ─────────────────────────────────────────────────────
    #[error("No active question — call start() or answer() first")]
    NoActiveQuestion,

    #[error("Answer type mismatch: expected {expected}, got {got}")]
    AnswerTypeMismatch { expected: &'static str, got: &'static str },

    #[error("Choice index {index} out of range (valid range: 0..={max})")]
    InvalidChoiceIndex { index: usize, max: usize },

    #[error("Number {value} is out of range [{min}, {max}]")]
    NumberOutOfRange { value: f64, min: f64, max: f64 },

    #[error("Session is already complete — create a new ClassificationSession")]
    SessionAlreadyComplete,

    // ── Pipeline / classification ───────────────────────────────────
    #[error(
        "Confidence {confidence:.2} is below threshold {threshold:.2} \
         and no LLM client is configured"
    )]
    LowConfidenceNoLlm { confidence: f32, threshold: f32 },

    #[error("Mixture classification depth limit exceeded (max 2 levels)")]
    MixtureDepthExceeded,

    // ── LLM ────────────────────────────────────────────────────────
    #[error(
        "LLM client is not configured. \
         Enable the `llm` feature and supply an API key."
    )]
    LlmNotConfigured,

    #[error("LLM API error {status_code}: {message}")]
    LlmApiError { status_code: u16, message: String },

    #[error("Failed to parse LLM response as JSON: {source}\nRaw response: {raw}")]
    LlmResponseParseError {
        #[source]
        source: serde_json::Error,
        raw: String,
    },

    #[error("LLM rate limit exceeded — retry after {retry_after_secs}s")]
    LlmRateLimitError { retry_after_secs: u64 },

    // ── Validation ──────────────────────────────────────────────────
    #[error("LLM returned '{code}': not a valid 6-digit HS 2022 code")]
    ValidationFailed { code: String },

    #[error(
        "LLM code '{llm_code}' conflicts with rule-engine chapter '{expected_chapter}'"
    )]
    ChapterConflict { llm_code: String, expected_chapter: String },

    // ── PubChem ────────────────────────────────────────────────────
    #[cfg(feature = "pubchem")]
    #[error("PubChem error: {0}")]
    PubChem(#[from] crate::pubchem::PubChemError),

    // ── Generic ─────────────────────────────────────────────────────
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(String),
}

pub type Result<T> = std::result::Result<T, HsPredictError>;
