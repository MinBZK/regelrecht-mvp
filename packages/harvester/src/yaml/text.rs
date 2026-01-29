//! Text wrapping utilities for YAML output.

use textwrap::{fill, Options};

use crate::config::TEXT_WRAP_WIDTH;

/// Wrap text at specified width, preserving paragraph breaks and reference definitions.
///
/// Reference definitions (lines starting with [refN]:) are preserved as-is
/// to maintain valid markdown reference-style links.
pub fn wrap_text(text: &str, width: usize) -> String {
    // Separate reference definitions from main text
    let lines: Vec<&str> = text.lines().collect();
    let mut ref_lines: Vec<&str> = Vec::new();
    let mut content_lines: Vec<&str> = Vec::new();

    // Find where reference definitions start (from end)
    let mut in_refs = false;
    for line in lines.iter().rev() {
        if line.starts_with("[ref") && line.contains("]: ") {
            ref_lines.insert(0, line);
            in_refs = true;
        } else if in_refs && line.is_empty() {
            ref_lines.insert(0, line);
        } else {
            in_refs = false;
            content_lines.insert(0, line);
        }
    }

    // Wrap content paragraphs
    let content_text = content_lines.join("\n");
    let paragraphs: Vec<&str> = content_text.split("\n\n").collect();

    let options = Options::new(width);
    let wrapped: Vec<String> = paragraphs.iter().map(|p| fill(p, &options)).collect();

    let wrapped_content = wrapped.join("\n\n");

    // Append reference definitions unchanged
    if !ref_lines.is_empty() {
        format!("{}\n\n{}", wrapped_content, ref_lines.join("\n"))
    } else {
        wrapped_content
    }
}

/// Check if text should be wrapped for readability.
pub fn should_wrap_text(text: &str) -> bool {
    let has_markdown_links = text.contains('[') && text.contains("](");
    text.len() > 80 || has_markdown_links
}

/// Wrap text with default width.
pub fn wrap_text_default(text: &str) -> String {
    wrap_text(text, TEXT_WRAP_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text_simple() {
        let text = "This is a simple text that should be wrapped when it exceeds the specified width limit.";
        let wrapped = wrap_text(text, 40);
        assert!(wrapped.contains('\n'));
    }

    #[test]
    fn test_wrap_text_preserves_paragraphs() {
        let text = "First paragraph.\n\nSecond paragraph.";
        let wrapped = wrap_text(text, 100);
        assert!(wrapped.contains("\n\n"));
    }

    #[test]
    fn test_wrap_text_preserves_references() {
        let text = "Some text with a [link][ref1].\n\n[ref1]: https://example.com";
        let wrapped = wrap_text(text, 100);
        assert!(wrapped.contains("[ref1]: https://example.com"));
    }

    #[test]
    fn test_should_wrap_text_long() {
        let long_text = "A".repeat(100);
        assert!(should_wrap_text(&long_text));
    }

    #[test]
    fn test_should_wrap_text_short() {
        let short_text = "Short text";
        assert!(!should_wrap_text(short_text));
    }

    #[test]
    fn test_should_wrap_text_with_links() {
        let text = "Text with [link](url)";
        assert!(should_wrap_text(text));
    }
}
