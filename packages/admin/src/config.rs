use std::env;

#[derive(Clone)]
pub struct OidcConfig {
    pub client_id: String,
    pub client_secret: String,
    pub discovery_url: String,
    pub keycloak_base_url: String,
    pub keycloak_realm: String,
}

#[derive(Clone)]
pub struct AppConfig {
    pub oidc: Option<OidcConfig>,
    pub base_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let oidc = env::var("OIDC_CLIENT_ID").ok().and_then(|client_id| {
            let client_secret = env::var("OIDC_CLIENT_SECRET").unwrap_or_default();
            let discovery_url = env::var("OIDC_DISCOVERY_URL").unwrap_or_default();
            let keycloak_base_url = env::var("KEYCLOAK_BASE_URL").unwrap_or_default();
            let keycloak_realm = env::var("KEYCLOAK_REALM").unwrap_or_default();

            let mut missing = Vec::new();
            if client_secret.is_empty() {
                missing.push("OIDC_CLIENT_SECRET");
            }
            if discovery_url.is_empty() {
                missing.push("OIDC_DISCOVERY_URL");
            }
            if keycloak_base_url.is_empty() {
                missing.push("KEYCLOAK_BASE_URL");
            }
            if keycloak_realm.is_empty() {
                missing.push("KEYCLOAK_REALM");
            }

            if !missing.is_empty() {
                tracing::warn!(
                    "OIDC_CLIENT_ID is set but required vars are missing: {}. Falling back to dev mode.",
                    missing.join(", ")
                );
                return None;
            }

            Some(OidcConfig {
                client_id,
                client_secret,
                discovery_url,
                keycloak_base_url,
                keycloak_realm,
            })
        });

        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());

        if oidc.is_some() {
            tracing::info!("OIDC authentication is enabled");
        } else {
            tracing::info!("OIDC authentication is disabled (dev mode)");
        }

        Self { oidc, base_url }
    }

    pub fn is_auth_enabled(&self) -> bool {
        self.oidc.is_some()
    }
}
