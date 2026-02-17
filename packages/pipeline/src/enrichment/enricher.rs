use tracing::{debug, info, warn};

use crate::enrichment::client::{LlmClient, LlmRequest, Message, Role};
use crate::enrichment::config::EnrichmentConfig;
use crate::enrichment::prompt;
use crate::enrichment::reverse_validator::ReverseValidator;
use crate::enrichment::schema_validator::SchemaValidator;
use crate::enrichment::types::{
    ArticleEnrichmentResult, ArticleInput, LawContext, LawEnrichmentResult, TokenUsage,
    ValidationFeedback,
};
use crate::error::{PipelineError, Result};

/// Approximate context window limit (in tokens) for safety checks.
/// Set conservatively below Claude's actual limit to leave headroom.
const CONTEXT_TOKEN_LIMIT: u64 = 180_000;

/// Main enrichment orchestrator.
///
/// Generates machine_readable sections for law articles using an LLM,
/// validates them against the JSON schema, and optionally runs reverse
/// validation to check traceability.
pub struct Enricher<'a, C: LlmClient> {
    client: &'a C,
    config: &'a EnrichmentConfig,
    schema_validator: SchemaValidator,
}

impl<'a, C: LlmClient> Enricher<'a, C> {
    pub fn new(client: &'a C, config: &'a EnrichmentConfig) -> Result<Self> {
        let schema_validator = SchemaValidator::new()?;
        Ok(Self {
            client,
            config,
            schema_validator,
        })
    }

