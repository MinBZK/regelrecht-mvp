//! YAML writer for law files.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::text::{should_wrap_text, wrap_text_default};
use crate::config::SCHEMA_URL;
use crate::error::Result;
use crate::types::{Law, Reference};

/// Article representation for YAML serialization.
#[derive(Debug, Serialize)]
struct YamlArticle {
    number: String,
    text: String,
    url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    references: Vec<YamlReference>,
}

/// Reference representation for YAML serialization.
#[derive(Debug, Serialize)]
struct YamlReference {
    id: String,
    bwb_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    artikel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    onderdeel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hoofdstuk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    paragraaf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    afdeling: Option<String>,
}

impl From<&Reference> for YamlReference {
    fn from(r: &Reference) -> Self {
        Self {
            id: r.id.clone(),
            bwb_id: r.bwb_id.clone(),
            artikel: r.artikel.clone(),
            lid: r.lid.clone(),
            onderdeel: r.onderdeel.clone(),
            hoofdstuk: r.hoofdstuk.clone(),
            paragraaf: r.paragraaf.clone(),
            afdeling: r.afdeling.clone(),
        }
    }
}

/// Full law representation for YAML serialization.
#[derive(Debug, Serialize)]
struct YamlLaw {
    #[serde(rename = "$schema")]
    schema: String,
    #[serde(rename = "$id")]
    id: String,
    regulatory_layer: String,
    publication_date: String,
    valid_from: String,
    bwb_id: String,
    url: String,
    articles: Vec<YamlArticle>,
}

/// Generate a schema-compliant YAML structure from a Law object.
fn generate_yaml_struct(law: &Law, effective_date: &str) -> YamlLaw {
    let law_id = law.metadata.to_slug();

    let articles: Vec<YamlArticle> = law
        .articles
        .iter()
        .map(|article| {
            let text = if should_wrap_text(&article.text) {
                wrap_text_default(&article.text)
            } else {
                article.text.clone()
            };

            YamlArticle {
                number: article.number.clone(),
                text,
                url: article.url.clone(),
                references: article.references.iter().map(YamlReference::from).collect(),
            }
        })
        .collect();

    YamlLaw {
        schema: SCHEMA_URL.to_string(),
        id: law_id,
        regulatory_layer: law.metadata.regulatory_layer.as_str().to_string(),
        publication_date: law
            .metadata
            .publication_date
            .clone()
            .unwrap_or_else(|| effective_date.to_string()),
        valid_from: effective_date.to_string(),
        bwb_id: law.metadata.bwb_id.clone(),
        url: format!(
            "https://wetten.overheid.nl/{}/{}",
            law.metadata.bwb_id, effective_date
        ),
        articles,
    }
}

/// Generate YAML string from a Law object.
pub fn generate_yaml(law: &Law, effective_date: &str) -> Result<String> {
    let yaml_struct = generate_yaml_struct(law, effective_date);
    let yaml_string = serde_yaml::to_string(&yaml_struct)?;

    // Add document start marker and clean up trailing whitespace
    let lines: Vec<&str> = yaml_string.lines().map(|l| l.trim_end()).collect();
    let content = format!("---\n{}\n", lines.join("\n"));

    Ok(content)
}

/// Save a Law object as a YAML file.
///
/// Uses atomic write pattern: writes to temp file, syncs to disk, then renames.
/// This ensures partial writes don't corrupt existing files on crash.
///
/// # Arguments
/// * `law` - The Law object to save
/// * `effective_date` - The effective date in YYYY-MM-DD format
/// * `output_base` - Base directory for output (default: "regulation/nl/")
///
/// # Returns
/// Path to the saved file
pub fn save_yaml(law: &Law, effective_date: &str, output_base: Option<&Path>) -> Result<PathBuf> {
    let output_base = output_base.unwrap_or(Path::new("regulation/nl"));

    // Determine directory structure
    let layer_dir = law.metadata.regulatory_layer.as_dir_name();
    let law_id = law.metadata.to_slug();
    let output_dir = output_base.join(layer_dir).join(&law_id);
    fs::create_dir_all(&output_dir)?;

    let output_file = output_dir.join(format!("{effective_date}.yaml"));
    let temp_file = output_dir.join(format!(".{effective_date}.yaml.tmp"));

    // Generate YAML content
    let content = generate_yaml(law, effective_date)?;

    // Write to temp file first, then sync and rename for atomicity
    {
        let mut file = File::create(&temp_file)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?; // Ensure data is flushed to disk
    }

    // Atomic rename (on most filesystems)
    fs::rename(&temp_file, &output_file)?;

    Ok(output_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Article, LawMetadata, RegulatoryLayer};
    use tempfile::tempdir;

    fn create_test_law() -> Law {
        let metadata = LawMetadata {
            bwb_id: "BWBR0018451".to_string(),
            title: "Wet op de zorgtoeslag".to_string(),
            regulatory_layer: RegulatoryLayer::Wet,
            publication_date: Some("2005-12-29".to_string()),
            effective_date: None,
        };

        let mut law = Law::new(metadata);
        law.add_article(Article::new(
            "1",
            "In deze wet wordt verstaan onder toeslagpartner: partner.",
            "https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel1",
        ));
        law
    }

    #[test]
    fn test_generate_yaml() {
        let law = create_test_law();
        let yaml = generate_yaml(&law, "2025-01-01").unwrap();

        assert!(yaml.starts_with("---\n"));
        assert!(yaml.contains("$schema:"));
        assert!(yaml.contains("$id: wet_op_de_zorgtoeslag"));
        assert!(yaml.contains("regulatory_layer: WET"));
        assert!(yaml.contains("bwb_id: BWBR0018451"));
    }

    #[test]
    fn test_save_yaml() {
        let law = create_test_law();
        let temp_dir = tempdir().unwrap();
        let output_path = save_yaml(&law, "2025-01-01", Some(temp_dir.path())).unwrap();

        assert!(output_path.exists());
        // Check path components (works on both Windows and Unix)
        let path_str = output_path.to_string_lossy();
        assert!(path_str.contains("wet"));
        assert!(path_str.contains("wet_op_de_zorgtoeslag"));
        assert!(path_str.contains("2025-01-01.yaml"));

        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.starts_with("---\n"));
    }

    #[test]
    fn test_yaml_reference_serialization() {
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

        let yaml_ref = YamlReference::from(&reference);
        assert_eq!(yaml_ref.id, "ref1");
        assert_eq!(yaml_ref.artikel, Some("4".to_string()));
        assert!(yaml_ref.lid.is_none());
    }
}
