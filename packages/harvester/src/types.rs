//! Core data types for the harvester.
//!
//! These types represent Dutch legal documents and their components,
//! matching the Python harvester's `models.py`.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::config::wetten_url;

/// Types of regulatory documents in Dutch law.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RegulatoryLayer {
    /// Formal law (wet).
    #[serde(rename = "WET")]
    Wet,

    /// General administrative measure (Algemene Maatregel van Bestuur).
    #[serde(rename = "AMVB")]
    Amvb,

    /// Ministerial regulation (Ministeriële regeling).
    #[serde(rename = "MINISTERIELE_REGELING")]
    MinisterieleRegeling,

    /// Royal decree (Koninklijk Besluit).
    #[serde(rename = "KONINKLIJK_BESLUIT")]
    KoninklijkBesluit,

    /// Policy rule (Beleidsregel).
    #[serde(rename = "BELEIDSREGEL")]
    Beleidsregel,

    /// Ordinance (Verordening).
    #[serde(rename = "VERORDENING")]
    Verordening,

    /// Regulation (Regeling).
    #[serde(rename = "REGELING")]
    Regeling,
}

impl RegulatoryLayer {
    /// Get the string value for YAML output.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wet => "WET",
            Self::Amvb => "AMVB",
            Self::MinisterieleRegeling => "MINISTERIELE_REGELING",
            Self::KoninklijkBesluit => "KONINKLIJK_BESLUIT",
            Self::Beleidsregel => "BELEIDSREGEL",
            Self::Verordening => "VERORDENING",
            Self::Regeling => "REGELING",
        }
    }

    /// Get the directory name for file output.
    #[must_use]
    pub fn to_dir_name(&self) -> &'static str {
        match self {
            Self::Wet => "wet",
            Self::Amvb => "amvb",
            Self::MinisterieleRegeling => "ministeriele_regeling",
            Self::KoninklijkBesluit => "koninklijk_besluit",
            Self::Beleidsregel => "beleidsregel",
            Self::Verordening => "verordening",
            Self::Regeling => "regeling",
        }
    }

    /// Parse from WTI "soort-regeling" text.
    #[must_use]
    pub fn from_soort_regeling(text: &str) -> Self {
        match text.to_lowercase().as_str() {
            "wet" => Self::Wet,
            "amvb" | "algemene maatregel van bestuur" => Self::Amvb,
            "ministeriele regeling" | "ministeriële regeling" => Self::MinisterieleRegeling,
            "koninklijk besluit" | "kb" => Self::KoninklijkBesluit,
            "beleidsregel" => Self::Beleidsregel,
            "verordening" => Self::Verordening,
            "regeling" => Self::Regeling,
            _ => Self::Wet, // Default
        }
    }
}

/// Metadata extracted from WTI file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawMetadata {
    /// BWB identifier (e.g., "BWBR0018451").
    pub bwb_id: String,

    /// Official title (citeertitel).
    pub title: String,

    /// Type of regulatory document.
    pub regulatory_layer: RegulatoryLayer,

    /// Publication date (optional).
    pub publication_date: Option<String>,

    /// Effective date (optional, usually set from request).
    pub effective_date: Option<String>,
}

/// Regex for slug generation - matches non-word characters.
static SLUG_NON_WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\w\s-]").expect("valid regex"));

/// Regex for slug generation - matches whitespace and dashes.
static SLUG_SPACE_DASH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[-\s]+").expect("valid regex"));

impl LawMetadata {
    /// Generate a URL-friendly slug from the title.
    ///
    /// # Examples
    /// ```
    /// use regelrecht_harvester::types::{LawMetadata, RegulatoryLayer};
    ///
    /// let metadata = LawMetadata {
    ///     bwb_id: "BWBR0018451".to_string(),
    ///     title: "Wet op de zorgtoeslag".to_string(),
    ///     regulatory_layer: RegulatoryLayer::Wet,
    ///     publication_date: None,
    ///     effective_date: None,
    /// };
    /// assert_eq!(metadata.to_slug(), "wet_op_de_zorgtoeslag");
    /// ```
    #[must_use]
    pub fn to_slug(&self) -> String {
        let text = self.title.to_lowercase();
        let text = SLUG_NON_WORD.replace_all(&text, "");
        let text = SLUG_SPACE_DASH.replace_all(&text, "_");
        text.trim_matches('_').to_string()
    }
}