    /// Enrich a single article with a machine_readable section.
    pub async fn enrich_article(
        &self,
        article: &ArticleInput,
        context: &LawContext,
    ) -> Result<ArticleEnrichmentResult> {
        info!(article = article.number, law = context.law_id, "enriching article");

        let system = prompt::build_system_prompt().to_string();
        let user_prompt = prompt::build_enrichment_prompt(article, context);

        let mut messages = vec![Message {
            role: Role::User,
            content: user_prompt,
        }];

        let mut total_usage = TokenUsage::default();
        let mut warnings = Vec::new();
        let mut assumptions = Vec::new();
        let mut schema_valid = false;
        let mut reverse_valid = false;
        let mut last_yaml = String::new();
        let mut actual_iterations: u32 = 0;

        for iteration in 1..=self.config.max_fix_iterations {
            actual_iterations = iteration;
            debug!(article = article.number, iteration, "LLM request");

            let request = LlmRequest {
                system: system.clone(),
                messages: messages.clone(),
                max_tokens: self.config.max_tokens,
                temperature: self.config.temperature,
            };

            let response = self.client.complete(&request).await?;

            total_usage.add(&TokenUsage {
                input_tokens: response.input_tokens,
                output_tokens: response.output_tokens,
            });

            // Guard: if input tokens are approaching the context limit, stop iterating
            // to avoid sending a request that will fail due to context overflow.
            if response.input_tokens > CONTEXT_TOKEN_LIMIT {
                warn!(
                    article = article.number,
                    iteration,
                    input_tokens = response.input_tokens,
                    limit = CONTEXT_TOKEN_LIMIT,
                    "conversation approaching context window limit, stopping fix loop"
                );
                warnings.push(format!(
                    "Stopped after iteration {iteration}: input tokens ({}) approaching context limit",
                    response.input_tokens
                ));
                // Still try to use the response we got
                let yaml_str = extract_yaml_from_response(&response.content);
                last_yaml = yaml_str;
                break;
            }

            let yaml_str = extract_yaml_from_response(&response.content);
            last_yaml = yaml_str.clone();

            // Append assistant response to conversation for potential fix
            messages.push(Message {
                role: Role::Assistant,
                content: response.content,
            });

            // Convert YAML to JSON for schema validation
            let json_value = match yaml_to_json(&yaml_str) {
                Ok(v) => v,
                Err(e) => {
                    warn!(article = article.number, iteration, error = %e, "YAML parse failed");
                    let feedback = ValidationFeedback {
                        schema_errors: vec![format!("YAML parse error: {e}")],
                        ..Default::default()
                    };
                    let fix_prompt = prompt::build_fix_prompt(&yaml_str, &feedback);
                    messages.push(Message {
                        role: Role::User,
                        content: fix_prompt,
                    });
                    continue;
                }
            };

            // Schema validation
            match self.schema_validator.validate(&json_value) {
                Ok(()) => {
                    debug!(article = article.number, iteration, "schema validation passed");
                    schema_valid = true;
                }
                Err(PipelineError::SchemaValidation { errors }) => {
                    warn!(
                        article = article.number,
                        iteration,
                        error_count = errors.len(),
                        "schema validation failed"
                    );
                    let feedback = ValidationFeedback {
                        schema_errors: errors,
                        ..Default::default()
                    };
                    let fix_prompt = prompt::build_fix_prompt(&yaml_str, &feedback);
                    messages.push(Message {
                        role: Role::User,
                        content: fix_prompt,
                    });
                    continue;
                }
                Err(e) => return Err(e),
            }

            // Reverse validation
            let reverse = ReverseValidator::new(self.client, self.config);
            let rv_result = reverse.validate(article, &yaml_str).await?;
            total_usage.add(&rv_result.token_usage);

            if rv_result.is_valid {
                debug!(article = article.number, iteration, "reverse validation passed");
                reverse_valid = true;
                break;
            }

            // Check if we have room for another iteration
            if iteration == self.config.max_fix_iterations {
                warnings.push(format!(
                    "Reverse validation issues remain after {} iterations",
                    self.config.max_fix_iterations
                ));
                for issue in &rv_result.issues {
                    if issue.contains("ASSUMPTION") {
                        assumptions.push(issue.clone());
                    } else {
                        warnings.push(issue.clone());
                    }
                }
                // Still return the best result we have
                break;
            }

            warn!(
                article = article.number,
                iteration,
                issue_count = rv_result.issues.len(),
                "reverse validation found issues"
            );

            let feedback = ValidationFeedback {
                reverse_validation_issues: rv_result.issues,
                ..Default::default()
            };
            let fix_prompt = prompt::build_fix_prompt(&yaml_str, &feedback);
            messages.push(Message {
                role: Role::User,
                content: fix_prompt,
            });
        }

        if !schema_valid {
            return Err(PipelineError::MaxIterationsExceeded {
                iterations: self.config.max_fix_iterations,
            });
        }

        Ok(ArticleEnrichmentResult {
            article_number: article.number.clone(),
            machine_readable: last_yaml,
            iterations_used: actual_iterations,
            schema_valid,
            reverse_valid,
            warnings,
            assumptions,
            token_usage: total_usage,
        })
    }

    /// Enrich all articles in a law sequentially.
    pub async fn enrich_law(
        &self,
        articles: &[ArticleInput],
        context: &LawContext,
    ) -> LawEnrichmentResult {
        let mut results = Vec::new();
        let mut total_usage = TokenUsage::default();
        let mut enriched = 0;
        let mut skipped = 0;
        let mut failed = 0;

        for article in articles {
            match self.enrich_article(article, context).await {
                Ok(result) => {
                    total_usage.add(&result.token_usage);
                    if result.machine_readable.trim().is_empty()
                        || result.machine_readable.trim() == "machine_readable:"
                    {
                        skipped += 1;
                    } else {
                        enriched += 1;
                    }
                    results.push(result);
                }
                Err(e) => {
                    warn!(article = article.number, error = %e, "article enrichment failed");
                    failed += 1;
                    results.push(ArticleEnrichmentResult {
                        article_number: article.number.clone(),
                        machine_readable: String::new(),
                        iterations_used: 0,
                        schema_valid: false,
                        reverse_valid: false,
                        warnings: vec![format!("Enrichment failed: {e}")],
                        assumptions: vec![],
                        token_usage: TokenUsage::default(),
                    });
                }
            }
        }

        LawEnrichmentResult {
            law_id: context.law_id.clone(),
            articles: results,
            total_articles: articles.len(),
            enriched_count: enriched,
            skipped_count: skipped,
            failed_count: failed,
            token_usage: total_usage,
        }
    }
}

