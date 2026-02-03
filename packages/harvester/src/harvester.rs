//! Main harvester service that ties all components together.

use roxmltree::Document;

use crate::config::{validate_bwb_id, validate_date, wetten_url};
use crate::content::download_content_xml;
use crate::error::Result;
use crate::http::create_client;
use crate::splitting::{create_dutch_law_hierarchy, LeafSplitStrategy, SplitContext, SplitEngine};
use crate::types::{Article, Law};
use crate::wti::download_wti;
use crate::xml::{find_bijlage_context, find_by_path, find_children, get_tag_name, get_text};

/// Download and parse a Dutch law.
///
/// # Arguments
/// * `bwb_id` - The BWB identifier (e.g., "BWBR0018451")
/// * `date` - The effective date in YYYY-MM-DD format
///
/// # Returns
/// A `Law` object containing metadata, articles, and any warnings encountered during parsing
pub fn download_law(bwb_id: &str, date: &str) -> Result<Law> {
    // Validate inputs
    validate_bwb_id(bwb_id)?;
    validate_date(date)?;

    // Create HTTP client
    let client = create_client()?;

    // Download and parse WTI metadata
    let metadata = download_wti(&client, bwb_id)?;

    // Download content XML
    let content_xml = download_content_xml(&client, bwb_id, date)?;

    // Parse articles from content
    let (articles, warnings) = parse_articles(&content_xml, bwb_id, date)?;

    Ok(Law {
        metadata,
        articles,
        warnings,
    })
}

/// Parse articles from content XML.
///
/// Returns `(articles, warnings)` tuple where warnings are non-fatal parse errors.
fn parse_articles(xml: &str, bwb_id: &str, date: &str) -> Result<(Vec<Article>, Vec<String>)> {
    let doc = Document::parse(xml)?;
    let mut articles = Vec::new();
    let mut all_warnings: Vec<String> = Vec::new();

    // Extract aanhef first
    if let Some(aanhef) = extract_aanhef(&doc, bwb_id, date) {
        articles.push(aanhef);
    }

    // Create split engine
    let hierarchy = create_dutch_law_hierarchy();
    let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

    // Find all artikel elements
    for artikel in doc
        .descendants()
        .filter(|n| n.is_element() && get_tag_name(*n) == "artikel")
    {
        // Get article number
        let artikel_nr = if let Some(nr_node) = find_by_path(artikel, "kop/nr") {
            get_text(nr_node)
        } else if let Some(label) = artikel.attribute("label") {
            label.strip_prefix("Artikel ").unwrap_or(label).to_string()
        } else {
            continue; // Skip articles without number
        };

        // Detect bijlage context
        let bijlage_context = find_bijlage_context(artikel);

        // Build base URL
        let artikel_nr_url = artikel_nr.replace(' ', "_");
        let base_url = wetten_url(bwb_id, Some(date), Some(&artikel_nr_url), None, None, None);

        // Create split context with bijlage prefix if applicable
        let mut context = SplitContext::new(bwb_id, date, base_url);
        if let Some(ctx) = bijlage_context {
            context = context.with_bijlage_prefix(format!("B{}", ctx.number));
        }

        // Split the artikel
        let components = engine.split(artikel, context);

        // Convert components to articles and collect warnings
        for component in components {
            // Collect warnings with article context
            for warning in &component.warnings {
                all_warnings.push(format!("Article {}: {}", component.to_number(), warning));
            }
            articles.push(component.to_article());
        }
    }

    Ok((articles, all_warnings))
}

/// Extract the aanhef (preamble) as an article.
fn extract_aanhef(doc: &Document<'_>, bwb_id: &str, date: &str) -> Option<Article> {
    let aanhef = doc
        .descendants()
        .find(|n| n.is_element() && get_tag_name(*n) == "aanhef")?;

    let mut parts: Vec<String> = Vec::new();

    // Extract <wij> element
    if let Some(wij) = find_children(aanhef, "wij").next() {
        if let Some(text) = wij.text() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                parts.push(trimmed.to_string());
            }
        }
    }

    // Extract <considerans> elements
    if let Some(considerans) = find_children(aanhef, "considerans").next() {
        for al in considerans
            .descendants()
            .filter(|n| n.is_element() && get_tag_name(*n) == "considerans.al")
        {
            let text = extract_simple_text(al);
            if !text.is_empty() {
                parts.push(text);
            }
        }
    }

    // Extract <afkondiging> element
    if let Some(afkondiging) = find_children(aanhef, "afkondiging").next() {
        for al in find_children(afkondiging, "al") {
            let text = extract_simple_text(al);
            if !text.is_empty() {
                parts.push(text);
            }
        }
    }

    if parts.is_empty() {
        return None;
    }

    let aanhef_text = parts.join("\n\n");
    let aanhef_url = wetten_url(bwb_id, Some(date), Some("Aanhef"), None, None, None);

    Some(Article {
        number: "aanhef".to_string(),
        text: aanhef_text,
        url: aanhef_url,
        references: Vec::new(), // Aanhef typically has no cross-law references
    })
}

/// Simple text extraction from a node.
fn extract_simple_text(node: roxmltree::Node<'_, '_>) -> String {
    let mut text = String::new();

    if let Some(t) = node.text() {
        text.push_str(t);
    }

    for child in node.children() {
        if child.is_element() {
            text.push_str(&extract_simple_text(child));
        }
        if let Some(tail) = child.tail() {
            text.push_str(tail);
        }
    }

    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_text() {
        let xml = "<al>Hello <nadruk>world</nadruk>!</al>";
        let doc = Document::parse(xml).unwrap();
        let text = extract_simple_text(doc.root_element());
        assert_eq!(text, "Hello world!");
    }
}
