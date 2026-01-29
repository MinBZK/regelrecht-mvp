//! HTTP client wrapper for downloading from BWB repository.

use std::thread;
use std::time::Duration;

use reqwest::blocking::Client;

use crate::config::HTTP_TIMEOUT_SECS;
use crate::error::{HarvesterError, Result};

/// User agent string identifying this harvester.
const USER_AGENT: &str = concat!("regelrecht-harvester/", env!("CARGO_PKG_VERSION"));

/// Maximum number of retry attempts for transient failures.
const MAX_RETRIES: u32 = 3;

/// Base delay for exponential backoff (milliseconds).
const RETRY_BASE_DELAY_MS: u64 = 500;

/// Create a configured HTTP client.
///
/// # Returns
/// A `reqwest::blocking::Client` configured with appropriate timeout and user agent.
pub fn create_client() -> Result<Client> {
    let client = Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .user_agent(USER_AGENT)
        .build()?;
    Ok(client)
}

/// Download content from a URL with retry logic.
///
/// Uses exponential backoff for transient failures (network errors, 5xx responses).
///
/// # Arguments
/// * `client` - HTTP client to use
/// * `url` - URL to download from
///
/// # Returns
/// Raw bytes of the response body
pub fn download_bytes(client: &Client, url: &str) -> Result<Vec<u8>> {
    let mut last_error: Option<String> = None;

    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            // Exponential backoff: 500ms, 1000ms, 2000ms
            let delay = RETRY_BASE_DELAY_MS * (1 << (attempt - 1));
            tracing::debug!(attempt, delay_ms = delay, "Retrying after delay");
            thread::sleep(Duration::from_millis(delay));
        }

        match client.get(url).send() {
            Ok(response) => {
                let status = response.status();

                // Retry on server errors (5xx)
                if status.is_server_error() {
                    tracing::warn!(
                        status = %status,
                        attempt = attempt + 1,
                        max_retries = MAX_RETRIES,
                        "Server error, will retry"
                    );
                    last_error = Some(format!("Server error: {status}"));
                    continue;
                }

                // Don't retry client errors (4xx) - they won't succeed
                let response = response.error_for_status()?;
                let bytes = response.bytes()?;
                return Ok(bytes.to_vec());
            }
            Err(e) => {
                // Retry on connection/timeout errors
                if e.is_connect() || e.is_timeout() {
                    tracing::warn!(
                        error = %e,
                        attempt = attempt + 1,
                        max_retries = MAX_RETRIES,
                        "Connection error, will retry"
                    );
                    last_error = Some(e.to_string());
                    continue;
                }
                // Other errors (like invalid URL) - don't retry
                return Err(HarvesterError::Http(e));
            }
        }
    }

    // All retries exhausted
    Err(HarvesterError::RetriesExhausted {
        attempts: MAX_RETRIES,
        message: last_error.unwrap_or_else(|| "Unknown error".to_string()),
    })
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
