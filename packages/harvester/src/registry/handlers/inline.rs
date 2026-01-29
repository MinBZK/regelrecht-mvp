//! Inline element handlers for text-level elements.
//!
//! These handlers process elements that appear inline within text,
//! such as emphasis (nadruk), external references (extref), and
//! internal references (intref).

use regex::Regex;
use roxmltree::Node;
use std::sync::LazyLock;

use crate::registry::handler::{extract_text_with_tail, ElementHandler, RecurseFn};
use crate::registry::types::{ElementType, ParseContext, ParseResult};
use crate::types::Reference;

/// Regex for extracting BWB ID from JCI reference.
static BWB_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"BWBR\d+").expect("valid regex"));

/// Regex for extracting article number from JCI reference.
static ARTIKEL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"artikel=(\d+\w*)").expect("valid regex"));

/// Convert JCI reference to wetten.overheid.nl URL.
fn convert_jci_to_url(jci_ref: &str) -> String {
    if let Some(bwb_match) = BWB_PATTERN.find(jci_ref) {
        let bwb_id = bwb_match.as_str();
        if let Some(caps) = ARTIKEL_PATTERN.captures(jci_ref) {
            let artikel = &caps[1];
            return format!("https://wetten.overheid.nl/{bwb_id}#Artikel{artikel}");
        }
        return format!("https://wetten.overheid.nl/{bwb_id}");
    }
    jci_ref.to_string()
}

/// Parse JCI reference to Reference object.
fn parse_jci_reference(jci_ref: &str) -> Option<Reference> {
    let bwb_id = BWB_PATTERN.find(jci_ref)?.as_str().to_string();

    let artikel = ARTIKEL_PATTERN
        .captures(jci_ref)
        .map(|c| c[1].to_string());

    // Extract other optional parameters
    let extract_param = |param: &str| -> Option<String> {
        let pattern = format!(r"{}=([^&]+)", regex::escape(param));
        Regex::new(&pattern)
            .ok()?
            .captures(jci_ref)
            .map(|c| c[1].to_string())
    };

    Some(Reference {
        id: String::new(), // Will be set by collector
        bwb_id,
        artikel,
        lid: extract_param("lid"),
        onderdeel: extract_param("onderdeel"),
        hoofdstuk: extract_param("hoofdstuk"),
        paragraaf: extract_param("paragraaf"),
        afdeling: extract_param("afdeling"),
    })
}

/// Handler for `<nadruk>` (emphasis) elements.
///
/// Converts emphasis to markdown: `**text**` for bold (type="vet"),
/// `*text*` for italic (default).
pub struct NadrukHandler;

impl ElementHandler for NadrukHandler {
    fn element_type(&self) -> ElementType {
        ElementType::Inline
    }

    fn handle<'a, 'input>(
        &self,
        node: Node<'a, 'input>,
        _context: &mut ParseContext<'_>,
        _recurse: &RecurseFn<'a, 'input>,
    ) -> ParseResult {
        let text = node.text().unwrap_or_default();
        let nadruk_type = node.attribute("type").unwrap_or_default();

        let formatted = if nadruk_type == "vet" {
            format!("**{text}**")
        } else {
            format!("*{text}*")
        };

        ParseResult::new(formatted)
    }
}

/// Handler for `<extref>` (external reference) elements.
///
/// Converts external references to markdown links using reference-style
/// formatting when a collector is available, or inline links otherwise.
pub struct ExtrefHandler;

impl ElementHandler for ExtrefHandler {
    fn element_type(&self) -> ElementType {
        ElementType::Inline
    }

    fn handle<'a, 'input>(
        &self,
        node: Node<'a, 'input>,
        context: &mut ParseContext<'_>,
        _recurse: &RecurseFn<'a, 'input>,
    ) -> ParseResult {
        let ref_text = node.text().unwrap_or_default();
        let doc_attr = node.attribute("doc").unwrap_or_default();

        if let Some(collector) = &mut context.collector {
            // Try to parse as JCI reference
            if let Some(reference) = parse_jci_reference(doc_attr) {
                let ref_id = collector.add_full_reference(reference);
                return ParseResult::new(format!("[{ref_text}][{ref_id}]"));
            }
        }

        // Fallback to inline link
        if !doc_attr.is_empty() {
            let url = convert_jci_to_url(doc_attr);
            return ParseResult::new(format!("[{ref_text}]({url})"));
        }

        ParseResult::new(ref_text)
    }
}

/// Handler for `<intref>` (internal reference) elements.
///
/// Converts internal references to markdown links using reference-style
/// formatting when a collector is available, or inline links otherwise.
pub struct IntrefHandler;

impl ElementHandler for IntrefHandler {
    fn element_type(&self) -> ElementType {
        ElementType::Inline
    }

