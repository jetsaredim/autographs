use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{
    Engine as _, engine::general_purpose::STANDARD, engine::general_purpose::URL_SAFE_NO_PAD,
};
use reqwest::{
    Client, Method,
    header::{HeaderMap, HeaderValue},
};
use rsa::{
    RsaPrivateKey,
    pkcs1::DecodeRsaPrivateKey,
    pkcs1v15::SigningKey,
    pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    rand_core::OsRng,
    signature::{SignatureEncoding, Signer},
};
use serde::Deserialize;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use x509_parser::{parse_x509_certificate, pem::parse_x509_pem};

const METADATA_BASE_URL: &str = "http://169.254.169.254/opc/v2";
const METADATA_AUTHORIZATION: &str = "Bearer Oracle";
const REFRESH_WINDOW: Duration = Duration::from_secs(300);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct OciInstancePrincipalClient {
    client: Client,
    region: Arc<String>,
    realm_domain: Arc<String>,
    session: Arc<Mutex<Option<InstancePrincipalSession>>>,
}

#[derive(Clone)]
struct InstancePrincipalSession {
    key_id: String,
    private_key_pem: String,
    expires_at: SystemTime,
}

#[derive(Deserialize)]
struct FederationResponse {
    token: String,
}

impl OciInstancePrincipalClient {
    pub fn new() -> Result<Self, String> {
        // ast-grep-ignore: no-distributed-env-read
        let auth_mode = std::env::var("OCI_AUTH_MODE").unwrap_or_default();
        if auth_mode != "instance_principal" {
            return Err("OCI_AUTH_MODE=instance_principal is required".to_owned());
        }
        // ast-grep-ignore: no-distributed-env-read
        let region = std::env::var("OCI_REGION").unwrap_or_else(|_| "us-ashburn-1".to_owned());
        let realm_domain = {
            // ast-grep-ignore: no-distributed-env-read
            std::env::var("OCI_REALM_DOMAIN").unwrap_or_else(|_| "oraclecloud.com".to_owned())
        };

        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| format!("configure OCI HTTP client: {error}"))?;

        Ok(Self {
            client,
            region: Arc::new(region),
            realm_domain: Arc::new(realm_domain),
            session: Arc::new(Mutex::new(None)),
        })
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn realm_domain(&self) -> &str {
        &self.realm_domain
    }

    pub async fn execute(
        &self,
        method: Method,
        host: &str,
        path: &str,
        body: Option<&[u8]>,
        content_type: Option<&str>,
    ) -> Result<reqwest::Response, String> {
        let session = self.session().await?;
        let url = format!("https://{host}{path}");
        let headers = sign_headers(
            method.as_str(),
            path,
            host,
            body,
            content_type,
            &session.key_id,
            &session.private_key_pem,
        )?;

        let mut request = self.client.request(method, url).headers(headers);
        if let Some(body) = body {
            request = request.body(body.to_vec());
        }
        request
            .send()
            .await
            .map_err(|error| format!("send OCI signed request: {error}"))
    }

    async fn session(&self) -> Result<InstancePrincipalSession, String> {
        let mut guard = self.session.lock().await;
        if let Some(session) = guard.as_ref() {
            let refresh_at = session
                .expires_at
                .checked_sub(REFRESH_WINDOW)
                .unwrap_or(UNIX_EPOCH);
            if SystemTime::now() < refresh_at {
                return Ok(session.clone());
            }
        }

        let session = self.refresh_session().await?;
        *guard = Some(session.clone());
        Ok(session)
    }

    async fn refresh_session(&self) -> Result<InstancePrincipalSession, String> {
        let leaf_cert = self.metadata_text("/identity/cert.pem").await?;
        let leaf_key = self.metadata_text("/identity/key.pem").await?;
        let intermediate_cert = self.metadata_text("/identity/intermediate.pem").await?;
        let tenancy_id = tenancy_id_from_cert(&leaf_cert)?;

        let session_private_key = RsaPrivateKey::new(&mut OsRng, 2048).map_err(|error| {
            format!("generate OCI instance-principal session private key: {error}")
        })?;
        let session_private_key_pem = session_private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|error| format!("encode OCI instance-principal session key: {error}"))?
            .to_string();
        let session_public_key_pem = session_private_key
            .to_public_key()
            .to_public_key_pem(LineEnding::LF)
            .map_err(|error| format!("encode OCI instance-principal public key: {error}"))?;

