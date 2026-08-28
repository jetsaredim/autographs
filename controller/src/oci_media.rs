use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Method;

use crate::{media::PrivateMediaStore, oci_auth};

#[derive(Clone)]
pub struct OciInstancePrincipalMediaStore {
    auth: oci_auth::OciInstancePrincipalClient,
    namespace: Arc<String>,
    bucket_name: Arc<String>,
}

impl OciInstancePrincipalMediaStore {
    pub fn new(namespace: String, bucket_name: String) -> Result<Self, String> {
        Ok(Self {
            auth: oci_auth::OciInstancePrincipalClient::new()?,
            namespace: Arc::new(namespace),
            bucket_name: Arc::new(bucket_name),
        })
    }

    async fn execute(
        &self,
        method: Method,
        object_key: &str,
        body: Option<&[u8]>,
    ) -> Result<reqwest::Response, String> {
        let host = format!(
            "objectstorage.{}.{}",
            self.auth.region(),
            self.auth.realm_domain()
        );
        let path = format!(
            "/n/{}/b/{}/o/{object_key}",
            self.namespace, self.bucket_name
        );
        self.auth
            .execute(
                method,
                &host,
                &path,
                body,
                body.map(|_| "application/octet-stream"),
            )
            .await
            .map_err(|error| format!("send OCI Object Storage request: {error}"))
    }
}

#[async_trait]
impl PrivateMediaStore for OciInstancePrincipalMediaStore {
    async fn write(&self, object_key: &str, body: &[u8]) -> Result<(), String> {
        let response = self
            .execute(Method::PUT, object_key, Some(body))
            .await
            .map_err(|error| format!("write OCI private media object: {error}"))?;
        oci_auth::require_success(response, "write OCI private media object")
            .await
            .map(|_| ())
    }

    async fn read(&self, object_key: &str) -> Result<Vec<u8>, String> {
        let response = self
            .execute(Method::GET, object_key, None)
            .await
            .map_err(|error| format!("read OCI private media object: {error}"))?;
        let response = oci_auth::require_success(response, "read OCI private media object").await?;
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
        oci_auth::require_success(response, "delete OCI private media object")
            .await
            .map(|_| ())
    }
}
