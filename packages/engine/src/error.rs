//! Error types for the RegelRecht engine

use thiserror::Error;

/// Main error type for engine operations
#[derive(Error, Debug)]
pub enum EngineError {
    /// Failed to load or parse a law file
    #[error("Failed to load law: {0}")]
    LoadError(String),

    /// YAML parsing error
    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// IO error (file operations)
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Variable not found during resolution
    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Type mismatch during operation
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Invalid URI format
    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    /// Law not found
    #[error("Law not found: {0}")]
    LawNotFound(String),

    /// Article not found
    #[error("Article not found: {law_id}#{article}")]
    ArticleNotFound { law_id: String, article: String },

    /// Output not found in article
    #[error("Output '{output}' not found in law '{law_id}'")]
    OutputNotFound { law_id: String, output: String },

    /// Circular reference detected
    #[error("Circular reference detected: {0}")]
    CircularReference(String),

    /// Required parameter missing
    #[error("Required parameter missing: {0}")]
    MissingParameter(String),

    /// Arithmetic overflow when converting f64 to i64
    #[error("Arithmetic overflow: {0}")]
    ArithmeticOverflow(String),

    /// Maximum operation nesting depth exceeded
    #[error("Maximum operation depth exceeded: {0} levels")]
    MaxDepthExceeded(usize),

    /// Delegation error (generic)
    #[error("Delegation error: {0}")]
    DelegationError(String),

    /// Delegation not resolved - requires ServiceProvider
    #[error(
        "Delegation not resolved: input '{input_name}' requires delegation lookup \
         (law_id: {law_id}, article: {article}, select_on: [{select_on}]). \
         Pass the value as a parameter or implement ServiceProvider."
    )]
    DelegationNotResolved {
        input_name: String,
        law_id: String,
        article: String,
        select_on: String,
    },

    /// Invalid date format
    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    /// External reference not resolved - requires ServiceProvider
    #[error(
        "External reference not resolved: input '{input_name}' requires resolution from \
         regulation '{regulation}' output '{output}'. \
         Pass the value as a parameter or use ServiceProvider."
    )]
    ExternalReferenceNotResolved {
        input_name: String,
        regulation: String,
        output: String,
    },
}

/// Result type alias for engine operations
pub type Result<T> = std::result::Result<T, EngineError>;

// WASM error conversion
#[cfg(feature = "wasm")]
impl From<EngineError> for wasm_bindgen::JsValue {
    fn from(err: EngineError) -> Self {
        wasm_bindgen::JsValue::from_str(&err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EngineError::VariableNotFound("test_var".to_string());
        assert_eq!(err.to_string(), "Variable not found: test_var");
    }

    #[test]
    fn test_type_mismatch_display() {
        let err = EngineError::TypeMismatch {
            expected: "number".to_string(),
            actual: "string".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Type mismatch: expected number, got string"
        );
    }
}
