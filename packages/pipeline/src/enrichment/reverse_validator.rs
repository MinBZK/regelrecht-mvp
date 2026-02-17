use tracing::debug;

use crate::enrichment::client::{LlmClient, LlmRequest, Message, Role};
use crate::enrichment::config::EnrichmentConfig;
use crate::enrichment::prompt;
use crate::enrichment::types::{ArticleInput, TokenUsage};
use crate::error::Result;

/// Result of reverse validation.
#[derive(Debug, Clone)]
pub struct ReverseValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub token_usage: TokenUsage,
}

/// LLM-based reverse validator that checks if generated machine_readable
/// sections are traceable to the original legal text.
pub struct ReverseValidator<'a, C: LlmClient> {
    client: &'a C,
    config: &'a EnrichmentConfig,
}

impl<'a, C: LlmClient> ReverseValidator<'a, C> {
    pub fn new(client: &'a C, config: &'a EnrichmentConfig) -> Self {
        Self { client, config }
    }

    /// Validate that a machine_readable section is traceable to the article text.
    pub async fn validate(
        &self,
        article: &ArticleInput,
        machine_readable_yaml: &str,
    ) -> Result<ReverseValidationResult> {
        let system = prompt::build_reverse_validation_system_prompt().to_string();
        let user_prompt = prompt::build_reverse_validation_prompt(article, machine_readable_yaml);

        let request = LlmRequest {
            system,
            messages: vec![Message {
                role: Role::User,
                content: user_prompt,
            }],
            max_tokens: self.config.max_tokens,
            temperature: 0.0,
        };

        let response = self.client.complete(&request).await?;

        let token_usage = TokenUsage {
            input_tokens: response.input_tokens,
            output_tokens: response.output_tokens,
        };

        let content = response.content.trim();

        let preview: String = content.chars().take(200).collect();
        debug!(
            article = article.number,
            response_preview = preview.as_str(),
            "reverse validation response"
        );

        if content.starts_with("VALID") {
            return Ok(ReverseValidationResult {
                is_valid: true,
                issues: vec![],
                token_usage,
            });
        }

        // Parse issues from the response
        let issues = parse_issues(content);

        Ok(ReverseValidationResult {
            is_valid: issues.is_empty(),
            issues,
            token_usage,
        })
    }
}

/// Parse issues from the LLM reverse validation response.
fn parse_issues(content: &str) -> Vec<String> {
    // If the response starts with VALID, no issues
    if content.trim().starts_with("VALID") {
        return vec![];
    }

    let mut issues = Vec::new();
    let mut current_issue = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip header lines and empty lines
        if trimmed == "ISSUES:" || trimmed.is_empty() {
            if !current_issue.is_empty() {
                issues.push(current_issue.trim().to_string());
                current_issue.clear();
            }
            continue;
        }

        // New issue starts with "- type:"
        if trimmed.starts_with("- type:") {
            if !current_issue.is_empty() {
                issues.push(current_issue.trim().to_string());
            }
            current_issue = trimmed.to_string();
        } else if trimmed.starts_with("element:") || trimmed.starts_with("description:") || trimmed.starts_with("reason:") {
            current_issue.push_str(" | ");
            current_issue.push_str(trimmed);
        } else {
            // Continuation of previous line
            current_issue.push(' ');
            current_issue.push_str(trimmed);
        }
    }

    if !current_issue.is_empty() {
        issues.push(current_issue.trim().to_string());
    }

    // Fail-safe: if response is non-empty, non-VALID, but yielded no parsed issues,
    // treat the raw content as a single unparsed issue rather than silently accepting.
    if issues.is_empty() && !content.trim().is_empty() {
        issues.push(content.trim().to_string());
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_issues_valid() {
        let issues = parse_issues("VALID");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_parse_issues_with_issues() {
        let content = r#"ISSUES:
- type: REMOVE
  element: execution.actions[0]
  description: not traceable to text
  reason: invented condition"#;

        let issues = parse_issues(content);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("REMOVE"));
        assert!(issues[0].contains("execution.actions[0]"));
    }

    #[test]
    fn test_parse_issues_multiple() {
        let content = r#"ISSUES:
- type: REMOVE
  element: execution.input[0]
  description: field not referenced
  reason: not in text
- type: ASSUMPTION
  element: execution.parameters[0]
  description: bsn implied
  reason: needed for lookup"#;

        let issues = parse_issues(content);
        assert_eq!(issues.len(), 2);
    }
}
