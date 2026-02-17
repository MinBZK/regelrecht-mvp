mod client;
mod config;
mod enricher;
mod prompt;
mod reverse_validator;
mod schema_validator;
mod types;

pub use client::{AnthropicClient, LlmClient, LlmRequest, LlmResponse, Message, Role};
#[cfg(any(test, feature = "test-utils"))]
pub use client::test_support::MockLlmClient;
pub use config::EnrichmentConfig;
pub use enricher::{extract_yaml_from_response, yaml_to_json, Enricher};
pub use reverse_validator::ReverseValidator;
pub use schema_validator::SchemaValidator;
pub use types::{
    ArticleEnrichmentResult, ArticleInput, LawContext, LawEnrichmentResult, TokenUsage,
    ValidationFeedback,
};
