//! Rule-based HS code classification engine.
//!
//! The engine uses a static CAS → HS mapping table ([`static_table`])
//! and a shape/purity matcher ([`matcher`]) to classify known compounds
//! without any LLM calls.

pub mod jp_table;
pub mod matcher;
pub mod static_table;
pub mod types;
