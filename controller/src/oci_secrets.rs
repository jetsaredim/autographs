use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::Method;
use serde::Deserialize;

use crate::oci_auth::{self, OciInstancePrincipalClient};

#[derive(Clone)]
pub struct OciVaultSecretClient {
    auth: OciInstancePrincipalClient,
    endpoint: SecretsEndpoint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SecretsEndpoint {
    host: String,
    base_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecretBundle {
    version_number: u64,
    secret_bundle_content: SecretBundleContent,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecretBundleContent {
    content_type: String,
    content: Option<String>,
}

pub(crate) struct VaultSecretValue {
    value: String,
    version_number: u64,
}

impl VaultSecretValue {
    pub(crate) fn new(value: String, version_number: u64) -> Self {
        Self {
            value,
            version_number,
        }
    }

    pub(crate) fn expose_secret(&self) -> &str {
        &self.value
    }

    pub(crate) fn into_secret(self) -> String {
        self.value
    }

    pub(crate) fn version_number(&self) -> u64 {
        self.version_number
    }
}

impl OciVaultSecretClient {
    pub fn new() -> Result<Self, String> {
        let auth = OciInstancePrincipalClient::new()?;
        let endpoint = SecretsEndpoint::from_env_or_region(&auth);
        Ok(Self { auth, endpoint })
    }

    pub(crate) async fn read_current_secret(
        &self,
        secret_id: &str,
    ) -> Result<VaultSecretValue, String> {
        let secret_id = secret_id.trim();
        if secret_id.is_empty() {
            return Err("Vault secret OCID must not be blank".to_owned());
        }
        let path = self.endpoint.current_secret_bundle_path(secret_id);
        let response = self
            .auth
            .execute(Method::GET, &self.endpoint.host, &path, None, None)
            .await
            .map_err(|error| format!("request OCI Vault secret bundle: {error}"))?;
        let response = oci_auth::require_success(response, "read OCI Vault secret bundle").await?;
        let bundle: SecretBundle = response
            .json()
            .await
            .map_err(|error| format!("decode OCI Vault secret bundle: {error}"))?;
        decode_secret_bundle(bundle)
    }
}

fn decode_secret_bundle(bundle: SecretBundle) -> Result<VaultSecretValue, String> {
    if bundle.secret_bundle_content.content_type != "BASE64" {
        return Err(format!(
            "unsupported OCI Vault secret content type: {}",
            bundle.secret_bundle_content.content_type
        ));
    }
    let encoded = bundle
        .secret_bundle_content
        .content
        .ok_or_else(|| "OCI Vault secret bundle content is missing".to_owned())?;
    let decoded = STANDARD
        .decode(encoded)
        .map_err(|error| format!("decode OCI Vault secret base64 content: {error}"))?;
    let value = String::from_utf8(decoded)
        .map_err(|error| format!("OCI Vault secret content is not valid UTF-8: {error}"))?;
    Ok(VaultSecretValue::new(value, bundle.version_number))
}

impl SecretsEndpoint {
    fn from_env_or_region(auth: &OciInstancePrincipalClient) -> Self {
        // ast-grep-ignore: no-distributed-env-read
        if let Ok(configured) = std::env::var("OCI_SECRETS_ENDPOINT")
            && !configured.trim().is_empty()
        {
            return Self::parse(&configured);
        }

        Self {
            host: format!(
                "secrets.vaults.{}.oci.{}",
                auth.region(),
                auth.realm_domain()
            ),
            base_path: String::new(),
        }
    }

    fn parse(value: &str) -> Self {
        let trimmed = value.trim().trim_end_matches('/');
        let without_scheme = trimmed
            .strip_prefix("https://")
            .unwrap_or_else(|| panic!("OCI_SECRETS_ENDPOINT must be an https URL, got {value}"));
        let (host, base_path) = match without_scheme.split_once('/') {
            Some((host, path)) => (host, format!("/{path}")),
            None => (without_scheme, String::new()),
        };
        assert!(!host.is_empty(), "OCI_SECRETS_ENDPOINT host is empty");
        Self {
            host: host.to_owned(),
            base_path,
        }
    }

    fn current_secret_bundle_path(&self, secret_id: &str) -> String {
        format!(
            "{}/20190301/secretbundles/{secret_id}?stage=CURRENT",
            self.base_path
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_endpoint_parses_host_and_base_path() {
        assert_eq!(
            SecretsEndpoint::parse("https://example.test/prefix/"),
            SecretsEndpoint {
                host: "example.test".to_owned(),
                base_path: "/prefix".to_owned(),
            }
        );
        assert_eq!(
            SecretsEndpoint::parse("https://example.test"),
            SecretsEndpoint {
                host: "example.test".to_owned(),
                base_path: String::new(),
            }
        );
    }

    #[test]
    fn secret_retrieval_explicitly_requests_the_current_version() {
        let endpoint = SecretsEndpoint {
            host: "example.test".to_owned(),
            base_path: "/prefix".to_owned(),
        };

        assert_eq!(
            endpoint.current_secret_bundle_path("ocid1.vaultsecret.example"),
            "/prefix/20190301/secretbundles/ocid1.vaultsecret.example?stage=CURRENT"
        );
    }

    #[test]
    fn secret_bundle_decoding_retains_numeric_version_metadata() {
        let bundle: SecretBundle = serde_json::from_str(
            r#"{
                "versionNumber": 17,
                "secretBundleContent": {
                    "contentType": "BASE64",
                    "content": "c2VjcmV0LXZhbHVl"
                }
            }"#,
        )
        .expect("decode test secret bundle");

        let secret = decode_secret_bundle(bundle).expect("decode secret content");

        assert_eq!(secret.expose_secret(), "secret-value");
        assert_eq!(secret.version_number(), 17);
    }
}
