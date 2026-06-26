use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub(crate) const REQUIRED_FIELDS_ERROR: &str = "title, signer, and category are required";

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

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AutographItemUpdate {
    pub title: Option<String>,
    pub signer: Option<String>,
    #[serde(default)]
    pub description: FieldPatch<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
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

    async fn set_primary_image(&self, item_id: Uuid, image_id: Uuid) -> Result<AutographItem, String>;

    async fn history(&self, _item_id: Uuid) -> Result<Vec<AutographEditEvent>, String> {
        Ok(Vec::new())
    }

    async fn pending_changes(&self) -> Result<PendingChangeSummary, String> {
        Ok(PendingChangeSummary::default())
    }

    async fn record_event(&self, event: AutographEditEvent) -> Result<AutographEditEvent, String> {
        Ok(event)
    }
}

#[derive(Clone)]
pub struct MemoryCatalogRepository {
    items: Arc<Mutex<HashMap<Uuid, AutographItem>>>,
    events: Arc<Mutex<Vec<AutographEditEvent>>>,
}

impl Default for MemoryCatalogRepository {
    fn default() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl CatalogRepository for MemoryCatalogRepository {
    async fn create(&self, input: AutographItemInput) -> Result<AutographItem, String> {
        validate_required_fields(&input.title, &input.signer, &input.category)?;
        let now = now_epoch_seconds();
        let item = AutographItem {
            id: Uuid::new_v4(),
            title: input.title,
            signer: input.signer,
            description: input.description,
            category: input.category,
            tags: input.tags,
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
        let updated = {
            let mut items = self.items.lock().expect("catalog state lock");
            let item = items
                .get_mut(&id)
                .ok_or_else(|| "autograph item was not found".to_owned())?;
            let mut candidate = item.clone();
            let field_diffs = apply_update(&mut candidate, input);
            validate_required_fields(&candidate.title, &candidate.signer, &candidate.category)?;
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

    async fn set_primary_image(&self, item_id: Uuid, image_id: Uuid) -> Result<AutographItem, String> {
        let now = now_epoch_seconds();
        let updated = {
            let mut items = self.items.lock().expect("catalog state lock");
            let item = items.get_mut(&item_id).ok_or_else(|| "autograph item was not found".to_owned())?;
            let image = item.images.iter_mut().find(|image| image.id == image_id)
                .ok_or_else(|| "autograph image was not found".to_owned())?;
            if !image.is_primary {
                for image in &mut item.images { image.is_primary = image.id == image_id; }
                item.updated_at_epoch_seconds = now;
            }
            item.clone()
        };
        self.events.lock().expect("catalog event lock").push(AutographEditEvent::new(
            item_id, EditEventKind::PrimaryImageChanged, "Primary image changed", Vec::new(), now,
        ));
        Ok(updated)
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
        Ok(PendingChangeSummary {
            count: events.len(),
            oldest_changed_at_epoch_seconds: events
                .iter()
                .map(|event| event.created_at_epoch_seconds)
                .min(),
        })
    }

    async fn record_event(&self, event: AutographEditEvent) -> Result<AutographEditEvent, String> {
        self.events
            .lock()
            .expect("catalog event lock")
            .push(event.clone());
        Ok(event)
    }
}

pub(crate) fn apply_update(item: &mut AutographItem, input: AutographItemUpdate) -> Vec<FieldDiff> {
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
    apply_required_update("tags", &mut item.tags, input.tags, &mut field_diffs);
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
