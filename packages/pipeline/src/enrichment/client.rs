use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

use crate::enrichment::config::EnrichmentConfig;
use crate::error::{PipelineError, Result};

/// Role of a message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// A single message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Request to the LLM.
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f64,
}

/// Response from the LLM.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Trait for LLM clients, enabling mocking in tests.
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse>;
}

/// Anthropic API client implementation.
///
/// NOTE: Do NOT derive `Debug` on this struct — `api_key` would be exposed.
/// If Debug is needed, implement it manually with the key redacted.
pub struct AnthropicClient {
    http: reqwest::Client,
    api_key: String,
    api_base_url: String,
    model: String,
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    temperature: f64,
    system: &'a str,
    messages: &'a [Message],
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u64,
    output_tokens: u64,
}

#[derive(Deserialize)]
struct AnthropicErrorResponse {
    error: Option<AnthropicErrorDetail>,
}

#[derive(Deserialize)]
struct AnthropicErrorDetail {
    message: String,
}

impl AnthropicClient {
    pub fn new(config: &EnrichmentConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(PipelineError::LlmApiRequest)?;

        Ok(Self {
            http,
            api_key: config.api_key.clone(),
            api_base_url: config.api_base_url.clone(),
            model: config.model.clone(),
        })
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/v1/messages", self.api_base_url);

        let body = AnthropicRequest {
            model: &self.model,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            system: &request.system,
            messages: &request.messages,
        };

        let base_delays = [
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(4),
        ];
        let max_attempts = base_delays.len() + 1;

        let mut last_error: Option<PipelineError> = None;
        let mut next_delay = Duration::ZERO;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                debug!(attempt, "retrying LLM request after {:?}", next_delay);
                tokio::time::sleep(next_delay).await;
            }

            // Reset to the base exponential delay for the next potential retry
            next_delay = base_delays.get(attempt).copied().unwrap_or(base_delays[base_delays.len() - 1]);

            let resp = self
                .http
                .post(&url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    warn!(attempt, error = %e, "LLM request failed");
                    last_error = Some(PipelineError::LlmApiRequest(e));
                    continue;
                }
            };

            let status = resp.status().as_u16();

            if status == 429 {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(60);
                warn!(attempt, retry_after, "LLM rate limited");
                // Use the server-provided retry-after, at least as long as the base delay
                next_delay = Duration::from_secs(retry_after).max(next_delay);
                last_error = Some(PipelineError::LlmRateLimited {
                    retry_after_secs: retry_after,
                });
                continue;
            }

            if status >= 500 {
                let body_text = resp.text().await.unwrap_or_default();
                warn!(attempt, status, body = %body_text, "LLM server error");
                last_error = Some(PipelineError::LlmApiError {
                    status,
                    message: body_text,
                });
                continue;
            }

            if status != 200 {
                let body_text = resp.text().await.unwrap_or_default();
                let message = serde_json::from_str::<AnthropicErrorResponse>(&body_text)
                    .ok()
                    .and_then(|r| r.error)
                    .map(|e| e.message)
                    .unwrap_or(body_text);
                return Err(PipelineError::LlmApiError { status, message });
            }

            let api_response: AnthropicResponse = resp
                .json()
                .await
                .map_err(|e| PipelineError::LlmResponseParse(e.to_string()))?;

            let content = api_response
                .content
                .into_iter()
                .filter_map(|block| block.text)
                .collect::<Vec<_>>()
                .join("");

            if content.is_empty() {
                warn!(attempt, "LLM returned empty response");
                last_error = Some(PipelineError::LlmEmptyResponse);
                continue;
            }

            return Ok(LlmResponse {
                content,
                input_tokens: api_response.usage.input_tokens,
                output_tokens: api_response.usage.output_tokens,
            });
        }

        Err(last_error.unwrap_or(PipelineError::LlmEmptyResponse))
    }
}

/// Test utilities for the LLM client.
#[cfg(any(test, feature = "test-utils"))]
pub mod test_support {
    use super::*;
    use std::sync::Mutex;

    /// Mock LLM client for testing. Returns pre-configured responses in order.
    pub struct MockLlmClient {
        responses: Mutex<Vec<Result<LlmResponse>>>,
    }

    impl MockLlmClient {
        pub fn new(responses: Vec<Result<LlmResponse>>) -> Self {
            // Reverse so we can pop from the end
            let mut responses = responses;
            responses.reverse();
            Self {
                responses: Mutex::new(responses),
            }
        }

        pub fn with_response(content: &str) -> Self {
            Self::new(vec![Ok(LlmResponse {
                content: content.to_string(),
                input_tokens: 100,
                output_tokens: 200,
            })])
        }

        pub fn with_responses(contents: Vec<&str>) -> Self {
            Self::new(
                contents
                    .into_iter()
                    .map(|c| {
                        Ok(LlmResponse {
                            content: c.to_string(),
                            input_tokens: 100,
                            output_tokens: 200,
                        })
                    })
                    .collect(),
            )
        }
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse> {
            let mut responses = self.responses.lock().map_err(|e| {
                PipelineError::LlmResponseParse(format!("mock lock poisoned: {e}"))
            })?;
            responses.pop().unwrap_or(Err(PipelineError::LlmEmptyResponse))
        }
    }
}
