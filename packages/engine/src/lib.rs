//! RegelRecht Engine
//!
//! A Rust implementation of the RegelRecht law execution engine.
//! This library provides functionality for:
//! - Loading and parsing article-based law specifications (YAML)
//! - Executing law logic with variable resolution
//! - Cross-law references and delegation handling
//!
//! # Example
//!
//! ```ignore
//! use regelrecht_engine::{LawExecutionService, Value};
//! use std::collections::HashMap;
//!
//! let service = LawExecutionService::new("./regulations")?;
//! let mut params = HashMap::new();
//! params.insert("BSN".to_string(), Value::String("123456789".to_string()));
//!
//! let result = service.evaluate_law_output(
//!     "zorgtoeslagwet",
//!     "heeft_recht_op_zorgtoeslag",
//!     params,
//!     Some("2024-01-01"),
//! )?;
//! ```

pub mod article;
pub mod context;
pub mod engine;
pub mod error;
pub mod operations;
pub mod types;
pub mod uri;

#[cfg(feature = "wasm")]
pub mod wasm;

// Re-export commonly used items
pub use article::{
    Action, ActionOperation, ActionValue, Article, ArticleBasedLaw, Delegation, Execution,
    MachineReadable, SelectOnCriteria, Source, SwitchCase,
};
pub use context::RuleContext;
pub use engine::{
    evaluate_select_on_criteria, get_delegation_info, matches_delegation_criteria, ArticleEngine,
    ArticleResult,
};
pub use error::{EngineError, Result};
pub use operations::{evaluate_value, execute_operation, ValueResolver};
pub use types::{Operation, ParameterType, PathNodeType, RegulatoryLayer, ResolveType, Value};
pub use uri::{internal_reference, ReferenceType, RegelrechtUri, RegelrechtUriBuilder};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0");
    }

    #[test]
    fn test_reexports() {
        // Verify re-exports work
        let _val = Value::Int(42);
        let _op = Operation::Equals;
        let _err = EngineError::DivisionByZero;
    }
}
