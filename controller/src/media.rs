use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use tokio::fs;

#[async_trait]
pub trait PrivateMediaStore: Send + Sync {
    async fn write(&self, object_key: &str, body: &[u8]) -> Result<(), String>;
    async fn read(&self, object_key: &str) -> Result<Vec<u8>, String>;
}

#[derive(Clone)]
pub struct LocalMediaStore {
    root: Arc<PathBuf>,
}

impl LocalMediaStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Arc::new(root.into()),
        }
    }

    fn path_for(&self, object_key: &str) -> Result<PathBuf, String> {
        let relative = Path::new(object_key);
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err("object key must stay within the configured media root".to_owned());
        }
        Ok(self.root.join(relative))
    }
}

#[async_trait]
impl PrivateMediaStore for LocalMediaStore {
    async fn write(&self, object_key: &str, body: &[u8]) -> Result<(), String> {
        let path = self.path_for(object_key)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|error| format!("create media directory: {error}"))?;
        }
        fs::write(path, body)
            .await
            .map_err(|error| format!("write media object: {error}"))
    }

    async fn read(&self, object_key: &str) -> Result<Vec<u8>, String> {
        fs::read(self.path_for(object_key)?)
            .await
            .map_err(|error| format!("read media object: {error}"))
    }
}