/// A reference to another article or law.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    /// Unique identifier for this reference (e.g., "ref1").
    pub id: String,

    /// BWB identifier of the referenced law.
    pub bwb_id: String,

    /// Article number (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artikel: Option<String>,

    /// Paragraph number (lid) (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lid: Option<String>,

    /// Subdivision (onderdeel) (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onderdeel: Option<String>,

    /// Chapter (hoofdstuk) (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hoofdstuk: Option<String>,

    /// Section (paragraaf) (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraaf: Option<String>,

    /// Division (afdeling) (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub afdeling: Option<String>,
}

impl Reference {
    /// Create a new reference with just BWB ID.
    #[must_use]
    pub fn new(id: impl Into<String>, bwb_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bwb_id: bwb_id.into(),
            artikel: None,
            lid: None,
            onderdeel: None,
            hoofdstuk: None,
            paragraaf: None,
            afdeling: None,
        }
    }

    /// Generate wetten.overheid.nl URL.
    ///
    /// # Arguments
    /// * `date` - Optional date for versioned URL
    #[must_use]
    pub fn to_wetten_url(&self, date: Option<&str>) -> String {
        wetten_url(&self.bwb_id, date, self.artikel.as_deref())
    }
}

/// Format references as markdown reference definitions.
///
/// # Arguments
/// * `references` - List of references to format
///
/// # Returns
/// Markdown reference definitions, e.g.:
/// ```text
/// [ref1]: https://wetten.overheid.nl/BWBR0018451#Artikel4
/// [ref2]: https://wetten.overheid.nl/BWBR0018450#Artikel1
/// ```
#[must_use]
pub fn format_reference_definitions(references: &[Reference]) -> String {
    if references.is_empty() {
        return String::new();
    }

    references
        .iter()
        .map(|r| format!("[{}]: {}", r.id, r.to_wetten_url(None)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A single article from a law.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    /// Article number (e.g., "1", "1.1", "1.1.a").
    pub number: String,

    /// Article text content.
    pub text: String,

    /// URL to the article on wetten.overheid.nl.
    pub url: String,

    /// References contained in this article.
    pub references: Vec<Reference>,
}

impl Article {
    /// Create a new article.
    #[must_use]
    pub fn new(
        number: impl Into<String>,
        text: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            number: number.into(),
            text: text.into(),
            url: url.into(),
            references: Vec::new(),
        }
    }

    /// Create an article with references.
    #[must_use]
    pub fn with_references(mut self, references: Vec<Reference>) -> Self {
        self.references = references;
        self
    }
}

/// Complete law with metadata and articles.
#[derive(Debug, Clone)]
pub struct Law {
    /// Metadata from WTI file.
    pub metadata: LawMetadata,

    /// List of articles.
    pub articles: Vec<Article>,
}

impl Law {
    /// Create a new law with metadata.
    #[must_use]
    pub fn new(metadata: LawMetadata) -> Self {
        Self {
            metadata,
            articles: Vec::new(),
        }
    }

    /// Add an article to the law.
    pub fn add_article(&mut self, article: Article) {
        self.articles.push(article);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regulatory_layer_as_str() {
        assert_eq!(RegulatoryLayer::Wet.as_str(), "WET");
        assert_eq!(RegulatoryLayer::Amvb.as_str(), "AMVB");
        assert_eq!(
            RegulatoryLayer::MinisterieleRegeling.as_str(),
            "MINISTERIELE_REGELING"
        );
    }

    #[test]
    fn test_regulatory_layer_to_dir_name() {
        assert_eq!(RegulatoryLayer::Wet.to_dir_name(), "wet");
        assert_eq!(
            RegulatoryLayer::MinisterieleRegeling.to_dir_name(),
            "ministeriele_regeling"
        );
    }

    #[test]
    fn test_regulatory_layer_from_soort_regeling() {
        assert_eq!(RegulatoryLayer::from_soort_regeling("wet"), RegulatoryLayer::Wet);
        assert_eq!(RegulatoryLayer::from_soort_regeling("WET"), RegulatoryLayer::Wet);
        assert_eq!(RegulatoryLayer::from_soort_regeling("amvb"), RegulatoryLayer::Amvb);
        assert_eq!(
            RegulatoryLayer::from_soort_regeling("algemene maatregel van bestuur"),
            RegulatoryLayer::Amvb
        );
        assert_eq!(
            RegulatoryLayer::from_soort_regeling("ministeriële regeling"),
            RegulatoryLayer::MinisterieleRegeling
        );
        // Unknown defaults to Wet
        assert_eq!(
            RegulatoryLayer::from_soort_regeling("unknown"),
            RegulatoryLayer::Wet
        );
    }

    #[test]
    fn test_law_metadata_to_slug() {
        let metadata = LawMetadata {
            bwb_id: "BWBR0018451".to_string(),
            title: "Wet op de zorgtoeslag".to_string(),
            regulatory_layer: RegulatoryLayer::Wet,
            publication_date: None,
            effective_date: None,
        };
        assert_eq!(metadata.to_slug(), "wet_op_de_zorgtoeslag");
    }

    #[test]
    fn test_law_metadata_to_slug_special_chars() {
        let metadata = LawMetadata {
            bwb_id: "BWBR0000000".to_string(),
            title: "Wet (test) - special!".to_string(),
            regulatory_layer: RegulatoryLayer::Wet,
            publication_date: None,
            effective_date: None,
        };
        assert_eq!(metadata.to_slug(), "wet_test_special");
    }

    #[test]
    fn test_reference_to_wetten_url() {
        let reference = Reference {
            id: "ref1".to_string(),
            bwb_id: "BWBR0018451".to_string(),
            artikel: Some("4".to_string()),
            lid: None,
            onderdeel: None,
            hoofdstuk: None,
            paragraaf: None,
            afdeling: None,
        };

        assert_eq!(
            reference.to_wetten_url(None),
            "https://wetten.overheid.nl/BWBR0018451#Artikel4"
        );
    }

    #[test]
    fn test_format_reference_definitions() {
        let references = vec![
            Reference::new("ref1", "BWBR0018451"),
            Reference {
                id: "ref2".to_string(),
                bwb_id: "BWBR0018450".to_string(),
                artikel: Some("1".to_string()),
                ..Reference::new("", "")
            },
        ];

        let result = format_reference_definitions(&references);
        assert!(result.contains("[ref1]: https://wetten.overheid.nl/BWBR0018451"));
        assert!(result.contains("[ref2]: https://wetten.overheid.nl/BWBR0018450#Artikel1"));
    }

    #[test]
    fn test_format_reference_definitions_empty() {
        let references: Vec<Reference> = vec![];
        assert_eq!(format_reference_definitions(&references), "");
    }

    #[test]
    fn test_article_creation() {
        let article = Article::new("1", "Test text", "https://example.com");
        assert_eq!(article.number, "1");
        assert_eq!(article.text, "Test text");
        assert!(article.references.is_empty());
    }

    #[test]
    fn test_law_add_article() {
        let metadata = LawMetadata {
            bwb_id: "BWBR0018451".to_string(),
            title: "Test Law".to_string(),
            regulatory_layer: RegulatoryLayer::Wet,
            publication_date: None,
            effective_date: None,
        };

        let mut law = Law::new(metadata);
        assert!(law.articles.is_empty());

        law.add_article(Article::new("1", "Text", "url"));
        assert_eq!(law.articles.len(), 1);
    }

    #[test]
    fn test_regulatory_layer_serialization() {
        assert_eq!(
            serde_json::to_string(&RegulatoryLayer::Wet).unwrap(),
            "\"WET\""
        );
        assert_eq!(
            serde_json::to_string(&RegulatoryLayer::MinisterieleRegeling).unwrap(),
            "\"MINISTERIELE_REGELING\""
        );
    }
}
