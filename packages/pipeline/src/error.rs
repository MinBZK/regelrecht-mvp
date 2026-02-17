use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("job not found: {0}")]
    JobNotFound(uuid::Uuid),

    #[error("job {0} is not in processing state")]
    JobNotProcessing(uuid::Uuid),

    #[error("law not found: {0}")]
    LawNotFound(String),

    #[error("invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("LLM API request failed: {0}")]
    LlmApiRequest(#[from] reqwest::Error),

    #[error("LLM API error (status {status}): {message}")]
    LlmApiError { status: u16, message: String },

    #[error("LLM rate limited, retry after {retry_after_secs}s")]
    LlmRateLimited { retry_after_secs: u64 },

    #[error("failed to parse LLM response: {0}")]
    LlmResponseParse(String),

    #[error("schema validation failed: {}", errors.join(", "))]
    SchemaValidation { errors: Vec<String> },

    #[error("YAML parse error: {0}")]
    YamlParse(String),

    #[error("schema load error: {0}")]
    SchemaLoad(String),

    #[error("max fix iterations ({iterations}) exceeded")]
    MaxIterationsExceeded { iterations: u32 },

    #[error("LLM returned empty response")]
    LlmEmptyResponse,
}

pub type Result<T> = std::result::Result<T, PipelineError>;
