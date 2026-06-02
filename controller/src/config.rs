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
            admin_password: env::var("AUTOGRAPHS_ADMIN_PASSWORD").ok(),
            admin_password_hash: env::var("AUTOGRAPHS_ADMIN_PASSWORD_HASH").ok(),
            operator_token: env::var("AUTOGRAPHS_OPERATOR_API_TOKEN").ok(),
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
}

fn all_present(names: &[&str]) -> bool {
    names.iter().all(|name| {
        env::var(name)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    })
}
