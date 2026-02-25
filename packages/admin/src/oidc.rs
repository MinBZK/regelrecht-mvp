use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::{
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, RedirectUrl,
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
) -> Result<ConfiguredClient, String> {
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    let issuer_url = format!(
        "{}/realms/{}",
        oidc_config.keycloak_base_url.trim_end_matches('/'),
        oidc_config.keycloak_realm
    );

    tracing::info!("discovering OIDC provider at {issuer_url}");

    let issuer = IssuerUrl::new(issuer_url)
        .map_err(|e| format!("invalid issuer URL: {e}"))?;

    let provider_metadata = CoreProviderMetadata::discover_async(issuer, &http_client)
        .await
        .map_err(|e| format!("OIDC discovery failed: {e}"))?;

    let token_url = provider_metadata
        .token_endpoint()
        .ok_or("provider metadata missing token_endpoint")?
        .clone();

    let redirect_url = RedirectUrl::new(format!("{base_url}/auth/callback"))
        .map_err(|e| format!("invalid redirect URL: {e}"))?;

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(oidc_config.client_id.clone()),
        Some(ClientSecret::new(oidc_config.client_secret.clone())),
    )
    .set_redirect_uri(redirect_url)
    .set_token_uri(token_url);

    tracing::info!("OIDC client configured successfully");

    Ok(client)
}
