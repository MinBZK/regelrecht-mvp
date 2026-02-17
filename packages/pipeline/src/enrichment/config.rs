use crate::error::{PipelineError, Result};

/// Configuration for LLM-based enrichment.
#[derive(Debug, Clone)]
pub struct EnrichmentConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub temperature: f64,
    pub max_fix_iterations: u32,
    pub api_base_url: String,
    pub max_tokens: u32,
    pub timeout_secs: u64,
}

impl EnrichmentConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("LLM_API_KEY")
            .map_err(|_| PipelineError::Config("LLM_API_KEY not set".into()))?;

        let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "anthropic".into());

        let model = std::env::var("LLM_MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-5-20250929".into());

        let temperature = std::env::var("LLM_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);

        let max_fix_iterations = std::env::var("LLM_MAX_FIX_ITERATIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        let api_base_url = std::env::var("LLM_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".into());

        let max_tokens = std::env::var("LLM_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8192);

        let timeout_secs = std::env::var("LLM_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(120);

        Ok(Self {
            provider,
            model,
            api_key,
            temperature,
            max_fix_iterations,
            api_base_url,
            max_tokens,
            timeout_secs,
        })
    }

    /// Create a config builder for testing.
    pub fn builder(api_key: impl Into<String>) -> EnrichmentConfigBuilder {
        EnrichmentConfigBuilder {
            api_key: api_key.into(),
            provider: "anthropic".into(),
            model: "claude-sonnet-4-5-20250929".into(),
            temperature: 0.0,
            max_fix_iterations: 3,
            api_base_url: "https://api.anthropic.com".into(),
            max_tokens: 8192,
            timeout_secs: 120,
        }
    }
}

/// Builder for constructing `EnrichmentConfig` in tests.
pub struct EnrichmentConfigBuilder {
    api_key: String,
    provider: String,
    model: String,
    temperature: f64,
    max_fix_iterations: u32,
    api_base_url: String,
    max_tokens: u32,
    timeout_secs: u64,
}

impl EnrichmentConfigBuilder {
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = provider.into();
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn max_fix_iterations(mut self, max_fix_iterations: u32) -> Self {
        self.max_fix_iterations = max_fix_iterations;
        self
    }

    pub fn api_base_url(mut self, api_base_url: impl Into<String>) -> Self {
        self.api_base_url = api_base_url.into();
        self
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn build(self) -> EnrichmentConfig {
        EnrichmentConfig {
            provider: self.provider,
            model: self.model,
            api_key: self.api_key,
            temperature: self.temperature,
            max_fix_iterations: self.max_fix_iterations,
            api_base_url: self.api_base_url,
            max_tokens: self.max_tokens,
            timeout_secs: self.timeout_secs,
        }
    }
}
