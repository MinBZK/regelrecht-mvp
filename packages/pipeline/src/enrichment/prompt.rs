use crate::enrichment::types::{ArticleInput, LawContext, ValidationFeedback};

const SYSTEM_ENRICHMENT: &str = include_str!("../../prompts/system_enrichment.txt");
const SYSTEM_REVERSE_VALIDATION: &str = include_str!("../../prompts/system_reverse_validation.txt");

/// Build the system prompt for enrichment.
pub fn build_system_prompt() -> &'static str {
    SYSTEM_ENRICHMENT
}

/// Build the user prompt for enriching a single article.
pub fn build_enrichment_prompt(article: &ArticleInput, context: &LawContext) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!("# Law: {} ({})\n", context.name, context.law_id));
    prompt.push_str(&format!(
        "- Regulatory layer: {}\n",
        context.regulatory_layer
    ));
    if let Some(ref bwb_id) = context.bwb_id {
        prompt.push_str(&format!("- BWB ID: {bwb_id}\n"));
    }
    prompt.push_str(&format!("- URL: {}\n", context.url));
    prompt.push_str(&format!(
        "- Publication date: {}\n\n",
        context.publication_date
    ));

    // Other articles for cross-reference context
    if !context.other_articles.is_empty() {
        prompt.push_str("## Other articles in this law (for reference):\n\n");
        for other in &context.other_articles {
            if other.number != article.number {
                prompt.push_str(&format!(
                    "### Artikel {}\n{}\n\n",
                    other.number, other.text
                ));
            }
        }
    }

    // Known regulations
    if !context.known_regulations.is_empty() {
        prompt.push_str("## Known regulations in the repository:\n");
        for reg in &context.known_regulations {
            prompt.push_str(&format!("- {reg}\n"));
        }
        prompt.push('\n');
    }

    // The target article
    prompt.push_str(&format!(
        "# Article to interpret\n\n## Artikel {}\n\nURL: {}\n\n{}\n\n",
        article.number, article.url, article.text
    ));

    prompt.push_str(
        "Generate the complete `machine_readable` YAML section for this article. \
         Return ONLY the YAML content, starting with `machine_readable:`. \
         No markdown fences or explanations.",
    );

    prompt
}

/// Build a fix prompt with validation feedback.
pub fn build_fix_prompt(
    original_yaml: &str,
    feedback: &ValidationFeedback,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("The previous machine_readable output had validation errors. Please fix them.\n\n");

    prompt.push_str("## Previous output:\n```yaml\n");
    prompt.push_str(original_yaml);
    prompt.push_str("\n```\n\n");

    if !feedback.schema_errors.is_empty() {
        prompt.push_str("## Schema validation errors:\n");
        for error in &feedback.schema_errors {
            prompt.push_str(&format!("- {error}\n"));
        }
        prompt.push('\n');
    }

    if !feedback.reverse_validation_issues.is_empty() {
        prompt.push_str("## Reverse validation issues:\n");
        for issue in &feedback.reverse_validation_issues {
            prompt.push_str(&format!("- {issue}\n"));
        }
        prompt.push('\n');
    }

    prompt.push_str(
        "Fix ALL listed errors and return the corrected YAML. \
         Return ONLY the YAML content, starting with `machine_readable:`. \
         No markdown fences or explanations.",
    );

    prompt
}

/// Build the system prompt for reverse validation.
pub fn build_reverse_validation_system_prompt() -> &'static str {
    SYSTEM_REVERSE_VALIDATION
}

/// Build the user prompt for reverse validation.
pub fn build_reverse_validation_prompt(article: &ArticleInput, machine_readable_yaml: &str) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!(
        "# Artikel {}\n\n## Original legal text:\n{}\n\n",
        article.number, article.text
    ));

    prompt.push_str("## Generated machine_readable section:\n```yaml\n");
    prompt.push_str(machine_readable_yaml);
    prompt.push_str("\n```\n\n");

    prompt.push_str(
        "Validate that every element in the machine_readable section \
         is traceable to the original legal text. \
         Return VALID if everything checks out, or list the issues.",
    );

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompts_not_empty() {
        assert!(!build_system_prompt().is_empty());
        assert!(!build_reverse_validation_system_prompt().is_empty());
    }

    #[test]
    fn test_enrichment_prompt_includes_article() {
        let article = ArticleInput {
            number: "2".into(),
            text: "Een persoon heeft recht op zorgtoeslag.".into(),
            url: "https://example.com/art2".into(),
        };
        let context = LawContext {
            law_id: "zorgtoeslagwet".into(),
            name: "Wet op de zorgtoeslag".into(),
            regulatory_layer: "WET".into(),
            bwb_id: Some("BWBR0018451".into()),
            url: "https://example.com".into(),
            publication_date: "2005-01-01".into(),
            other_articles: vec![],
            known_regulations: vec!["awir".into()],
        };

        let prompt = build_enrichment_prompt(&article, &context);
        assert!(prompt.contains("Artikel 2"));
        assert!(prompt.contains("zorgtoeslag"));
        assert!(prompt.contains("awir"));
    }

    #[test]
    fn test_fix_prompt_includes_errors() {
        let feedback = ValidationFeedback {
            schema_errors: vec!["missing required field: output".into()],
            reverse_validation_issues: vec![],
        };
        let prompt = build_fix_prompt("machine_readable:\n  execution: {}", &feedback);
        assert!(prompt.contains("missing required field: output"));
    }
}
