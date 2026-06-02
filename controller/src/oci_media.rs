use std::sync::Arc;

use async_trait::async_trait;
use s3::{Bucket, creds::Credentials, region::Region};

use crate::media::PrivateMediaStore;

#[derive(Clone)]
pub struct OciS3MediaStore {
    bucket: Arc<Bucket>,
}

impl OciS3MediaStore {
    pub fn new(
        bucket_name: String,
        region: String,
        endpoint: String,
        access_key: String,
        secret_key: String,
    ) -> Result<Self, String> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)
            .map_err(|error| format!("configure OCI Customer Secret credentials: {error}"))?;
        let bucket = Bucket::new(
            &bucket_name,
            Region::Custom { region, endpoint },
            credentials,
        )
        .map_err(|error| format!("configure OCI S3-compatible bucket: {error}"))?
        .with_path_style();
        Ok(Self {
            bucket: Arc::from(bucket),
        })
    }
}

#[async_trait]
impl PrivateMediaStore for OciS3MediaStore {
    async fn write(&self, object_key: &str, body: &[u8]) -> Result<(), String> {
        let response = self
            .bucket
            .put_object(object_key, body)
            .await
            .map_err(|error| format!("write OCI private media object: {error}"))?;
        if !(200..300).contains(&response.status_code()) {
            return Err(format!(
                "write OCI private media object returned status {}",
                response.status_code()
            ));
        }
        Ok(())
    }

    async fn read(&self, object_key: &str) -> Result<Vec<u8>, String> {
        let response = self
            .bucket
            .get_object(object_key)
            .await
            .map_err(|error| format!("read OCI private media object: {error}"))?;
        if !(200..300).contains(&response.status_code()) {
            return Err(format!(
                "read OCI private media object returned status {}",
                response.status_code()
            ));
        }
        Ok(response.bytes().to_vec())
    }
}
