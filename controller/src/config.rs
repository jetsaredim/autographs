use std::{env, net::SocketAddr, path::PathBuf};

use crate::publisher::ReleaseRetentionPolicy;

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
    pub static_promoted_release_retain_count: usize,
    pub static_failed_candidate_retain_count: usize,
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
            static_promoted_release_retain_count: retain_count_env_or_default(
                "AUTOGRAPHS_STATIC_PROMOTED_RELEASE_RETAIN_COUNT",
                ReleaseRetentionPolicy::DEFAULT_PROMOTED_RELEASE_RETAIN_COUNT,
            ),
            static_failed_candidate_retain_count: retain_count_env_or_default(
                "AUTOGRAPHS_STATIC_FAILED_CANDIDATE_RETAIN_COUNT",
                ReleaseRetentionPolicy::DEFAULT_FAILED_CANDIDATE_RETAIN_COUNT,
            ),
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
            static_promoted_release_retain_count:
                ReleaseRetentionPolicy::DEFAULT_PROMOTED_RELEASE_RETAIN_COUNT,
            static_failed_candidate_retain_count:
                ReleaseRetentionPolicy::DEFAULT_FAILED_CANDIDATE_RETAIN_COUNT,
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

fn retain_count_env_or_default(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
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

    #[test]
    fn retention_counts_ignore_zero_or_invalid_values() {
        assert_eq!(
            retain_count_env_or_default("AUTOGRAPHS_TEST_MISSING_RETAIN_COUNT", 5),
            5
        );
        unsafe {
            env::set_var("AUTOGRAPHS_TEST_ZERO_RETAIN_COUNT", "0");
            env::set_var("AUTOGRAPHS_TEST_INVALID_RETAIN_COUNT", "nope");
            env::set_var("AUTOGRAPHS_TEST_VALID_RETAIN_COUNT", "9");
        }
        assert_eq!(
            retain_count_env_or_default("AUTOGRAPHS_TEST_ZERO_RETAIN_COUNT", 5),
            5
        );
        assert_eq!(
            retain_count_env_or_default("AUTOGRAPHS_TEST_INVALID_RETAIN_COUNT", 5),
            5
        );
        assert_eq!(
            retain_count_env_or_default("AUTOGRAPHS_TEST_VALID_RETAIN_COUNT", 5),
            9
        );
    }
}
