use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PublicationStatus {
    Draft,
    Published,
    Archived,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutographItemInput {
    pub title: String,
    pub signer: String,
    pub description: Option<String>,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "draft")]
    pub publication_status: PublicationStatus,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AutographItemUpdate {
    pub title: Option<String>,
    pub signer: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub publication_status: Option<PublicationStatus>,
}

#[derive(Clone, Debug)]
pub struct AutographItem {
    pub id: Uuid,
    pub title: String,
    pub signer: String,
    pub description: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub publication_status: PublicationStatus,
    pub images: Vec<AutographImage>,
}

#[derive(Clone, Debug)]
pub struct AutographImage {
    pub id: Uuid,
    pub object_key: String,
    pub original_filename: String,
    pub content_type: String,
    pub byte_size: usize,
    pub is_primary: bool,
    pub sort_order: i32,
}

#[async_trait]
pub trait CatalogRepository: Send + Sync {
    async fn create(&self, input: AutographItemInput) -> Result<AutographItem, String>;
    async fn update(&self, id: Uuid, input: AutographItemUpdate) -> Result<AutographItem, String>;
    async fn get(&self, id: Uuid) -> Result<Option<AutographItem>, String>;
    async fn list(&self) -> Result<Vec<AutographItem>, String>;
    async fn attach_image(
        &self,
        item_id: Uuid,
        image: AutographImage,
    ) -> Result<AutographItem, String>;
}

#[derive(Clone, Default)]
pub struct MemoryCatalogRepository {
    items: Arc<Mutex<HashMap<Uuid, AutographItem>>>,
}

#[async_trait]
impl CatalogRepository for MemoryCatalogRepository {
    async fn create(&self, input: AutographItemInput) -> Result<AutographItem, String> {
        if input.title.trim().is_empty()
            || input.signer.trim().is_empty()
            || input.category.trim().is_empty()
        {
            return Err("title, signer, and category are required".to_owned());
        }
        let item = AutographItem {
            id: Uuid::new_v4(),
            title: input.title,
            signer: input.signer,
            description: input.description,
            category: input.category,
            tags: input.tags,
            publication_status: input.publication_status,
            images: Vec::new(),
        };
        self.items
            .lock()
            .expect("catalog state lock")
            .insert(item.id, item.clone());
        Ok(item)
    }

    async fn update(&self, id: Uuid, input: AutographItemUpdate) -> Result<AutographItem, String> {
        let mut items = self.items.lock().expect("catalog state lock");
        let item = items
            .get_mut(&id)
            .ok_or_else(|| "autograph item was not found".to_owned())?;
        if let Some(title) = input.title {
            item.title = title;
        }
        if let Some(signer) = input.signer {
            item.signer = signer;
        }
        if let Some(description) = input.description {
            item.description = Some(description);
        }
        if let Some(category) = input.category {
            item.category = category;
        }
        if let Some(tags) = input.tags {
            item.tags = tags;
        }
        if let Some(status) = input.publication_status {
            item.publication_status = status;
        }
        Ok(item.clone())
    }

    async fn get(&self, id: Uuid) -> Result<Option<AutographItem>, String> {
        Ok(self
            .items
            .lock()
            .expect("catalog state lock")
            .get(&id)
            .cloned())
    }

    async fn list(&self) -> Result<Vec<AutographItem>, String> {
        Ok(self
            .items
            .lock()
            .expect("catalog state lock")
            .values()
            .cloned()
            .collect())
    }

    async fn attach_image(
        &self,
        item_id: Uuid,
        image: AutographImage,
    ) -> Result<AutographItem, String> {
        let mut items = self.items.lock().expect("catalog state lock");
        let item = items
            .get_mut(&item_id)
            .ok_or_else(|| "autograph item was not found".to_owned())?;
        if image.is_primary {
            for existing in &mut item.images {
                existing.is_primary = false;
            }
        }
        item.images.push(image);
        Ok(item.clone())
    }
}

const fn draft() -> PublicationStatus {
    PublicationStatus::Draft
}