/// Extract YAML content from an LLM response, stripping markdown fences if present.
///
/// When multiple fenced blocks exist, prefers the one containing `machine_readable:`.
/// Falls back to the last fenced block, since LLMs often put explanatory blocks first.
pub fn extract_yaml_from_response(response: &str) -> String {
    let trimmed = response.trim();

    // Collect all fenced code blocks
    let blocks = extract_fenced_blocks(trimmed);

    if !blocks.is_empty() {
        // Prefer a block that contains "machine_readable:"
        if let Some(block) = blocks.iter().find(|b| b.contains("machine_readable:")) {
            return block.trim().to_string();
        }
        // Otherwise use the last block (LLMs often put explanations first)
        // Safety: blocks is non-empty (checked above), but use if-let to satisfy clippy
        if let Some(block) = blocks.last() {
            return block.trim().to_string();
        }
    }

    // No fences found, return as-is
    trimmed.to_string()
}

/// Extract all fenced code blocks from text.
fn extract_fenced_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("```") {
        let after_fence = &remaining[start + 3..];
        // Skip optional language identifier on the same line
        let content_start = after_fence.find('\n').map(|i| i + 1).unwrap_or(0);
        let content = &after_fence[content_start..];
        if let Some(end) = content.find("```") {
            blocks.push(content[..end].to_string());
            remaining = &content[end + 3..];
        } else {
            break;
        }
    }

    blocks
}

/// Convert a YAML string to a JSON value for schema validation.
///
/// If the YAML starts with `machine_readable:`, unwraps to get the inner value.
pub fn yaml_to_json(yaml_str: &str) -> Result<serde_json::Value> {
    let value: serde_json::Value = serde_yaml_ng::from_str(yaml_str)
        .map_err(|e| PipelineError::YamlParse(e.to_string()))?;

    // If the top-level is { "machine_readable": { ... } }, unwrap it
    if let Some(inner) = value.get("machine_readable") {
        Ok(inner.clone())
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_yaml_no_fences() {
        let input = "machine_readable:\n  execution:\n    output:\n      - name: test\n        type: boolean";
        assert_eq!(extract_yaml_from_response(input), input.trim());
    }

    #[test]
    fn test_extract_yaml_with_fences() {
        let input = "```yaml\nmachine_readable:\n  execution: {}\n```";
        assert_eq!(
            extract_yaml_from_response(input),
            "machine_readable:\n  execution: {}"
        );
    }

    #[test]
    fn test_extract_yaml_with_surrounding_text() {
        let input = "Here is the YAML:\n```yaml\nmachine_readable:\n  execution: {}\n```\nDone!";
        assert_eq!(
            extract_yaml_from_response(input),
            "machine_readable:\n  execution: {}"
        );
    }

    #[test]
    fn test_yaml_to_json_with_wrapper() {
        let yaml = "machine_readable:\n  execution:\n    output:\n      - name: test\n        type: boolean";
        let result = yaml_to_json(yaml).expect("should parse");
        assert!(result.get("execution").is_some());
    }

    #[test]
    fn test_yaml_to_json_without_wrapper() {
        let yaml = "execution:\n  output:\n    - name: test\n      type: boolean";
        let result = yaml_to_json(yaml).expect("should parse");
        assert!(result.get("execution").is_some());
    }

    #[test]
    fn test_yaml_to_json_invalid() {
        let yaml = "not: valid: yaml: {{{}}}";
        assert!(yaml_to_json(yaml).is_err());
    }
}
