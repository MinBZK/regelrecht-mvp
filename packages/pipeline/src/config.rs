use crate::error::{PipelineError, Result};

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub database_url: String,
    pub max_connections: u32,
}

impl PipelineConfig {
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| PipelineError::Config("DATABASE_URL not set".into()))?;

        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        Ok(Self {
            database_url,
            max_connections,
        })
    }

    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            max_connections: 5,
        }
    }

    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }
}
