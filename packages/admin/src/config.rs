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
        let oidc = env::var("OIDC_CLIENT_ID").ok().map(|client_id| OidcConfig {
            client_id,
            client_secret: env::var("OIDC_CLIENT_SECRET").unwrap_or_else(|_| String::new()),
            discovery_url: env::var("OIDC_DISCOVERY_URL").unwrap_or_else(|_| String::new()),
            keycloak_base_url: env::var("KEYCLOAK_BASE_URL").unwrap_or_else(|_| String::new()),
            keycloak_realm: env::var("KEYCLOAK_REALM").unwrap_or_else(|_| String::new()),
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
