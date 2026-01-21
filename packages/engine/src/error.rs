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

    /// Delegation error
    #[error("Delegation error: {0}")]
    DelegationError(String),

    /// Invalid date format
    #[error("Invalid date format: {0}")]
    InvalidDate(String),
}

/// Result type alias for engine operations
pub type Result<T> = std::result::Result<T, EngineError>;

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
        assert_eq!(err.to_string(), "Type mismatch: expected number, got string");
    }
}
