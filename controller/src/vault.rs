use std::{env, fs};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::blocking::Client;
use ring::{
    rand::SystemRandom,
    signature::{RSA_PKCS1_SHA256, RsaKeyPair},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use time::{Month, OffsetDateTime, Weekday};

#[derive(Clone, Debug)]
pub struct OciApiKeyAuth {
    tenancy_ocid: String,
    user_ocid: String,
    fingerprint: String,
    private_key_pem: String,
}

#[derive(Clone, Debug)]
pub struct OciVaultConfig {
    region: String,
    vault_id: String,
    auth: OciApiKeyAuth,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecretBundle {
    secret_bundle_content: SecretBundleContent,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecretBundleContent {
    content: String,
    content_type: String,
}

impl OciVaultConfig {
    pub fn from_env() -> Result<Option<Self>, String> {
        let Some(vault_id) = optional_env("OCI_ADMIN_VAULT_ID") else {
            return Ok(None);
        };

        let private_key_pem = match optional_env("OCI_PRIVATE_KEY_PEM") {
            Some(value) => value,
            None => {
                let path = required_env("OCI_PRIVATE_KEY_PATH")?;
                fs::read_to_string(&path)
                    .map_err(|error| format!("read OCI private key from {path}: {error}"))?
            }
        };

        Ok(Some(Self {
            region: env::var("OCI_REGION").unwrap_or_else(|_| "us-ashburn-1".to_owned()),
            vault_id,
            auth: OciApiKeyAuth {
                tenancy_ocid: required_env("OCI_TENANCY_OCID")?,
                user_ocid: required_env("OCI_CLI_USER_OCID")?,
                fingerprint: required_env("OCI_FINGERPRINT")?,
                private_key_pem,
            },
        }))
    }

    pub fn get_secret_by_name(&self, secret_name: &str) -> Result<String, String> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let endpoint = format!("https://secrets.vaults.{}.oci.oraclecloud.com", self.region);
        let request_target = "/20190301/secretbundles/actions/getByName";
        let request_query = format!(
            "secretName={}&vaultId={}&stage=CURRENT",
            percent_encode(secret_name),
            percent_encode(&self.vault_id)
        );
        let signed_target = format!("{request_target}?{request_query}");
        let url = format!("{endpoint}{signed_target}");
        let host = format!("secrets.vaults.{}.oci.oraclecloud.com", self.region);
        let body = "";
        let date = http_date();
        let content_sha256 = STANDARD.encode(Sha256::digest(body.as_bytes()));
        let content_length = body.len().to_string();
        let signing_string = format!(
            "date: {date}\n(request-target): post {signed_target}\nhost: {host}\nx-content-sha256: {content_sha256}\ncontent-type: application/json\ncontent-length: {content_length}"
        );
        let signature = self.auth.sign(&signing_string)?;
        let authorization = format!(
            "Signature version=\"1\",keyId=\"{}/{}/{}\",algorithm=\"rsa-sha256\",headers=\"date (request-target) host x-content-sha256 content-type content-length\",signature=\"{}\"",
            self.auth.tenancy_ocid, self.auth.user_ocid, self.auth.fingerprint, signature
        );

        let response = Client::new()
            .post(url)
            .header("date", date)
            .header("host", host)
            .header("x-content-sha256", content_sha256)
            .header("content-type", "application/json")
            .header("content-length", content_length)
            .header("authorization", authorization)
            .body(body.to_owned())
            .send()
            .map_err(|error| format!("fetch OCI Vault secret {secret_name}: {error}"))?;
        let status = response.status();
        let response_body = response
            .text()
            .map_err(|error| format!("read OCI Vault secret {secret_name} response: {error}"))?;
        if !status.is_success() {
            return Err(format!(
                "fetch OCI Vault secret {secret_name} returned status {status}: {response_body}"
            ));
        }

        let bundle: SecretBundle = serde_json::from_str(&response_body)
            .map_err(|error| format!("parse OCI Vault secret {secret_name} response: {error}"))?;
        if bundle.secret_bundle_content.content_type != "BASE64" {
            return Err(format!(
                "OCI Vault secret {secret_name} uses unsupported content type {}",
                bundle.secret_bundle_content.content_type
            ));
        }
        let bytes = STANDARD
            .decode(bundle.secret_bundle_content.content)
            .map_err(|error| format!("decode OCI Vault secret {secret_name}: {error}"))?;
        String::from_utf8(bytes)
            .map(|value| value.trim().to_owned())
            .map_err(|error| format!("decode OCI Vault secret {secret_name} as UTF-8: {error}"))
    }
}

impl OciApiKeyAuth {
    fn sign(&self, signing_string: &str) -> Result<String, String> {
        let key_der = decode_private_key_pem(&self.private_key_pem)?;
        let key = RsaKeyPair::from_pkcs8(&key_der)
            .map_err(|error| format!("parse OCI private key: {error}"))?;
        let rng = SystemRandom::new();
        let mut signature = vec![0; key.public().modulus_len()];
        key.sign(
            &RSA_PKCS1_SHA256,
            &rng,
            signing_string.as_bytes(),
            &mut signature,
        )
        .map_err(|error| format!("sign OCI request: {error}"))?;
        Ok(STANDARD.encode(signature))
    }
}

fn decode_private_key_pem(pem: &str) -> Result<Vec<u8>, String> {
    let body = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<String>();
    STANDARD
        .decode(body)
        .map_err(|error| format!("decode OCI private key PEM: {error}"))
}

fn required_env(name: &str) -> Result<String, String> {
    optional_env(name).ok_or_else(|| format!("{name} is required for OCI Vault secret lookup"))
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn http_date() -> String {
    let now = OffsetDateTime::now_utc();
    format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
        weekday(now.weekday()),
        now.day(),
        month(now.month()),
        now.year(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

fn weekday(value: Weekday) -> &'static str {
    match value {
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
        Weekday::Sunday => "Sun",
    }
}

fn month(value: Month) -> &'static str {
    match value {
        Month::January => "Jan",
        Month::February => "Feb",
        Month::March => "Mar",
        Month::April => "Apr",
        Month::May => "May",
        Month::June => "Jun",
        Month::July => "Jul",
        Month::August => "Aug",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    }
}
