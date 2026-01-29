//! Configuration constants and validation functions for the harvester.

use regex::Regex;
use std::sync::LazyLock;

use crate::error::{HarvesterError, Result};

/// Base URL for BWB (Basiswettenbestand) repository.
pub const BWB_REPOSITORY_URL: &str = "https://repository.officiele-overheidspublicaties.nl/bwb";

/// HTTP timeout in seconds.
pub const HTTP_TIMEOUT_SECS: u64 = 10;

/// Schema URL for regelrecht YAML files.
pub const SCHEMA_URL: &str = "https://raw.githubusercontent.com/MinBZK/poc-machine-law/refs/heads/main/schema/v0.3.1/schema.json";

/// Text wrap width for YAML output.
pub const TEXT_WRAP_WIDTH: usize = 100;

/// BWB ID pattern: BWBR followed by 7 digits.
static BWB_ID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^BWBR\d{7}$").expect("valid regex"));

/// Date pattern: YYYY-MM-DD.
static DATE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("valid regex"));

/// Validate BWB ID format.
///
/// # Arguments
/// * `bwb_id` - The BWB identifier to validate
///
/// # Returns
/// * `Ok(())` if valid
/// * `Err(HarvesterError::InvalidBwbId)` if invalid
///
/// # Examples
/// ```
/// use regelrecht_harvester::config::validate_bwb_id;
///
/// assert!(validate_bwb_id("BWBR0018451").is_ok());
/// assert!(validate_bwb_id("INVALID").is_err());
/// ```
pub fn validate_bwb_id(bwb_id: &str) -> Result<()> {
    if BWB_ID_PATTERN.is_match(bwb_id) {
        Ok(())
    } else {
        Err(HarvesterError::InvalidBwbId(bwb_id.to_string()))
    }
}

/// Validate date format (YYYY-MM-DD).
///
/// # Arguments
/// * `date_str` - Date string to validate
///
/// # Returns
/// * `Ok(())` if valid format and valid date
/// * `Err(HarvesterError::InvalidDate)` if invalid
///
/// # Examples
/// ```
/// use regelrecht_harvester::config::validate_date;
///
/// assert!(validate_date("2025-01-01").is_ok());
/// assert!(validate_date("invalid").is_err());
/// assert!(validate_date("2025-13-01").is_err()); // Invalid month
/// ```
pub fn validate_date(date_str: &str) -> Result<()> {
    if !DATE_PATTERN.is_match(date_str) {
        return Err(HarvesterError::InvalidDate(date_str.to_string()));
    }

    // Also validate that it's a real date
    match chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(_) => Ok(()),
        Err(_) => Err(HarvesterError::InvalidDate(date_str.to_string())),
    }
}

/// Build WTI (metadata) URL for a law.
///
/// # Arguments
/// * `bwb_id` - The BWB identifier
///
/// # Returns
/// URL to the WTI file
pub fn wti_url(bwb_id: &str) -> String {
    format!("{BWB_REPOSITORY_URL}/{bwb_id}/{bwb_id}.WTI")
}

/// Build content (consolidated XML) URL for a law at a specific date.
///
/// # Arguments
/// * `bwb_id` - The BWB identifier
/// * `date` - The effective date in YYYY-MM-DD format
///
/// # Returns
/// URL to the consolidated XML file
pub fn content_url(bwb_id: &str, date: &str) -> String {
    format!("{BWB_REPOSITORY_URL}/{bwb_id}/{date}_0/xml/{bwb_id}_{date}_0.xml")
}

/// Build wetten.overheid.nl URL for a law.
///
/// # Arguments
/// * `bwb_id` - The BWB identifier
/// * `date` - Optional effective date
/// * `article` - Optional article number for fragment
///
/// # Returns
/// Public URL to wetten.overheid.nl
pub fn wetten_url(bwb_id: &str, date: Option<&str>, article: Option<&str>) -> String {
    let mut url = format!("https://wetten.overheid.nl/{bwb_id}");

    if let Some(d) = date {
        url.push('/');
        url.push_str(d);
    }

    if let Some(a) = article {
        url.push_str("#Artikel");
        url.push_str(&a.replace(' ', "_"));
    }

    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bwb_id_valid() {
        assert!(validate_bwb_id("BWBR0018451").is_ok());
        assert!(validate_bwb_id("BWBR0000001").is_ok());
        assert!(validate_bwb_id("BWBR9999999").is_ok());
    }

    #[test]
    fn test_validate_bwb_id_invalid() {
        assert!(validate_bwb_id("").is_err());
        assert!(validate_bwb_id("BWBR001845").is_err()); // 6 digits
        assert!(validate_bwb_id("BWBR00184512").is_err()); // 8 digits
        assert!(validate_bwb_id("BWBX0018451").is_err()); // Wrong prefix
        assert!(validate_bwb_id("bwbr0018451").is_err()); // Lowercase
    }

    #[test]
    fn test_validate_date_valid() {
        assert!(validate_date("2025-01-01").is_ok());
        assert!(validate_date("2024-12-31").is_ok());
        assert!(validate_date("2000-06-15").is_ok());
    }

    #[test]
    fn test_validate_date_invalid_format() {
        assert!(validate_date("").is_err());
        assert!(validate_date("2025/01/01").is_err());
        assert!(validate_date("01-01-2025").is_err());
        assert!(validate_date("2025-1-1").is_err());
    }

    #[test]
    fn test_validate_date_invalid_date() {
        assert!(validate_date("2025-13-01").is_err()); // Invalid month
        assert!(validate_date("2025-02-30").is_err()); // Invalid day
        assert!(validate_date("2025-00-01").is_err()); // Zero month
    }

    #[test]
    fn test_wti_url() {
        assert_eq!(
            wti_url("BWBR0018451"),
            "https://repository.officiele-overheidspublicaties.nl/bwb/BWBR0018451/BWBR0018451.WTI"
        );
    }

    #[test]
    fn test_content_url() {
        assert_eq!(
            content_url("BWBR0018451", "2025-01-01"),
            "https://repository.officiele-overheidspublicaties.nl/bwb/BWBR0018451/2025-01-01_0/xml/BWBR0018451_2025-01-01_0.xml"
        );
    }

    #[test]
    fn test_wetten_url() {
        assert_eq!(
            wetten_url("BWBR0018451", None, None),
            "https://wetten.overheid.nl/BWBR0018451"
        );

        assert_eq!(
            wetten_url("BWBR0018451", Some("2025-01-01"), None),
            "https://wetten.overheid.nl/BWBR0018451/2025-01-01"
        );

        assert_eq!(
            wetten_url("BWBR0018451", Some("2025-01-01"), Some("1")),
            "https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel1"
        );

        // Test space replacement in article numbers
        assert_eq!(
            wetten_url("BWBR0018451", Some("2025-01-01"), Some("A 1")),
            "https://wetten.overheid.nl/BWBR0018451/2025-01-01#ArtikelA_1"
        );
    }
}