        let auth_host = format!("auth.{}.{}", self.region, self.realm_domain);
        let auth_path = "/v1/x509";
        let auth_key_id = format!("{tenancy_id}/fed-x509/{}", cert_fingerprint(&leaf_cert)?);
        let body = serde_json::json!({
            "certificate": sanitize_pem(&leaf_cert),
            "publicKey": sanitize_pem(&session_public_key_pem),
            "intermediateCertificates": [sanitize_pem(&intermediate_cert)],
        })
        .to_string();
        let headers = sign_headers(
            "POST",
            auth_path,
            &auth_host,
            Some(body.as_bytes()),
            Some("application/json"),
            &auth_key_id,
            &leaf_key,
        )?;
        let response = self
            .client
            .post(format!("https://{auth_host}{auth_path}"))
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|error| format!("request OCI federation token: {error}"))?;
        let response = require_success(response, "request OCI federation token").await?;
        let FederationResponse { token } = response
            .json()
            .await
            .map_err(|error| format!("decode OCI federation token: {error}"))?;
        let expires_at = jwt_expiration(&token)?;

        Ok(InstancePrincipalSession {
            key_id: format!("ST${token}"),
            private_key_pem: session_private_key_pem,
            expires_at,
        })
    }

    async fn metadata_text(&self, path: &str) -> Result<String, String> {
        let response = self
            .client
            .get(format!("{METADATA_BASE_URL}{path}"))
            .header("authorization", METADATA_AUTHORIZATION)
            .send()
            .await
            .map_err(|error| format!("request OCI instance metadata {path}: {error}"))?;
        let response = require_success(response, "request OCI instance metadata").await?;
        response
            .text()
            .await
            .map_err(|error| format!("read OCI instance metadata {path}: {error}"))
    }
}

pub async fn require_success(
    response: reqwest::Response,
    context: &str,
) -> Result<reqwest::Response, String> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Err(format!("{context} returned status {status}: {body}"))
}

fn sign_headers(
    method: &str,
    path: &str,
    host: &str,
    body: Option<&[u8]>,
    content_type: Option<&str>,
    key_id: &str,
    private_key_pem: &str,
) -> Result<HeaderMap, String> {
    let date = httpdate::fmt_http_date(SystemTime::now());
    let mut signing_lines = vec![
        format!("date: {date}"),
        format!("(request-target): {} {path}", method.to_lowercase()),
        format!("host: {host}"),
    ];
    let mut signed_headers = vec!["date", "(request-target)", "host"];
    let mut headers = HeaderMap::new();
    headers.insert(
        "date",
        HeaderValue::from_str(&date).map_err(|error| format!("set date header: {error}"))?,
    );
    headers.insert(
        "host",
        HeaderValue::from_str(host).map_err(|error| format!("set host header: {error}"))?,
    );

    if let Some(body) = body {
        let content_type = content_type.unwrap_or("application/octet-stream");
        let content_length = body.len().to_string();
        let mut hasher = Sha256::new();
        hasher.update(body);
        let body_sha256 = STANDARD.encode(hasher.finalize());

        signing_lines.push(format!("content-length: {content_length}"));
        signing_lines.push(format!("content-type: {content_type}"));
        signing_lines.push(format!("x-content-sha256: {body_sha256}"));
        signed_headers.extend(["content-length", "content-type", "x-content-sha256"]);

        headers.insert(
            "content-length",
            HeaderValue::from_str(&content_length)
                .map_err(|error| format!("set content-length header: {error}"))?,
        );
        headers.insert(
            "content-type",
            HeaderValue::from_str(content_type)
                .map_err(|error| format!("set content-type header: {error}"))?,
        );
        headers.insert(
            "x-content-sha256",
            HeaderValue::from_str(&body_sha256)
                .map_err(|error| format!("set x-content-sha256 header: {error}"))?,
        );
    }

    let private_key = parse_private_key(private_key_pem)?;
    let signing_key = SigningKey::<Sha256>::new(private_key);
    let signature = signing_key.sign(signing_lines.join("\n").as_bytes());
    let authorization = format!(
        "Signature version=\"1\",headers=\"{}\",keyId=\"{}\",algorithm=\"rsa-sha256\",signature=\"{}\"",
        signed_headers.join(" "),
        key_id,
        STANDARD.encode(signature.to_bytes())
    );
    headers.insert(
        "authorization",
        HeaderValue::from_str(&authorization)
            .map_err(|error| format!("set authorization header: {error}"))?,
    );
    Ok(headers)
}

