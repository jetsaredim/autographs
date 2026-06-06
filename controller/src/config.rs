use std::{env, net::SocketAddr, path::PathBuf};

#[derive(Clone, Debug)]
pub struct ControllerConfig {
    pub bind_addr: SocketAddr,
    pub public_origin: String,
    pub secure_cookies: bool,
    pub admin_password: Option<String>,
    pub admin_password_hash: Option<String>,
    pub operator_token: Option<String>,
    pub oracle_configured: bool,
    pub media_configured: bool,
    pub static_release_configured: bool,
    pub static_release_root: PathBuf,
}

impl ControllerConfig {
    pub fn from_env() -> Result<Self, String> {
        let bind_addr = env::var("AUTOGRAPHS_CONTROLLER_BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_owned())
            .parse()
            .map_err(|error| format!("invalid AUTOGRAPHS_CONTROLLER_BIND_ADDR: {error}"))?;
        let public_origin = env::var("AUTOGRAPHS_PUBLIC_ORIGIN")
            .unwrap_or_else(|_| "https://autographs.jetsaredim.net".to_owned());
        let secure_cookies = env::var("AUTOGRAPHS_ADMIN_SECURE_COOKIES")
            .map(|value| value != "false")
            .unwrap_or(true);

        Ok(Self {
            bind_addr,
            public_origin,
            secure_cookies,
            admin_password: non_blank_env("AUTOGRAPHS_ADMIN_PASSWORD"),
            admin_password_hash: non_blank_env("AUTOGRAPHS_ADMIN_PASSWORD_HASH"),
            operator_token: non_blank_env("AUTOGRAPHS_OPERATOR_API_TOKEN"),
            oracle_configured: all_present(&[
                "ORACLE_DB_USER",
                "ORACLE_DB_PASSWORD",
                "ORACLE_DB_CONNECT_STRING",
            ]),
            media_configured: all_present(&["OCI_MEDIA_BUCKET_NAME", "OCI_MEDIA_NAMESPACE"]),
            static_release_configured: env::var("AUTOGRAPHS_STATIC_RELEASE_ROOT").is_ok(),
            static_release_root: env::var("AUTOGRAPHS_STATIC_RELEASE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/tmp/autographs-static")),
        })
    }

    pub fn for_test(secure_cookies: bool) -> Self {
        Self {
            bind_addr: "127.0.0.1:0".parse().expect("test bind address"),
            public_origin: "https://autographs.example.test".to_owned(),
            secure_cookies,
            admin_password: Some("local-test-password".to_owned()),
            admin_password_hash: None,
            operator_token: Some("operator-test-token".to_owned()),
            oracle_configured: false,
            media_configured: false,
            static_release_configured: false,
            static_release_root: PathBuf::from("/tmp/autographs-static"),
        }
    }

    pub fn validate_runtime_auth(&self) -> Result<(), String> {
        if self.admin_password.is_none()
            && self.admin_password_hash.is_none()
            && self.operator_token.is_none()
        {
            return Err(
                "configure a non-empty AUTOGRAPHS_ADMIN_PASSWORD, AUTOGRAPHS_ADMIN_PASSWORD_HASH, or AUTOGRAPHS_OPERATOR_API_TOKEN"
                    .to_owned(),
            );
        }
        Ok(())
    }
}

fn all_present(names: &[&str]) -> bool {
    names.iter().all(|name| {
        env::var(name)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    })
}

fn non_blank_env(name: &str) -> Option<String> {
    env::var(name).ok().and_then(non_blank)
}

pub(crate) fn non_blank(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_auth_values_are_ignored_for_runtime_validation() {
        let mut config = ControllerConfig::for_test(true);
        config.admin_password = Some("   ".to_owned()).and_then(non_blank);
        config.admin_password_hash = None;
        config.operator_token = Some("\t\n".to_owned()).and_then(non_blank);

        assert!(config.validate_runtime_auth().is_err());
    }

    #[test]
    fn non_empty_auth_value_satisfies_runtime_validation() {
        let mut config = ControllerConfig::for_test(true);
        config.admin_password = None;
        config.admin_password_hash = Some(" hash ".to_owned()).and_then(non_blank);
        config.operator_token = None;

        assert!(config.validate_runtime_auth().is_ok());
        assert_eq!(config.admin_password_hash.as_deref(), Some("hash"));
    }
}
