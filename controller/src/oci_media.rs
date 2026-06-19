use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
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

use crate::media::PrivateMediaStore;

const METADATA_BASE_URL: &str = "http://169.254.169.254/opc/v2";
const METADATA_AUTHORIZATION: &str = "Bearer Oracle";
const REFRESH_WINDOW: Duration = Duration::from_secs(300);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct OciInstancePrincipalMediaStore {
    client: Client,
    namespace: Arc<String>,
    bucket_name: Arc<String>,
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

impl OciInstancePrincipalMediaStore {
    pub fn new(namespace: String, bucket_name: String) -> Result<Self, String> {
        let auth_mode = std::env::var("OCI_AUTH_MODE").unwrap_or_default();
        if auth_mode != "instance_principal" {
            return Err("OCI_AUTH_MODE=instance_principal is required".to_owned());
        }
        let region = std::env::var("OCI_REGION").unwrap_or_else(|_| "us-ashburn-1".to_owned());
        let realm_domain =
            std::env::var("OCI_REALM_DOMAIN").unwrap_or_else(|_| "oraclecloud.com".to_owned());

        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| format!("configure OCI HTTP client: {error}"))?;

        Ok(Self {
            client,
            namespace: Arc::new(namespace),
            bucket_name: Arc::new(bucket_name),
            region: Arc::new(region),
            realm_domain: Arc::new(realm_domain),
            session: Arc::new(Mutex::new(None)),
        })
    }

    async fn execute(
        &self,
        method: Method,
        object_key: &str,
        body: Option<&[u8]>,
    ) -> Result<reqwest::Response, String> {
        let session = self.session().await?;
        let host = format!("objectstorage.{}.{}", self.region, self.realm_domain);
        let path = format!(
            "/n/{}/b/{}/o/{object_key}",
            self.namespace, self.bucket_name
        );
        let url = format!("https://{host}{path}");
        let headers = sign_headers(
            method.as_str(),
            &path,
            &host,
            body,
            body.map(|_| "application/octet-stream"),
            &session.key_id,
            &session.private_key_pem,
        )?;

        let mut request = self.client.request(method, url).headers(headers);
        if let Some(body) = body {
            request = request.body(body.to_vec());
        }
        let response = request
            .send()
            .await
            .map_err(|error| format!("send OCI Object Storage request: {error}"))?;
        Ok(response)
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

#[async_trait]
impl PrivateMediaStore for OciInstancePrincipalMediaStore {
    async fn write(&self, object_key: &str, body: &[u8]) -> Result<(), String> {
        let response = self
            .execute(Method::PUT, object_key, Some(body))
            .await
            .map_err(|error| format!("write OCI private media object: {error}"))?;
        require_success(response, "write OCI private media object")
            .await
            .map(|_| ())
    }

    async fn read(&self, object_key: &str) -> Result<Vec<u8>, String> {
        let response = self
            .execute(Method::GET, object_key, None)
            .await
            .map_err(|error| format!("read OCI private media object: {error}"))?;
        let response = require_success(response, "read OCI private media object").await?;
        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|error| format!("read OCI private media bytes: {error}"))
    }

    async fn delete(&self, object_key: &str) -> Result<(), String> {
        let response = self
            .execute(Method::DELETE, object_key, None)
            .await
            .map_err(|error| format!("delete OCI private media object: {error}"))?;
        if response.status().as_u16() == 404 {
            return Ok(());
        }
        require_success(response, "delete OCI private media object")
            .await
            .map(|_| ())
    }
}

async fn require_success(
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