fn parse_private_key(private_key_pem: &str) -> Result<RsaPrivateKey, String> {
    RsaPrivateKey::from_pkcs8_pem(private_key_pem)
        .or_else(|_| RsaPrivateKey::from_pkcs1_pem(private_key_pem))
        .map_err(|error| format!("parse OCI private key: {error}"))
}

fn sanitize_pem(value: &str) -> String {
    value
        .replace("-----BEGIN CERTIFICATE-----", "")
        .replace("-----END CERTIFICATE-----", "")
        .replace("-----BEGIN PUBLIC KEY-----", "")
        .replace("-----END PUBLIC KEY-----", "")
        .replace('\n', "")
}

fn cert_fingerprint(cert_pem: &str) -> Result<String, String> {
    let der = STANDARD
        .decode(sanitize_pem(cert_pem))
        .map_err(|error| format!("decode OCI certificate: {error}"))?;
    let digest = Sha1::digest(der);
    Ok(digest
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":"))
}

fn tenancy_id_from_cert(cert_pem: &str) -> Result<String, String> {
    let (_, pem) = parse_x509_pem(cert_pem.as_bytes())
        .map_err(|error| format!("parse OCI certificate PEM: {error}"))?;
    let (_, cert) = parse_x509_certificate(&pem.contents)
        .map_err(|error| format!("parse OCI certificate DER: {error}"))?;
    let mut fallback = None;
    for attr in cert.subject().iter_attributes() {
        let value = attr
            .as_str()
            .map_err(|error| format!("decode OCI certificate subject: {error}"))?;
        if let Some(tenancy) = value.strip_prefix("opc-tenant:") {
            return Ok(tenancy.to_owned());
        }
        if let Some(tenancy) = value.strip_prefix("opc-identity:") {
            fallback = Some(tenancy.to_owned());
        }
    }
    fallback.ok_or_else(|| {
        "OCI certificate subject does not contain an opc-tenant or opc-identity value".to_owned()
    })
}

