use serde::{Deserialize, Serialize};

/// Input for a single article to be enriched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleInput {
    pub number: String,
    pub text: String,
    pub url: String,
}

/// Context about the law being enriched, provided to the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawContext {
    pub law_id: String,
    pub name: String,
    pub regulatory_layer: String,
    pub bwb_id: Option<String>,
    pub url: String,
    pub publication_date: String,
    /// Other articles in the same law (for cross-reference context).
    pub other_articles: Vec<ArticleInput>,
    /// Known regulation IDs in the repository (for resolving references).
    pub known_regulations: Vec<String>,
}

/// Result of enriching a single article.
#[derive(Debug, Clone)]
pub struct ArticleEnrichmentResult {
    pub article_number: String,
    /// The generated machine_readable YAML string.
    pub machine_readable: String,
    /// How many generate/fix iterations were used.
    pub iterations_used: u32,
    /// Whether the result passed JSON schema validation.
    pub schema_valid: bool,
    /// Whether the result passed reverse validation.
    pub reverse_valid: bool,
    /// Warnings from the enrichment process.
    pub warnings: Vec<String>,
    /// Assumptions made during interpretation.
    pub assumptions: Vec<String>,
    /// Token usage for this article.
    pub token_usage: TokenUsage,
}

/// Result of enriching an entire law.
#[derive(Debug, Clone)]
pub struct LawEnrichmentResult {
    pub law_id: String,
    pub articles: Vec<ArticleEnrichmentResult>,
    pub total_articles: usize,
    pub enriched_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub token_usage: TokenUsage,
}

/// Feedback from validation used to build fix prompts.
#[derive(Debug, Clone, Default)]
pub struct ValidationFeedback {
    pub schema_errors: Vec<String>,
    pub reverse_validation_issues: Vec<String>,
}

/// Token usage tracking.
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl TokenUsage {
    pub fn add(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
    }
}
