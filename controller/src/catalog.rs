use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub(crate) const REQUIRED_FIELDS_ERROR: &str = "title, signer, and category are required";
const DEFAULT_FORMAT: &str = "Trading Card";
const DEFAULT_LANGUAGE: &str = "English";
const MAX_PROFILE_URL_LENGTH: usize = 1000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PublicationStatus {
    Draft,
    Published,
    Archived,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ItemOrigin {
    Official,
    Custom,
}

impl Default for ItemOrigin {
    fn default() -> Self {
        Self::Official
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerProfile {
    pub id: Uuid,
    pub display_name: String,
    pub normalized_name: String,
    pub default_role: Option<String>,
    pub wikipedia_url: Option<String>,
    pub imdb_url: Option<String>,
    pub created_at_epoch_seconds: i64,
    pub updated_at_epoch_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerCredit {
    pub signer: SignerProfile,
    pub sort_order: i32,
    pub item_role: Option<String>,
    pub item_context: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerCreditInput {
    pub signer_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub default_role: Option<String>,
    pub item_role: Option<String>,
    pub item_context: Option<String>,
    pub wikipedia_url: Option<String>,
    pub imdb_url: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerMergeResult {
    pub source_signer_id: Uuid,
    pub target_signer_id: Uuid,
    pub updated_item_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerSuggestion {
    pub profile: SignerProfile,
    pub possible_duplicate: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerProfileUpdateInput {
    pub display_name: Option<String>,
    pub default_role: Option<String>,
    pub wikipedia_url: Option<String>,
    pub imdb_url: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomySuggestions {
    pub signers: Vec<SignerProfile>,
    pub characters: Vec<String>,
    pub formats: Vec<String>,
    pub origins: Vec<ItemOrigin>,
    pub franchises: Vec<String>,
    pub product_lines: Vec<String>,
    pub set_names: Vec<String>,
    pub languages: Vec<String>,
    pub roles: Vec<String>,
    pub tags: Vec<String>,
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
    #[serde(default)]
    pub signer_credits: Vec<SignerCreditInput>,
    #[serde(default)]
    pub characters: Vec<String>,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default)]
    pub origin: ItemOrigin,
    #[serde(default)]
    pub franchises: Vec<String>,
    pub product_line: Option<String>,
    pub set_name: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
    pub object_reference: Option<String>,
    pub event_name: Option<String>,
    pub event_location: Option<String>,
    pub source: Option<String>,
    pub inscription: Option<String>,
    pub certification_company: Option<String>,
    pub certification_id: Option<String>,
    pub estimated_year: Option<i32>,
    #[serde(default = "draft")]
    pub publication_status: PublicationStatus,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum FieldPatch<T> {
    #[default]
    Unchanged,
    Clear,
    Set(T),
}

impl<'de, T> Deserialize<'de> for FieldPatch<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(|value| match value {
            Some(value) => Self::Set(value),
            None => Self::Clear,
        })
    }
}

impl<T> FieldPatch<T> {
    fn map<U, F>(self, map_value: F) -> FieldPatch<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Unchanged => FieldPatch::Unchanged,
            Self::Clear => FieldPatch::Clear,
            Self::Set(value) => FieldPatch::Set(map_value(value)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PublishBoundary {
    pub started_at_epoch_seconds: i64,
    pub included_event_ids: BTreeSet<Uuid>,
}

impl PublishBoundary {
    pub fn conservative(started_at_epoch_seconds: i64) -> Self {
        Self {
            started_at_epoch_seconds,
            included_event_ids: BTreeSet::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AutographItemUpdate {
    pub title: Option<String>,
    pub signer: Option<String>,
    #[serde(default)]
    pub description: FieldPatch<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub signer_credits: Option<Vec<SignerCreditInput>>,
    pub characters: Option<Vec<String>>,
    pub format: Option<String>,
    pub origin: Option<ItemOrigin>,
    pub franchises: Option<Vec<String>>,
    #[serde(default)]
    pub product_line: FieldPatch<String>,
    #[serde(default)]
    pub set_name: FieldPatch<String>,
    pub language: Option<String>,
    #[serde(default)]
    pub object_reference: FieldPatch<String>,
    #[serde(default)]
    pub event_name: FieldPatch<String>,
    #[serde(default)]
    pub event_location: FieldPatch<String>,
    #[serde(default)]
    pub source: FieldPatch<String>,
    #[serde(default)]
    pub inscription: FieldPatch<String>,
    #[serde(default)]
    pub certification_company: FieldPatch<String>,
    #[serde(default)]
    pub certification_id: FieldPatch<String>,
    #[serde(default)]
    pub estimated_year: FieldPatch<i32>,
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
    pub signer_credits: Vec<SignerCredit>,
    pub characters: Vec<String>,
    pub format: String,
    pub origin: ItemOrigin,
    pub franchises: Vec<String>,
    pub product_line: Option<String>,
    pub set_name: Option<String>,
    pub language: String,
    pub object_reference: Option<String>,
    pub event_name: Option<String>,
    pub event_location: Option<String>,
    pub source: Option<String>,
    pub inscription: Option<String>,
    pub certification_company: Option<String>,
    pub certification_id: Option<String>,
    pub estimated_year: Option<i32>,
    pub publication_status: PublicationStatus,
    pub images: Vec<AutographImage>,
    pub created_at_epoch_seconds: i64,
    pub updated_at_epoch_seconds: i64,
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
    pub alt_text: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CleanupStatus {
    Succeeded,
    DeleteFailed,
    RetrySucceeded,
}

impl CleanupStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::DeleteFailed => "deleteFailed",
            Self::RetrySucceeded => "retrySucceeded",
        }
    }
}

impl std::str::FromStr for CleanupStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "succeeded" => Ok(Self::Succeeded),
            "deleteFailed" => Ok(Self::DeleteFailed),
            "retrySucceeded" => Ok(Self::RetrySucceeded),
            _ => Err(format!("unsupported cleanup status: {value}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageCleanupEvent {
    pub id: Uuid,
    pub item_id: Uuid,
    pub image_id: Uuid,
    pub target_object_key: String,
    pub operation: String,
    pub status: CleanupStatus,
    pub admin_message: String,
    pub created_at_epoch_seconds: i64,
    pub resolved_at_epoch_seconds: Option<i64>,
}

impl ImageCleanupEvent {
    pub fn new(
        item_id: Uuid,
        image_id: Uuid,
        target_object_key: impl Into<String>,
        operation: impl Into<String>,
        status: CleanupStatus,
        admin_message: impl Into<String>,
        created_at_epoch_seconds: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            item_id,
            image_id,
            target_object_key: target_object_key.into(),
            operation: operation.into(),
            status,
            admin_message: admin_message.into(),
            created_at_epoch_seconds,
            resolved_at_epoch_seconds: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupWarning {
    pub image_id: Uuid,
    #[serde(skip_serializing)]
    pub target_object_key: String,
    pub operation: String,
    pub status: CleanupStatus,
    pub admin_message: String,
}

#[derive(Clone, Debug)]
pub struct ImageCleanupOutcome {
    pub item: AutographItem,
    pub warning: Option<CleanupWarning>,
}

#[derive(Clone, Debug)]
pub struct ImageReplacementInput {
    pub image: AutographImage,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDiff {
    pub field: String,
    pub before: Value,
    pub after: Value,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EditEventKind {
    Created,
    MetadataUpdated,
    ImageAdded,
    ImageRemoved,
    ImageReplaced,
    PrimaryImageChanged,
    PublicationChanged,
    CleanupChanged,
}

impl EditEventKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::MetadataUpdated => "metadataUpdated",
            Self::ImageAdded => "imageAdded",
            Self::ImageRemoved => "imageRemoved",
            Self::ImageReplaced => "imageReplaced",
            Self::PrimaryImageChanged => "primaryImageChanged",
            Self::PublicationChanged => "publicationChanged",
            Self::CleanupChanged => "cleanupChanged",
        }
    }
}

impl std::str::FromStr for EditEventKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "created" => Ok(Self::Created),
            "metadataUpdated" => Ok(Self::MetadataUpdated),
            "imageAdded" => Ok(Self::ImageAdded),
            "imageRemoved" => Ok(Self::ImageRemoved),
            "imageReplaced" => Ok(Self::ImageReplaced),
            "primaryImageChanged" => Ok(Self::PrimaryImageChanged),
            "publicationChanged" => Ok(Self::PublicationChanged),
            "cleanupChanged" => Ok(Self::CleanupChanged),
            _ => Err(format!("unsupported catalog edit event kind: {value}")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutographEditEvent {
    pub id: Uuid,
    pub item_id: Uuid,
    pub kind: EditEventKind,
    pub summary: String,
    pub field_diffs: Vec<FieldDiff>,
    pub created_at_epoch_seconds: i64,
}

impl AutographEditEvent {
    pub fn new(
        item_id: Uuid,
        kind: EditEventKind,
        summary: impl Into<String>,
        field_diffs: Vec<FieldDiff>,
        created_at_epoch_seconds: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            item_id,
            kind,
            summary: summary.into(),
            field_diffs,
            created_at_epoch_seconds,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingChangeSummary {
    pub count: usize,
    pub oldest_changed_at_epoch_seconds: Option<i64>,
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

    async fn set_primary_image(
        &self,
        _item_id: Uuid,
        _image_id: Uuid,
    ) -> Result<AutographItem, String> {
        Err("primary image selection is not supported by this repository".to_owned())
    }

    async fn remove_image_metadata(
        &self,
        _item_id: Uuid,
        _image_id: Uuid,
    ) -> Result<AutographItem, String> {
        Err("image metadata removal is not supported by this repository".to_owned())
    }

    async fn replace_image_metadata(
        &self,
        _item_id: Uuid,
        _image_id: Uuid,
        _input: ImageReplacementInput,
    ) -> Result<AutographItem, String> {
        Err("image metadata replacement is not supported by this repository".to_owned())
    }

    async fn record_cleanup_event(
        &self,
        event: ImageCleanupEvent,
    ) -> Result<ImageCleanupEvent, String> {
        Ok(event)
    }

    async fn cleanup_warnings(&self, _item_id: Uuid) -> Result<Vec<CleanupWarning>, String> {
        Ok(Vec::new())
    }

    async fn mark_cleanup_retry_succeeded(
        &self,
        _item_id: Uuid,
        _image_id: Uuid,
        _target_object_key: &str,
    ) -> Result<bool, String> {
        Ok(false)
    }

    async fn history(&self, _item_id: Uuid) -> Result<Vec<AutographEditEvent>, String> {
        Ok(Vec::new())
    }

    async fn pending_changes(&self) -> Result<PendingChangeSummary, String> {
        Ok(PendingChangeSummary::default())
    }

    async fn pending_changes_for_item(
        &self,
        _item_id: Uuid,
    ) -> Result<PendingChangeSummary, String> {
        Ok(PendingChangeSummary::default())
    }

    async fn begin_publish_boundary(&self) -> Result<PublishBoundary, String> {
        Ok(PublishBoundary::conservative(now_epoch_seconds()))
    }

    async fn record_successful_publish(
        &self,
        _mode: &str,
        _release_id: Option<&str>,
        _publish_boundary: PublishBoundary,
        _started_at_epoch_seconds: Option<i64>,
        _finished_at_epoch_seconds: i64,
    ) -> Result<(), String> {
        Ok(())
    }

    async fn record_event(&self, event: AutographEditEvent) -> Result<AutographEditEvent, String> {
        Ok(event)
    }

    async fn signer_suggestions(&self, _query: String) -> Result<Vec<SignerSuggestion>, String> {
        Ok(Vec::new())
    }

    async fn taxonomy_suggestions(&self) -> Result<TaxonomySuggestions, String> {
        Ok(TaxonomySuggestions::default())
    }

    async fn update_signer_profile(
        &self,
        _signer_id: Uuid,
        _input: SignerProfileUpdateInput,
    ) -> Result<SignerProfile, String> {
        Err("signer profile updates are not supported by this repository".to_owned())
    }

    async fn merge_signer_profiles(
        &self,
        _source_signer_id: Uuid,
        _target_signer_id: Uuid,
    ) -> Result<SignerMergeResult, String> {
        Err("signer profile merging is not supported by this repository".to_owned())
    }
}

#[derive(Clone)]
pub struct MemoryCatalogRepository {
    items: Arc<Mutex<HashMap<Uuid, AutographItem>>>,
    signers: Arc<Mutex<HashMap<Uuid, SignerProfile>>>,
    events: Arc<Mutex<Vec<AutographEditEvent>>>,
    cleanup_events: Arc<Mutex<Vec<ImageCleanupEvent>>>,
    last_successful_publish_boundary: Arc<Mutex<Option<PublishBoundary>>>,
}

impl Default for MemoryCatalogRepository {
    fn default() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
            signers: Arc::new(Mutex::new(HashMap::new())),
            events: Arc::new(Mutex::new(Vec::new())),
            cleanup_events: Arc::new(Mutex::new(Vec::new())),
            last_successful_publish_boundary: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl CatalogRepository for MemoryCatalogRepository {
    async fn create(&self, input: AutographItemInput) -> Result<AutographItem, String> {
        let now = now_epoch_seconds();
        let signer_credits = {
            let mut signers = self.signers.lock().expect("catalog signer lock");
            resolve_signer_credits(&mut signers, &input.signer_credits, &input.signer, now)?
        };
        validate_item_taxonomy(
            &input.title,
            &input.signer,
            &input.category,
            &signer_credits,
            &input.format,
            &input.language,
        )?;
        let item = AutographItem {
            id: Uuid::new_v4(),
            title: input.title,
            signer: input.signer,
            description: input.description,
            category: input.category,
            tags: normalize_unique_string_list(input.tags),
            signer_credits,
            characters: normalize_unique_string_list(input.characters),
            format: input.format,
            origin: input.origin,
            franchises: normalize_unique_string_list(input.franchises),
            product_line: normalize_optional_string(input.product_line),
            set_name: normalize_optional_string(input.set_name),
            language: input.language,
            object_reference: input.object_reference,
            event_name: input.event_name,
            event_location: input.event_location,
            source: input.source,
            inscription: input.inscription,
            certification_company: input.certification_company,
            certification_id: input.certification_id,
            estimated_year: input.estimated_year,
            publication_status: input.publication_status,
            images: Vec::new(),
            created_at_epoch_seconds: now,
            updated_at_epoch_seconds: now,
        };
        self.items
            .lock()
            .expect("catalog state lock")
            .insert(item.id, item.clone());
        self.events
            .lock()
            .expect("catalog event lock")
            .push(AutographEditEvent::new(
                item.id,
                EditEventKind::Created,
                format!("Created autograph item `{}`", item.title),
                Vec::new(),
                now,
            ));
        Ok(item)
    }

    async fn update(&self, id: Uuid, input: AutographItemUpdate) -> Result<AutographItem, String> {
        let now = now_epoch_seconds();
        let mut event = None;
        let resolved_signer_credits = if let Some(signer_inputs) = input.signer_credits.as_ref() {
            let fallback_signer = {
                let items = self.items.lock().expect("catalog state lock");
                items
                    .get(&id)
                    .map(|item| item.signer.clone())
                    .ok_or_else(|| "autograph item was not found".to_owned())?
            };
            let mut signers = self.signers.lock().expect("catalog signer lock");
            Some(resolve_signer_credits(
                &mut signers,
                signer_inputs,
                &fallback_signer,
                now,
            )?)
        } else {
            None
        };
        let updated = {
            let mut items = self.items.lock().expect("catalog state lock");
            let item = items
                .get_mut(&id)
                .ok_or_else(|| "autograph item was not found".to_owned())?;
            let mut candidate = item.clone();
            let field_diffs = apply_update(&mut candidate, input, resolved_signer_credits);
            validate_item_taxonomy(
                &candidate.title,
                &candidate.signer,
                &candidate.category,
                &candidate.signer_credits,
                &candidate.format,
                &candidate.language,
            )?;
            if !field_diffs.is_empty() {
                candidate.updated_at_epoch_seconds = now;
                let kind = event_kind_for_diffs(&field_diffs);
                event = Some(AutographEditEvent::new(
                    id,
                    kind,
                    event_summary(kind, &field_diffs),
                    field_diffs,
                    now,
                ));
            }
            *item = candidate.clone();
            candidate
        };
        if let Some(event) = event {
            self.events.lock().expect("catalog event lock").push(event);
        }
        Ok(updated)
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
        let now = now_epoch_seconds();
        let updated = {
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
            item.updated_at_epoch_seconds = now;
            item.clone()
        };
        self.events
            .lock()
            .expect("catalog event lock")
            .push(AutographEditEvent::new(
                item_id,
                EditEventKind::ImageAdded,
                "Image added",
                Vec::new(),
                now,
            ));
        Ok(updated)
    }

    async fn set_primary_image(
        &self,
        item_id: Uuid,
        image_id: Uuid,
    ) -> Result<AutographItem, String> {
        let now = now_epoch_seconds();
        let updated = {
            let mut items = self.items.lock().expect("catalog state lock");
            let item = items
                .get_mut(&item_id)
                .ok_or_else(|| "autograph item was not found".to_owned())?;
            let image = item
                .images
                .iter_mut()
                .find(|image| image.id == image_id)
                .ok_or_else(|| "autograph image was not found".to_owned())?;
            if !image.is_primary {
                for image in &mut item.images {
                    image.is_primary = image.id == image_id;
                }
                item.updated_at_epoch_seconds = now;
            }
            item.clone()
        };
        self.events
            .lock()
            .expect("catalog event lock")
            .push(AutographEditEvent::new(
                item_id,
                EditEventKind::PrimaryImageChanged,
                "Primary image changed",
                Vec::new(),
                now,
            ));
        Ok(updated)
    }

    async fn remove_image_metadata(
        &self,
        item_id: Uuid,
        image_id: Uuid,
    ) -> Result<AutographItem, String> {
        let now = now_epoch_seconds();
        let updated = {
            let mut items = self.items.lock().expect("catalog state lock");
            let item = items
                .get_mut(&item_id)
                .ok_or_else(|| "autograph item was not found".to_owned())?;
            let position = item
                .images
                .iter()
                .position(|image| image.id == image_id)
                .ok_or_else(|| "autograph image was not found".to_owned())?;
            let was_primary = item.images[position].is_primary;
            item.images.remove(position);
            if was_primary
                && !item.images.iter().any(|image| image.is_primary)
                && let Some(first) = item.images.first_mut()
            {
                first.is_primary = true;
            }
            item.updated_at_epoch_seconds = now;
            item.clone()
        };
        self.events
            .lock()
            .expect("catalog event lock")
            .push(AutographEditEvent::new(
                item_id,
                EditEventKind::ImageRemoved,
                "Image removed",
                Vec::new(),
                now,
            ));
        Ok(updated)
    }

    async fn replace_image_metadata(
        &self,
        item_id: Uuid,
        image_id: Uuid,
        input: ImageReplacementInput,
    ) -> Result<AutographItem, String> {
        let now = now_epoch_seconds();
        let updated = {
            let mut items = self.items.lock().expect("catalog state lock");
            let item = items
                .get_mut(&item_id)
                .ok_or_else(|| "autograph item was not found".to_owned())?;
            let existing = item
                .images
                .iter_mut()
                .find(|image| image.id == image_id)
                .ok_or_else(|| "autograph image was not found".to_owned())?;
            let mut replacement = input.image;
            replacement.id = existing.id;
            replacement.is_primary = existing.is_primary;
            replacement.sort_order = existing.sort_order;
            *existing = replacement;
            item.updated_at_epoch_seconds = now;
            item.clone()
        };
        self.events
            .lock()
            .expect("catalog event lock")
            .push(AutographEditEvent::new(
                item_id,
                EditEventKind::ImageReplaced,
                "Image replaced",
                Vec::new(),
                now,
            ));
        Ok(updated)
    }

    async fn record_cleanup_event(
        &self,
        event: ImageCleanupEvent,
    ) -> Result<ImageCleanupEvent, String> {
        self.cleanup_events
            .lock()
            .expect("cleanup event lock")
            .push(event.clone());
        self.events
            .lock()
            .expect("catalog event lock")
            .push(AutographEditEvent::new(
                event.item_id,
                EditEventKind::CleanupChanged,
                "Cleanup status changed",
                Vec::new(),
                event.created_at_epoch_seconds,
            ));
        Ok(event)
    }

    async fn cleanup_warnings(&self, item_id: Uuid) -> Result<Vec<CleanupWarning>, String> {
        Ok(self
            .cleanup_events
            .lock()
            .expect("cleanup event lock")
            .iter()
            .filter(|event| event.item_id == item_id && event.status == CleanupStatus::DeleteFailed)
            .map(|event| CleanupWarning {
                image_id: event.image_id,
                target_object_key: event.target_object_key.clone(),
                operation: event.operation.clone(),
                status: event.status,
                admin_message: event.admin_message.clone(),
            })
            .collect())
    }

    async fn mark_cleanup_retry_succeeded(
        &self,
        item_id: Uuid,
        image_id: Uuid,
        target_object_key: &str,
    ) -> Result<bool, String> {
        let now = now_epoch_seconds();
        let mut updated = false;
        for event in self
            .cleanup_events
            .lock()
            .expect("cleanup event lock")
            .iter_mut()
            .filter(|event| {
                event.item_id == item_id
                    && event.image_id == image_id
                    && event.target_object_key == target_object_key
                    && event.status == CleanupStatus::DeleteFailed
            })
        {
            event.status = CleanupStatus::RetrySucceeded;
            event.resolved_at_epoch_seconds = Some(now);
            updated = true;
        }
        if !updated {
            return Ok(false);
        }
        self.events
            .lock()
            .expect("catalog event lock")
            .push(AutographEditEvent::new(
                item_id,
                EditEventKind::CleanupChanged,
                "Cleanup retry succeeded",
                Vec::new(),
                now,
            ));
        Ok(true)
    }

    async fn history(&self, item_id: Uuid) -> Result<Vec<AutographEditEvent>, String> {
        let mut events = self
            .events
            .lock()
            .expect("catalog event lock")
            .iter()
            .filter(|event| event.item_id == item_id)
            .cloned()
            .collect::<Vec<_>>();
        events.sort_by(|left, right| {
            right
                .created_at_epoch_seconds
                .cmp(&left.created_at_epoch_seconds)
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(events)
    }

    async fn pending_changes(&self) -> Result<PendingChangeSummary, String> {
        let events = self.events.lock().expect("catalog event lock");
        let last_successful_publish = self
            .last_successful_publish_boundary
            .lock()
            .expect("publish boundary lock")
            .clone();
        let pending = events
            .iter()
            .filter(|event| is_event_pending(event, last_successful_publish.as_ref()))
            .collect::<Vec<_>>();
        Ok(PendingChangeSummary {
            count: pending.len(),
            oldest_changed_at_epoch_seconds: pending
                .iter()
                .map(|event| event.created_at_epoch_seconds)
                .min(),
        })
    }

    async fn pending_changes_for_item(
        &self,
        item_id: Uuid,
    ) -> Result<PendingChangeSummary, String> {
        let events = self.events.lock().expect("catalog event lock");
        let last_successful_publish = self
            .last_successful_publish_boundary
            .lock()
            .expect("publish boundary lock")
            .clone();
        let pending = events
            .iter()
            .filter(|event| event.item_id == item_id)
            .filter(|event| is_event_pending(event, last_successful_publish.as_ref()))
            .collect::<Vec<_>>();
        Ok(PendingChangeSummary {
            count: pending.len(),
            oldest_changed_at_epoch_seconds: pending
                .iter()
                .map(|event| event.created_at_epoch_seconds)
                .min(),
        })
    }

    async fn begin_publish_boundary(&self) -> Result<PublishBoundary, String> {
        let events = self.events.lock().expect("catalog event lock");
        Ok(PublishBoundary {
            started_at_epoch_seconds: now_epoch_seconds(),
            included_event_ids: events.iter().map(|event| event.id).collect(),
        })
    }

    async fn record_successful_publish(
        &self,
        _mode: &str,
        _release_id: Option<&str>,
        publish_boundary: PublishBoundary,
        _started_at_epoch_seconds: Option<i64>,
        _finished_at_epoch_seconds: i64,
    ) -> Result<(), String> {
        *self
            .last_successful_publish_boundary
            .lock()
            .expect("publish boundary lock") = Some(publish_boundary);
        Ok(())
    }

    async fn record_event(&self, event: AutographEditEvent) -> Result<AutographEditEvent, String> {
        self.events
            .lock()
            .expect("catalog event lock")
            .push(event.clone());
        Ok(event)
    }

    async fn signer_suggestions(&self, query: String) -> Result<Vec<SignerSuggestion>, String> {
        let normalized_query = normalize_signer_name(&query);
        if normalized_query.is_empty() {
            return Ok(Vec::new());
        }
        let mut suggestions = self
            .signers
            .lock()
            .expect("catalog signer lock")
            .values()
            .filter_map(|profile| {
                signer_match_rank(&normalized_query, &profile.normalized_name).map(|rank| {
                    (
                        rank,
                        profile.display_name.clone(),
                        SignerSuggestion {
                            profile: profile.clone(),
                            possible_duplicate: rank == 0 || rank >= 2,
                        },
                    )
                })
            })
            .collect::<Vec<_>>();
        suggestions.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        Ok(suggestions
            .into_iter()
            .map(|(_, _, suggestion)| suggestion)
            .take(10)
            .collect())
    }

    async fn taxonomy_suggestions(&self) -> Result<TaxonomySuggestions, String> {
        let items = self.items.lock().expect("catalog state lock");
        let signers = self.signers.lock().expect("catalog signer lock");
        let mut characters = BTreeSet::new();
        let mut formats = BTreeSet::new();
        let mut origins = BTreeSet::new();
        let mut franchises = BTreeSet::new();
        let mut product_lines = BTreeSet::new();
        let mut set_names = BTreeSet::new();
        let mut languages = BTreeSet::new();
        let mut roles = BTreeSet::new();
        let mut tags = BTreeSet::new();
        for item in items.values() {
            characters.extend(item.characters.iter().cloned());
            formats.insert(item.format.clone());
            origins.insert(item.origin);
            franchises.extend(item.franchises.iter().cloned());
            if let Some(product_line) = item.product_line.clone() {
                product_lines.insert(product_line);
            }
            if let Some(set_name) = item.set_name.clone() {
                set_names.insert(set_name);
            }
            languages.insert(item.language.clone());
            tags.extend(item.tags.iter().cloned());
            for credit in &item.signer_credits {
                if let Some(role) = credit.item_role.clone() {
                    roles.insert(role);
                }
            }
        }
        for profile in signers.values() {
            if let Some(role) = profile.default_role.clone() {
                roles.insert(role);
            }
        }
        let mut signer_profiles = signers.values().cloned().collect::<Vec<_>>();
        signer_profiles.sort_by(|left, right| {
            left.display_name
                .cmp(&right.display_name)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(TaxonomySuggestions {
            signers: signer_profiles,
            characters: characters.into_iter().collect(),
            formats: formats.into_iter().collect(),
            origins: origins.into_iter().collect(),
            franchises: franchises.into_iter().collect(),
            product_lines: product_lines.into_iter().collect(),
            set_names: set_names.into_iter().collect(),
            languages: languages.into_iter().collect(),
            roles: roles.into_iter().collect(),
            tags: tags.into_iter().collect(),
        })
    }

    async fn update_signer_profile(
        &self,
        signer_id: Uuid,
        input: SignerProfileUpdateInput,
    ) -> Result<SignerProfile, String> {
        let now = now_epoch_seconds();
        let (before, updated, field_diffs) = {
            let mut signers = self.signers.lock().expect("catalog signer lock");
            let before = signers
                .get(&signer_id)
                .cloned()
                .ok_or_else(|| "signer profile was not found".to_owned())?;
            let mut updated = before.clone();
            apply_signer_profile_update(&mut updated, input, now)?;
            if signers.values().any(|profile| {
                profile.id != signer_id && profile.normalized_name == updated.normalized_name
            }) {
                return Err("signer normalized name already exists".to_owned());
            }
            let field_diffs = signer_profile_field_diffs(&before, &updated);
            signers.insert(signer_id, updated.clone());
            (before, updated, field_diffs)
        };
        if field_diffs.is_empty() {
            return Ok(updated);
        }
        let linked_item_ids = {
            let mut items = self.items.lock().expect("catalog state lock");
            let mut linked_item_ids = Vec::new();
            for item in items.values_mut() {
                let mut changed = false;
                for credit in &mut item.signer_credits {
                    if credit.signer.id == signer_id {
                        credit.signer = updated.clone();
                        changed = true;
                    }
                }
                if changed {
                    item.signer = compact_signer_text(&item.signer_credits);
                    item.updated_at_epoch_seconds = now;
                    linked_item_ids.push(item.id);
                }
            }
            linked_item_ids
        };
        let summary = format!(
            "Updated signer profile {} -> {}",
            before.display_name, updated.display_name
        );
        let mut events = self.events.lock().expect("catalog event lock");
        for item_id in linked_item_ids {
            events.push(AutographEditEvent::new(
                item_id,
                EditEventKind::MetadataUpdated,
                summary.clone(),
                field_diffs.clone(),
                now,
            ));
        }
        Ok(updated)
    }

    async fn merge_signer_profiles(
        &self,
        source_signer_id: Uuid,
        target_signer_id: Uuid,
    ) -> Result<SignerMergeResult, String> {
        if source_signer_id == target_signer_id {
            return Err("source and target signer profiles must differ".to_owned());
        }
        let (source, target) = {
            let mut signers = self.signers.lock().expect("catalog signer lock");
            let source = signers
                .get(&source_signer_id)
                .cloned()
                .ok_or_else(|| "source signer profile was not found".to_owned())?;
            let target = signers
                .get(&target_signer_id)
                .cloned()
                .ok_or_else(|| "target signer profile was not found".to_owned())?;
            signers.remove(&source_signer_id);
            (source, target)
        };
        let now = now_epoch_seconds();
        let linked_item_ids = {
            let mut items = self.items.lock().expect("catalog state lock");
            let mut linked_item_ids = Vec::new();
            for item in items.values_mut() {
                if !item
                    .signer_credits
                    .iter()
                    .any(|credit| credit.signer.id == source_signer_id)
                {
                    continue;
                }
                let target_already_present = item
                    .signer_credits
                    .iter()
                    .any(|credit| credit.signer.id == target_signer_id);
                if target_already_present {
                    item.signer_credits
                        .retain(|credit| credit.signer.id != source_signer_id);
                } else {
                    for credit in &mut item.signer_credits {
                        if credit.signer.id == source_signer_id {
                            credit.signer = target.clone();
                        }
                    }
                }
                for (index, credit) in item.signer_credits.iter_mut().enumerate() {
                    credit.sort_order = index as i32;
                }
                item.signer = compact_signer_text(&item.signer_credits);
                item.updated_at_epoch_seconds = now;
                linked_item_ids.push(item.id);
            }
            linked_item_ids
        };
        let summary = format!(
            "Merged signer {} into {}",
            source.display_name, target.display_name
        );
        let field_diffs = vec![FieldDiff {
            field: "signers".to_owned(),
            before: serde_json::to_value(&source).unwrap_or(Value::Null),
            after: serde_json::to_value(&target).unwrap_or(Value::Null),
        }];
        let mut events = self.events.lock().expect("catalog event lock");
        for item_id in &linked_item_ids {
            events.push(AutographEditEvent::new(
                *item_id,
                EditEventKind::MetadataUpdated,
                summary.clone(),
                field_diffs.clone(),
                now,
            ));
        }
        Ok(SignerMergeResult {
            source_signer_id,
            target_signer_id,
            updated_item_count: linked_item_ids.len(),
        })
    }
}

fn is_event_pending(event: &AutographEditEvent, boundary: Option<&PublishBoundary>) -> bool {
    boundary
        .map(|boundary| {
            !boundary.included_event_ids.contains(&event.id)
                && event.created_at_epoch_seconds >= boundary.started_at_epoch_seconds
        })
        .unwrap_or(true)
}

pub(crate) fn apply_update(
    item: &mut AutographItem,
    input: AutographItemUpdate,
    resolved_signer_credits: Option<Vec<SignerCredit>>,
) -> Vec<FieldDiff> {
    let mut field_diffs = Vec::new();
    apply_required_update("title", &mut item.title, input.title, &mut field_diffs);
    apply_required_update("signer", &mut item.signer, input.signer, &mut field_diffs);
    apply_optional_update(
        "description",
        &mut item.description,
        input.description,
        &mut field_diffs,
    );
    apply_required_update(
        "category",
        &mut item.category,
        input.category,
        &mut field_diffs,
    );
    apply_required_update(
        "tags",
        &mut item.tags,
        input.tags.map(normalize_unique_string_list),
        &mut field_diffs,
    );
    apply_required_update(
        "signers",
        &mut item.signer_credits,
        resolved_signer_credits,
        &mut field_diffs,
    );
    apply_required_update(
        "characters",
        &mut item.characters,
        input.characters.map(normalize_unique_string_list),
        &mut field_diffs,
    );
    apply_required_update("format", &mut item.format, input.format, &mut field_diffs);
    apply_required_update("origin", &mut item.origin, input.origin, &mut field_diffs);
    apply_required_update(
        "franchises",
        &mut item.franchises,
        input.franchises.map(normalize_unique_string_list),
        &mut field_diffs,
    );
    apply_optional_update(
        "productLine",
        &mut item.product_line,
        input.product_line.map(normalize_string),
        &mut field_diffs,
    );
    apply_optional_update(
        "setName",
        &mut item.set_name,
        input.set_name.map(normalize_string),
        &mut field_diffs,
    );
    apply_required_update(
        "language",
        &mut item.language,
        input.language,
        &mut field_diffs,
    );
    apply_optional_update(
        "objectReference",
        &mut item.object_reference,
        input.object_reference,
        &mut field_diffs,
    );
    apply_optional_update(
        "eventName",
        &mut item.event_name,
        input.event_name,
        &mut field_diffs,
    );
    apply_optional_update(
        "eventLocation",
        &mut item.event_location,
        input.event_location,
        &mut field_diffs,
    );
    apply_optional_update("source", &mut item.source, input.source, &mut field_diffs);
    apply_optional_update(
        "inscription",
        &mut item.inscription,
        input.inscription,
        &mut field_diffs,
    );
    apply_optional_update(
        "certificationCompany",
        &mut item.certification_company,
        input.certification_company,
        &mut field_diffs,
    );
    apply_optional_update(
        "certificationId",
        &mut item.certification_id,
        input.certification_id,
        &mut field_diffs,
    );
    apply_optional_update(
        "estimatedYear",
        &mut item.estimated_year,
        input.estimated_year,
        &mut field_diffs,
    );
    apply_required_update(
        "publicationStatus",
        &mut item.publication_status,
        input.publication_status,
        &mut field_diffs,
    );
    field_diffs
}

pub(crate) fn validate_required_fields(
    title: &str,
    signer: &str,
    category: &str,
) -> Result<(), String> {
    if title.trim().is_empty() || signer.trim().is_empty() || category.trim().is_empty() {
        return Err(REQUIRED_FIELDS_ERROR.to_owned());
    }
    Ok(())
}

fn validate_item_taxonomy(
    title: &str,
    signer: &str,
    category: &str,
    signer_credits: &[SignerCredit],
    format: &str,
    language: &str,
) -> Result<(), String> {
    validate_required_fields(title, signer, category)?;
    if signer_credits.is_empty() {
        return Err("at least one signer credit is required".to_owned());
    }
    if format.trim().is_empty() {
        return Err("format is required".to_owned());
    }
    if !matches!(language, "English" | "Japanese" | "Chinese") {
        return Err("language must be English, Japanese, or Chinese".to_owned());
    }
    Ok(())
}

fn resolve_signer_credits(
    signers: &mut HashMap<Uuid, SignerProfile>,
    inputs: &[SignerCreditInput],
    fallback_signer: &str,
    now: i64,
) -> Result<Vec<SignerCredit>, String> {
    let inputs = if inputs.is_empty() {
        vec![SignerCreditInput {
            display_name: Some(fallback_signer.to_owned()),
            ..Default::default()
        }]
    } else {
        inputs.to_vec()
    };

    let mut seen = BTreeSet::new();
    let mut credits = Vec::with_capacity(inputs.len());
    for (index, input) in inputs.iter().enumerate() {
        validate_profile_url(input.wikipedia_url.as_deref(), "wikipediaUrl")?;
        validate_profile_url(input.imdb_url.as_deref(), "imdbUrl")?;
        let profile = resolve_signer_profile(signers, input, now)?;
        if !seen.insert(profile.normalized_name.clone()) {
            return Err("duplicate signer credits are not allowed".to_owned());
        }
        credits.push(SignerCredit {
            signer: profile,
            sort_order: index as i32,
            item_role: normalize_optional_string(input.item_role.clone()),
            item_context: normalize_optional_string(input.item_context.clone()),
        });
    }

    Ok(credits)
}

fn resolve_signer_profile(
    signers: &mut HashMap<Uuid, SignerProfile>,
    input: &SignerCreditInput,
    now: i64,
) -> Result<SignerProfile, String> {
    if let Some(signer_id) = input.signer_id
        && let Some(profile) = signers.get_mut(&signer_id)
    {
        update_signer_profile(profile, input, now);
        return Ok(profile.clone());
    }

    let display_name = input
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "signer displayName is required".to_owned())?;
    let normalized_name = normalize_signer_name(display_name);
    if normalized_name.is_empty() {
        return Err("signer displayName is required".to_owned());
    }

    if let Some(existing_id) = signers
        .values()
        .find(|profile| profile.normalized_name == normalized_name)
        .map(|profile| profile.id)
        && let Some(profile) = signers.get_mut(&existing_id)
    {
        update_signer_profile(profile, input, now);
        return Ok(profile.clone());
    }

    let profile = SignerProfile {
        id: input.signer_id.unwrap_or_else(Uuid::new_v4),
        display_name: display_name.to_owned(),
        normalized_name,
        default_role: normalize_optional_string(input.default_role.clone()),
        wikipedia_url: normalize_optional_string(input.wikipedia_url.clone()),
        imdb_url: normalize_optional_string(input.imdb_url.clone()),
        created_at_epoch_seconds: now,
        updated_at_epoch_seconds: now,
    };
    signers.insert(profile.id, profile.clone());
    Ok(profile)
}

fn update_signer_profile(profile: &mut SignerProfile, input: &SignerCreditInput, now: i64) {
    let mut changed = false;
    if let Some(display_name) = normalize_optional_string(input.display_name.clone()) {
        let normalized_name = normalize_signer_name(&display_name);
        if profile.display_name != display_name || profile.normalized_name != normalized_name {
            profile.display_name = display_name;
            profile.normalized_name = normalized_name;
            changed = true;
        }
    }
    for (current, incoming) in [
        (&mut profile.default_role, input.default_role.clone()),
        (&mut profile.wikipedia_url, input.wikipedia_url.clone()),
        (&mut profile.imdb_url, input.imdb_url.clone()),
    ] {
        let normalized = normalize_optional_string(incoming);
        if *current != normalized {
            *current = normalized;
            changed = true;
        }
    }
    if changed {
        profile.updated_at_epoch_seconds = now;
    }
}

fn validate_profile_url(value: Option<&str>, field: &str) -> Result<(), String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    if value.len() > MAX_PROFILE_URL_LENGTH {
        return Err(format!(
            "{field} must be {MAX_PROFILE_URL_LENGTH} characters or fewer"
        ));
    }
    let Some(rest) = value.strip_prefix("https://") else {
        return Err(format!("{field} must be an https URL"));
    };
    let host = rest
        .split(['/', '?', '#', ':'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let allowed = match field {
        "wikipediaUrl" => host == "wikipedia.org" || host.ends_with(".wikipedia.org"),
        "imdbUrl" => host == "imdb.com" || host.ends_with(".imdb.com"),
        _ => false,
    };
    if !allowed {
        let expected_host = match field {
            "wikipediaUrl" => "wikipedia.org",
            "imdbUrl" => "imdb.com",
            _ => "the expected profile host",
        };
        return Err(format!("{field} must point to {expected_host}"));
    }
    Ok(())
}

pub(crate) fn apply_signer_profile_update(
    profile: &mut SignerProfile,
    input: SignerProfileUpdateInput,
    now: i64,
) -> Result<(), String> {
    validate_profile_url(input.wikipedia_url.as_deref(), "wikipediaUrl")?;
    validate_profile_url(input.imdb_url.as_deref(), "imdbUrl")?;
    let mut changed = false;
    if let Some(display_name) = normalize_optional_string(input.display_name) {
        let normalized_name = normalize_signer_name(&display_name);
        if normalized_name.is_empty() {
            return Err("signer displayName is required".to_owned());
        }
        if profile.display_name != display_name || profile.normalized_name != normalized_name {
            profile.display_name = display_name;
            profile.normalized_name = normalized_name;
            changed = true;
        }
    }
    for (current, incoming) in [
        (&mut profile.default_role, input.default_role),
        (&mut profile.wikipedia_url, input.wikipedia_url),
        (&mut profile.imdb_url, input.imdb_url),
    ] {
        let normalized = normalize_optional_string(incoming);
        if *current != normalized {
            *current = normalized;
            changed = true;
        }
    }
    if changed {
        profile.updated_at_epoch_seconds = now;
    }
    Ok(())
}

pub(crate) fn signer_profile_field_diffs(
    before: &SignerProfile,
    after: &SignerProfile,
) -> Vec<FieldDiff> {
    let mut field_diffs = Vec::new();
    push_diff_if_changed(
        "signerProfile.displayName",
        &before.display_name,
        &after.display_name,
        &mut field_diffs,
    );
    push_diff_if_changed(
        "signerProfile.defaultRole",
        &before.default_role,
        &after.default_role,
        &mut field_diffs,
    );
    push_diff_if_changed(
        "signerProfile.wikipediaUrl",
        &before.wikipedia_url,
        &after.wikipedia_url,
        &mut field_diffs,
    );
    push_diff_if_changed(
        "signerProfile.imdbUrl",
        &before.imdb_url,
        &after.imdb_url,
        &mut field_diffs,
    );
    field_diffs
}

pub(crate) fn signer_match_rank(query: &str, candidate: &str) -> Option<u8> {
    if candidate == query {
        return Some(0);
    }
    if candidate.starts_with(query) || query.starts_with(candidate) {
        return Some(1);
    }
    if candidate.contains(query) || query.contains(candidate) {
        return Some(2);
    }
    if query
        .split_whitespace()
        .any(|token| token.len() >= 3 && candidate.contains(token))
        || candidate
            .split_whitespace()
            .any(|token| token.len() >= 3 && query.contains(token))
    {
        return Some(2);
    }
    if levenshtein_distance(query, candidate) <= 2 {
        return Some(2);
    }
    None
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let mut costs = (0..=right.chars().count()).collect::<Vec<_>>();
    for (left_index, left_char) in left.chars().enumerate() {
        let mut previous = costs[0];
        costs[0] = left_index + 1;
        for (right_index, right_char) in right.chars().enumerate() {
            let insertion = costs[right_index + 1] + 1;
            let deletion = costs[right_index] + 1;
            let replacement = previous + usize::from(left_char != right_char);
            previous = costs[right_index + 1];
            costs[right_index + 1] = insertion.min(deletion).min(replacement);
        }
    }
    *costs.last().unwrap_or(&0)
}

fn compact_signer_text(credits: &[SignerCredit]) -> String {
    credits
        .iter()
        .map(|credit| credit.signer.display_name.as_str())
        .collect::<Vec<_>>()
        .join(" + ")
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = normalize_string(value);
        if value.is_empty() { None } else { Some(value) }
    })
}

fn normalize_string(value: String) -> String {
    value.trim().to_owned()
}

fn normalize_string_list(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .filter_map(|value| {
            let value = normalize_string(value);
            if value.is_empty() { None } else { Some(value) }
        })
        .collect()
}

fn normalize_unique_string_list(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    normalize_string_list(values)
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

pub fn normalize_signer_name(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character.is_whitespace() {
                character.to_lowercase().collect::<String>()
            } else {
                " ".to_owned()
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn event_kind_for_diffs(field_diffs: &[FieldDiff]) -> EditEventKind {
    if field_diffs
        .iter()
        .any(|diff| diff.field == "publicationStatus")
    {
        EditEventKind::PublicationChanged
    } else {
        EditEventKind::MetadataUpdated
    }
}

pub(crate) fn event_summary(kind: EditEventKind, field_diffs: &[FieldDiff]) -> String {
    match kind {
        EditEventKind::Created => "Created autograph item".to_owned(),
        EditEventKind::PublicationChanged => "Updated publication status".to_owned(),
        EditEventKind::MetadataUpdated => format!(
            "Updated metadata field{}: {}",
            if field_diffs.len() == 1 { "" } else { "s" },
            field_diffs
                .iter()
                .map(|diff| diff.field.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        EditEventKind::ImageAdded => "Image added".to_owned(),
        EditEventKind::ImageRemoved => "Image removed".to_owned(),
        EditEventKind::ImageReplaced => "Image replaced".to_owned(),
        EditEventKind::PrimaryImageChanged => "Primary image changed".to_owned(),
        EditEventKind::CleanupChanged => "Cleanup status changed".to_owned(),
    }
}

pub(crate) fn now_epoch_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

fn apply_required_update<T>(
    field: &str,
    current: &mut T,
    update: Option<T>,
    field_diffs: &mut Vec<FieldDiff>,
) where
    T: PartialEq + Serialize,
{
    if let Some(after) = update {
        push_diff_if_changed(field, current, &after, field_diffs);
        *current = after;
    }
}

fn apply_optional_update<T>(
    field: &str,
    current: &mut Option<T>,
    update: FieldPatch<T>,
    field_diffs: &mut Vec<FieldDiff>,
) where
    T: PartialEq + Serialize,
{
    match update {
        FieldPatch::Unchanged => {}
        FieldPatch::Clear => {
            let after = None;
            push_diff_if_changed(field, current, &after, field_diffs);
            *current = after;
        }
        FieldPatch::Set(value) => {
            let after = Some(value);
            push_diff_if_changed(field, current, &after, field_diffs);
            *current = after;
        }
    }
}

fn push_diff_if_changed<T>(field: &str, before: &T, after: &T, field_diffs: &mut Vec<FieldDiff>)
where
    T: PartialEq + Serialize,
{
    if before != after {
        field_diffs.push(FieldDiff {
            field: field.to_owned(),
            before: serde_json::to_value(before).unwrap_or(Value::Null),
            after: serde_json::to_value(after).unwrap_or(Value::Null),
        });
    }
}

const fn draft() -> PublicationStatus {
    PublicationStatus::Draft
}

fn default_format() -> String {
    DEFAULT_FORMAT.to_owned()
}

fn default_language() -> String {
    DEFAULT_LANGUAGE.to_owned()
}
