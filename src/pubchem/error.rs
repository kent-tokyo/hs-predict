use thiserror::Error;

/// Errors produced by the PubChem API client.
#[derive(Debug, Error)]
pub enum PubChemError {
    /// PubChem returned no compound matching the given input.
    #[error("No compound found in PubChem for: '{input}'")]
    NotFound { input: String },

    /// The `SubstanceIdentifier` has no field usable for a PubChem lookup.
    ///
    /// At least one of: CAS number, SMILES, InChIKey, InChI, or IUPAC name
    /// must be present.
    #[error("SubstanceIdentifier has no usable field for PubChem lookup (provide CAS, SMILES, InChIKey, InChI, or IUPAC name)")]
    NoUsableIdentifier,

    /// HTTP-level error (network failure, server error, etc.).
    #[error("PubChem HTTP error: {0}")]
    Http(String),

    /// The API response could not be parsed.
    #[error("Failed to parse PubChem response: {0}")]
    Parse(String),

    /// PubChem server returned a rate-limit response (HTTP 429).
    #[error("PubChem rate limit exceeded — retry after a few seconds")]
    RateLimitExceeded,
}