fn jwt_expiration(token: &str) -> Result<SystemTime, String> {
    let payload = token
        .split('.')
        .nth(1)
        .ok_or_else(|| "OCI federation token payload is missing".to_owned())?;
    let decoded = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|error| format!("decode OCI federation token payload: {error}"))?;
    let value: serde_json::Value = serde_json::from_slice(&decoded)
        .map_err(|error| format!("parse OCI federation token payload: {error}"))?;
    let exp = value
        .get("exp")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| "OCI federation token exp claim is missing".to_owned())?;
    Ok(UNIX_EPOCH + Duration::from_secs(exp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_requires_instance_principal_auth_mode() {
        with_env_removed("OCI_AUTH_MODE");
        assert_eq!(
            OciInstancePrincipalClient::new().err().unwrap(),
            "OCI_AUTH_MODE=instance_principal is required"
        );

        set_env_for_test("OCI_AUTH_MODE", "api_key");
        assert_eq!(
            OciInstancePrincipalClient::new().err().unwrap(),
            "OCI_AUTH_MODE=instance_principal is required"
        );

        with_env_removed("OCI_AUTH_MODE");
    }

    #[test]
    fn new_uses_region_and_realm_defaults() {
        set_env_for_test("OCI_AUTH_MODE", "instance_principal");
        with_env_removed("OCI_REGION");
        with_env_removed("OCI_REALM_DOMAIN");

        let client = OciInstancePrincipalClient::new().expect("client builds");
        assert_eq!(client.region(), "us-ashburn-1");
        assert_eq!(client.realm_domain(), "oraclecloud.com");

        with_env_removed("OCI_AUTH_MODE");
    }

    #[test]
    fn new_uses_configured_region_and_realm() {
        set_env_for_test("OCI_AUTH_MODE", "instance_principal");
        set_env_for_test("OCI_REGION", "eu-frankfurt-1");
        set_env_for_test("OCI_REALM_DOMAIN", "oraclecloud.eu");

        let client = OciInstancePrincipalClient::new().expect("client builds");
        assert_eq!(client.region(), "eu-frankfurt-1");
        assert_eq!(client.realm_domain(), "oraclecloud.eu");

        with_env_removed("OCI_AUTH_MODE");
        with_env_removed("OCI_REGION");
        with_env_removed("OCI_REALM_DOMAIN");
    }

    #[test]
    fn sanitize_pem_removes_wrappers_and_line_breaks() {
        let value = "-----BEGIN CERTIFICATE-----\nabc\n123\n-----END CERTIFICATE-----\n";
        assert_eq!(sanitize_pem(value), "abc123");

        let value = "-----BEGIN PUBLIC KEY-----\nxyz\n-----END PUBLIC KEY-----\n";
        assert_eq!(sanitize_pem(value), "xyz");
    }

    #[test]
    fn jwt_expiration_reads_exp_claim() {
        let payload = URL_SAFE_NO_PAD.encode(r#"{"exp":42}"#);
        let expires_at = jwt_expiration(&format!("header.{payload}.signature")).unwrap();
        assert_eq!(expires_at, UNIX_EPOCH + Duration::from_secs(42));
    }

    #[test]
    fn jwt_expiration_rejects_missing_or_invalid_payloads() {
        assert!(jwt_expiration("not-a-jwt").unwrap_err().contains("payload"));

        let payload = URL_SAFE_NO_PAD.encode(r#"{"sub":"instance"}"#);
        assert!(
            jwt_expiration(&format!("header.{payload}.signature"))
                .unwrap_err()
                .contains("exp claim")
        );
    }

    #[test]
    fn sign_headers_adds_required_get_signature_headers() {
        let private_key_pem = generated_private_key_pem();
        let headers = sign_headers(
            "GET",
            "/n/example/b/bucket/o/object",
            "objectstorage.us-ashburn-1.oraclecloud.com",
            None,
            None,
            "test-key",
            &private_key_pem,
        )
        .expect("headers sign");

        assert!(headers.contains_key("date"));
        assert_eq!(
            headers.get("host").unwrap(),
            "objectstorage.us-ashburn-1.oraclecloud.com"
        );
        let authorization = headers
            .get("authorization")
            .expect("authorization header")
            .to_str()
            .unwrap();
        assert!(authorization.contains("headers=\"date (request-target) host\""));
        assert!(authorization.contains("keyId=\"test-key\""));
        assert!(authorization.contains("algorithm=\"rsa-sha256\""));
    }

    #[test]
    fn sign_headers_adds_body_digest_headers() {
        let private_key_pem = generated_private_key_pem();
        let headers = sign_headers(
            "POST",
            "/v1/x509",
            "auth.us-ashburn-1.oraclecloud.com",
            Some(br#"{"certificate":"redacted"}"#),
            Some("application/json"),
            "test-key",
            &private_key_pem,
        )
        .expect("headers sign");

        assert_eq!(headers.get("content-type").unwrap(), "application/json");
        assert_eq!(headers.get("content-length").unwrap(), "26");
        assert!(headers.contains_key("x-content-sha256"));
        let authorization = headers
            .get("authorization")
            .expect("authorization header")
            .to_str()
            .unwrap();
        assert!(authorization.contains(
            "headers=\"date (request-target) host content-length content-type x-content-sha256\""
        ));
    }

    #[test]
    fn parse_private_key_rejects_invalid_pem() {
        assert!(
            parse_private_key("not a private key")
                .unwrap_err()
                .contains("parse OCI private key")
        );
    }

    fn generated_private_key_pem() -> String {
        RsaPrivateKey::new(&mut OsRng, 2048)
            .expect("test key generation")
            .to_pkcs8_pem(LineEnding::LF)
            .expect("test key encoding")
            .to_string()
    }

    fn set_env_for_test(name: &str, value: &str) {
        // SAFETY: These unit tests update only task-specific OCI env vars and
        // clean up after themselves.
        unsafe {
            std::env::set_var(name, value);
        }
    }

    fn with_env_removed(name: &str) {
        // SAFETY: These unit tests update only task-specific OCI env vars and
        // clean up after themselves.
        unsafe {
            std::env::remove_var(name);
        }
    }
}
