//! # hs-predict
//!
//! HS (Harmonized System) code prediction for chemical products.
//!
//! Uses an **Akinator-style interactive session** to collect just enough
//! information to classify the product, then applies a hybrid rule-based
//! and LLM prediction engine.
//!
//! ## Disclaimer
//! Predictions are advisory only and must not be used as the sole basis for
//! customs declarations. Always verify with a qualified trade-compliance expert.
//!
//! ## Quick start — interactive (Akinator-style)
//! ```rust,no_run
//! use hs_predict::session::{ClassificationSession, Answer, SessionResult};
//! use hs_predict::pipeline::HsPipeline;
//!
//! let mut session = ClassificationSession::new();
//! let pipeline = HsPipeline::new();
//!
//! // Q1: provide an identifier
//! let q = session.start();
//! println!("{}", q.prompt());
//!
//! // User answers with a CAS number; answer remaining questions the same way.
//! // When SessionResult::Ready is returned, call to_product_description().
//! let result = session.answer(Answer::Text("1310-73-2".to_string())).unwrap();
//! ```
//!
//! ## Quick start — direct (known CAS + physical form)
//! ```rust
//! use hs_predict::pipeline::HsPipeline;
//! use hs_predict::types::{ProductDescription, SubstanceIdentifier, PhysicalForm};
//!
//! let pipeline = HsPipeline::new();
//!
//! let product = ProductDescription {
//!     identifier: SubstanceIdentifier::from_cas("1310-73-2"),
//!     physical_form: Some(PhysicalForm::Solid),
//!     purity_pct: None,
//!     purity_type: None,
//!     mixture_components: None,
//!     intended_use: None,
//!     additional_context: None,
//! };
//!
//! let p = pipeline.classify(&product).unwrap();
//! assert_eq!(&p.hs_code, "281511");
//! assert_eq!(p.display(), "28.15.11");
//! ```

pub mod error;
pub mod pipeline;
pub mod rules;
pub mod session;
pub mod smiles;
pub mod types;

// Feature-gated modules
#[cfg(feature = "pubchem")]
pub mod pubchem;

#[cfg(feature = "pubchem")]
pub use pubchem::{PubChemClient, PubChemCompound};

#[cfg(feature = "llm")]
pub mod llm;

// Top-level convenience re-exports
pub use error::{HsPredictError, Result};
pub use pipeline::HsPipeline;
pub use session::{ClassificationSession, QuestionStep};
pub use types::{HsPrediction, Language, ProductDescription, SubstanceIdentifier};
