//! Rule-based HS code classification engine.
//!
//! The engine uses a static CAS → HS mapping table ([`static_table`])
//! and a shape/purity matcher ([`matcher`]) to classify known compounds
//! without any LLM calls.
//!
//! Chapter 38 (miscellaneous chemical preparations) is handled by
//! [`chapter38`], which provides use-case-based classification for mixtures.

pub mod chapter38;
pub mod jp_table;
pub mod matcher;
pub mod static_table;
pub mod types;
