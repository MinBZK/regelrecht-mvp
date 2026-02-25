use std::env;

#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub client_id: String,
    pub client_secret: String,
    pub keycloak_base_url: String,
    pub keycloak_realm: String,
    pub required_role: String,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub oidc: Option<OidcConfig>,
    pub base_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        match Self::try_from_env() {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("{e}");
                std::process::exit(1);
            }
        }
    }

    fn try_from_env() -> Result<Self, String> {
        let oidc = match env::var("OIDC_CLIENT_ID").ok() {
            None => None,
            Some(client_id) => Some(Self::parse_oidc_config(client_id)?),
        };

        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());

        if oidc.is_some() {
            tracing::info!("OIDC authentication is enabled");
        } else {
            tracing::info!("OIDC authentication is disabled (dev mode)");
        }

        Ok(Self { oidc, base_url })
    }

    fn parse_oidc_config(client_id: String) -> Result<OidcConfig, String> {
        let client_secret = env::var("OIDC_CLIENT_SECRET").unwrap_or_default();
        let keycloak_base_url = env::var("KEYCLOAK_BASE_URL").unwrap_or_default();
        let keycloak_realm = env::var("KEYCLOAK_REALM").unwrap_or_default();

        let mut missing = Vec::new();
        if client_secret.is_empty() {
            missing.push("OIDC_CLIENT_SECRET");
        }
        if keycloak_base_url.is_empty() {
            missing.push("KEYCLOAK_BASE_URL");
        }
        if keycloak_realm.is_empty() {
            missing.push("KEYCLOAK_REALM");
        }

        if !missing.is_empty() {
            return Err(format!(
                "OIDC_CLIENT_ID is set but required vars are missing: {}. \
                 Refusing to start without complete OIDC configuration.",
                missing.join(", ")
            ));
        }

        let required_role =
            env::var("OIDC_REQUIRED_ROLE").unwrap_or_else(|_| "allowed-user".to_string());

        Ok(OidcConfig {
            client_id,
            client_secret,
            keycloak_base_url,
            keycloak_realm,
            required_role,
        })
    }

    pub fn is_auth_enabled(&self) -> bool {
        self.oidc.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    const OIDC_VARS: &[&str] = &[
        "OIDC_CLIENT_ID",
        "OIDC_CLIENT_SECRET",
        "KEYCLOAK_BASE_URL",
        "KEYCLOAK_REALM",
        "OIDC_REQUIRED_ROLE",
        "BASE_URL",
    ];

    fn clear_oidc_env() {
        for var in OIDC_VARS {
            env::remove_var(var);
        }
    }

    fn set_complete_oidc_env() {
        env::set_var("OIDC_CLIENT_ID", "test-client");
        env::set_var("OIDC_CLIENT_SECRET", "secret");
        env::set_var("KEYCLOAK_BASE_URL", "https://keycloak.example.com");
        env::set_var("KEYCLOAK_REALM", "test-realm");
    }

    #[test]
    fn no_oidc_vars_disables_auth() {
        let _lock = ENV_LOCK.lock();
        clear_oidc_env();

        let config = AppConfig::try_from_env().expect("should succeed");
        assert!(config.oidc.is_none());
        assert!(!config.is_auth_enabled());
    }

    #[test]
    fn complete_oidc_vars_enables_auth() {
        let _lock = ENV_LOCK.lock();
        clear_oidc_env();
        set_complete_oidc_env();

        let config = AppConfig::try_from_env().expect("should succeed");
        assert!(config.is_auth_enabled());
        let oidc = config.oidc.unwrap();
        assert_eq!(oidc.client_id, "test-client");
        assert_eq!(oidc.client_secret, "secret");
        assert_eq!(oidc.keycloak_base_url, "https://keycloak.example.com");
        assert_eq!(oidc.keycloak_realm, "test-realm");

        clear_oidc_env();
    }

    #[test]
    fn partial_oidc_config_is_hard_error() {
        let _lock = ENV_LOCK.lock();
        clear_oidc_env();
        env::set_var("OIDC_CLIENT_ID", "test-client");

        let result = AppConfig::try_from_env();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("OIDC_CLIENT_SECRET"));
        assert!(err.contains("KEYCLOAK_BASE_URL"));
        assert!(err.contains("KEYCLOAK_REALM"));

        clear_oidc_env();
    }

    #[test]
    fn partial_config_missing_one_var() {
        let _lock = ENV_LOCK.lock();
        clear_oidc_env();
        set_complete_oidc_env();
        env::remove_var("OIDC_CLIENT_SECRET");

        let result = AppConfig::try_from_env();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("OIDC_CLIENT_SECRET"));
        assert!(!err.contains("KEYCLOAK_BASE_URL"));

        clear_oidc_env();
    }

    #[test]
    fn default_role_is_allowed_user() {
        let _lock = ENV_LOCK.lock();
        clear_oidc_env();
        set_complete_oidc_env();

        let config = AppConfig::try_from_env().expect("should succeed");
        assert_eq!(config.oidc.unwrap().required_role, "allowed-user");

        clear_oidc_env();
    }

    #[test]
    fn custom_role_from_env() {
        let _lock = ENV_LOCK.lock();
        clear_oidc_env();
        set_complete_oidc_env();
        env::set_var("OIDC_REQUIRED_ROLE", "super-admin");

        let config = AppConfig::try_from_env().expect("should succeed");
        assert_eq!(config.oidc.unwrap().required_role, "super-admin");

        clear_oidc_env();
    }

    #[test]
    fn default_base_url() {
        let _lock = ENV_LOCK.lock();
        clear_oidc_env();

        let config = AppConfig::try_from_env().expect("should succeed");
        assert_eq!(config.base_url, "http://localhost:8000");
    }

    #[test]
    fn custom_base_url() {
        let _lock = ENV_LOCK.lock();
        clear_oidc_env();
        env::set_var("BASE_URL", "https://admin.example.com");

        let config = AppConfig::try_from_env().expect("should succeed");
        assert_eq!(config.base_url, "https://admin.example.com");

        clear_oidc_env();
    }
}
