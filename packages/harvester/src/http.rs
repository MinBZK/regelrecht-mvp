//! HTTP client wrapper for downloading from BWB repository.

use std::time::Duration;

use reqwest::blocking::Client;

use crate::config::HTTP_TIMEOUT_SECS;
use crate::error::Result;

/// Create a configured HTTP client.
///
/// # Returns
/// A `reqwest::blocking::Client` configured with appropriate timeout.
pub fn create_client() -> Result<Client> {
    let client = Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()?;
    Ok(client)
}

/// Download content from a URL.
///
/// # Arguments
/// * `client` - HTTP client to use
/// * `url` - URL to download from
///
/// # Returns
/// Raw bytes of the response body
pub fn download_bytes(client: &Client, url: &str) -> Result<Vec<u8>> {
    let response = client.get(url).send()?;
    let response = response.error_for_status()?;
    let bytes = response.bytes()?;
    Ok(bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client() {
        let client = create_client();
        assert!(client.is_ok());
    }
}
