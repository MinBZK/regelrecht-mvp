use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::{
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, RedirectUrl,
};

use crate::config::OidcConfig;

pub type ConfiguredClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
    EndpointMaybeSet,
>;

pub async fn discover_client(
    oidc_config: &OidcConfig,
    base_url: &str,
) -> Result<(ConfiguredClient, CoreProviderMetadata), String> {
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    let discovery_url = &oidc_config.discovery_url;

    tracing::info!("fetching OIDC provider metadata from {discovery_url}");

    let metadata_response = http_client
        .get(discovery_url)
        .send()
        .await
        .map_err(|e| format!("failed to fetch OIDC discovery document: {e}"))?;

    let metadata_json: serde_json::Value = metadata_response
        .json()
        .await
        .map_err(|e| format!("failed to parse OIDC discovery JSON: {e}"))?;

    let provider_metadata: CoreProviderMetadata = serde_json::from_value(metadata_json)
        .map_err(|e| format!("failed to parse provider metadata: {e}"))?;

    let token_url = provider_metadata
        .token_endpoint()
        .ok_or("provider metadata missing token_endpoint")?
        .clone();

    let redirect_url = RedirectUrl::new(format!("{base_url}/auth/callback"))
        .map_err(|e| format!("invalid redirect URL: {e}"))?;

    let client = CoreClient::from_provider_metadata(
        provider_metadata.clone(),
        ClientId::new(oidc_config.client_id.clone()),
        Some(ClientSecret::new(oidc_config.client_secret.clone())),
    )
    .set_redirect_uri(redirect_url)
    .set_token_uri(token_url);

    tracing::info!("OIDC client configured successfully");

    Ok((client, provider_metadata))
}
