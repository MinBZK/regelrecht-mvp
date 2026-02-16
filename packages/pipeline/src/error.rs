use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("job not found: {0}")]
    JobNotFound(uuid::Uuid),

    #[error("law not found: {0}")]
    LawNotFound(String),

    #[error("invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, PipelineError>;