    fn handle<'a, 'input>(
        &self,
        node: Node<'a, 'input>,
        context: &mut ParseContext<'_>,
        _recurse: &RecurseFn<'a, 'input>,
    ) -> ParseResult {
        let ref_text = node.text().unwrap_or_default();
        let doc_attr = node.attribute("doc").unwrap_or_default();

        if let Some(collector) = &mut context.collector {
            // Try to parse as JCI reference
            if let Some(reference) = parse_jci_reference(doc_attr) {
                let ref_id = collector.add_full_reference(reference);
                return ParseResult::new(format!("[{ref_text}][{ref_id}]"));
            }
        }

        // Fallback to inline link
        if !doc_attr.is_empty() {
            let url = convert_jci_to_url(doc_attr);
            return ParseResult::new(format!("[{ref_text}]({url})"));
        }

        ParseResult::new(ref_text)
    }
}

/// Handler for `<al>` (paragraph/alinea) elements.
///
/// Extracts inline text including child elements like extref, intref,
/// and nadruk. This is the main text container element.
pub struct AlHandler;

impl ElementHandler for AlHandler {
    fn element_type(&self) -> ElementType {
        ElementType::Inline
    }

    fn handle<'a, 'input>(
        &self,
        node: Node<'a, 'input>,
        context: &mut ParseContext<'_>,
        recurse: &RecurseFn<'a, 'input>,
    ) -> ParseResult {
        ParseResult::new(extract_text_with_tail(node, context, recurse))
    }
}

/// Handler for `<redactie>` (editorial note) elements.
///
/// Editorial notes indicate text that has been modified or replaced.
pub struct RedactieHandler;

impl ElementHandler for RedactieHandler {
    fn element_type(&self) -> ElementType {
        ElementType::Inline
    }

    fn handle<'a, 'input>(
        &self,
        node: Node<'a, 'input>,
        context: &mut ParseContext<'_>,
        recurse: &RecurseFn<'a, 'input>,
    ) -> ParseResult {
        ParseResult::new(extract_text_with_tail(node, context, recurse))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ReferenceCollector;
    use roxmltree::Document;

    fn parse_and_handle<H: ElementHandler>(handler: &H, xml: &str) -> ParseResult {
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();
        let mut context = ParseContext::new("BWBR0000000", "2025-01-01");
        let recurse = |_: Node<'_, '_>, _: &mut ParseContext<'_>| ParseResult::empty();
        handler.handle(node, &mut context, &recurse)
    }

    #[test]
    fn test_nadruk_handler_vet() {
        let result = parse_and_handle(&NadrukHandler, r#"<nadruk type="vet">bold</nadruk>"#);
        assert_eq!(result.text, "**bold**");
    }

    #[test]
    fn test_nadruk_handler_default() {
        let result = parse_and_handle(&NadrukHandler, "<nadruk>italic</nadruk>");
        assert_eq!(result.text, "*italic*");
    }

    #[test]
    fn test_convert_jci_to_url() {
        assert_eq!(
            convert_jci_to_url("jci1.3:c:BWBR0018451&artikel=4"),
            "https://wetten.overheid.nl/BWBR0018451#Artikel4"
        );

        assert_eq!(
            convert_jci_to_url("jci1.3:c:BWBR0018451"),
            "https://wetten.overheid.nl/BWBR0018451"
        );
    }

    #[test]
    fn test_extref_handler_with_collector() {
        let xml = r#"<extref doc="jci1.3:c:BWBR0018451&amp;artikel=4">artikel 4</extref>"#;
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        let mut collector = ReferenceCollector::new();
        let mut context = ParseContext::new("BWBR0000000", "2025-01-01").with_collector(&mut collector);

        let recurse = |_: Node<'_, '_>, _: &mut ParseContext<'_>| ParseResult::empty();
        let result = ExtrefHandler.handle(node, &mut context, &recurse);

        assert_eq!(result.text, "[artikel 4][ref1]");
    }

    #[test]
    fn test_extref_handler_without_collector() {
        let xml = r#"<extref doc="jci1.3:c:BWBR0018451&amp;artikel=4">artikel 4</extref>"#;
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        let mut context = ParseContext::new("BWBR0000000", "2025-01-01");

        let recurse = |_: Node<'_, '_>, _: &mut ParseContext<'_>| ParseResult::empty();
        let result = ExtrefHandler.handle(node, &mut context, &recurse);

        assert_eq!(
            result.text,
            "[artikel 4](https://wetten.overheid.nl/BWBR0018451#Artikel4)"
        );
    }

    #[test]
    fn test_parse_jci_reference() {
        let jci = "jci1.3:c:BWBR0018451&artikel=4&lid=2";
        let reference = parse_jci_reference(jci).unwrap();

        assert_eq!(reference.bwb_id, "BWBR0018451");
        assert_eq!(reference.artikel, Some("4".to_string()));
        assert_eq!(reference.lid, Some("2".to_string()));
    }
}
