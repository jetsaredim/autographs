use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use oracledb::{Connection, FromDbValue, Row, ToDbValue};
use serde_json::Value;
use tokio::task;
use uuid::Uuid;

use crate::catalog::{
    AutographEditEvent, AutographImage, AutographItem, AutographItemInput, AutographItemUpdate,
    CatalogRepository, CleanupStatus, CleanupWarning, EditEventKind, FieldDiff, ImageCleanupEvent,
    ImageReplacementInput, ItemOrigin, PendingChangeSummary, PublicationStatus, PublishBoundary,
    SignerCredit, SignerCreditInput, SignerMergeResult, SignerProfile, SignerProfileUpdateInput,
    SignerSuggestion, TaxonomySuggestions, apply_signer_profile_update, apply_update,
    event_kind_for_diffs, event_summary, normalize_profile_link, normalize_signer_name,
    now_epoch_seconds, signer_match_rank, signer_profile_field_diffs, validate_required_fields,
};
use crate::oracle_connection;

const ITEM_SELECT_COLUMNS: &str = "title, signer, description, category, object_reference,
    event_name, event_location, source, inscription,
    certification_company, certification_id, estimated_year,
    publication_status, format, origin, language, product_line, set_name,
    cast(round((cast(created_at as date) - date '1970-01-01') * 86400) as number(19)),
    cast(round((cast(updated_at as date) - date '1970-01-01') * 86400) as number(19))";

const LOAD_ITEM_SQL: &str = "select
    title, signer, description, category, object_reference,
    event_name, event_location, source, inscription,
    certification_company, certification_id, estimated_year,
    publication_status, format, origin, language, product_line, set_name,
    cast(round((cast(created_at as date) - date '1970-01-01') * 86400) as number(19)),
    cast(round((cast(updated_at as date) - date '1970-01-01') * 86400) as number(19))
from autograph_items where id = :1";

const IMAGE_SELECT_COLUMNS: &str = "id, object_key, original_filename, content_type, byte_size,
    checksum, etag, is_primary, sort_order, alt_text";

const SIGNER_CREDIT_ROWS_FOR_UPDATE_SQL: &str =
    "select signer_id, sort_order, item_role, item_context
    from autograph_item_signers
    where item_id = :1
    order by signer_id
    for update";

const GLOBAL_PENDING_CHANGES_SQL: &str = "with latest_publish as (
    select id, started_at, snapshot_event_count
    from (
        select id, started_at, snapshot_event_count, created_at
        from autograph_publish_jobs
        where status = 'succeeded'
        order by started_at desc, created_at desc, id desc
    )
    where rownum = 1
)
select
    count(*),
    cast(round((cast(min(e.created_at) as date) - date '1970-01-01') * 86400) as number(19))
from autograph_edit_events e
left join latest_publish p on 1 = 1
left join autograph_publish_job_events pe
    on pe.publish_job_id = p.id
   and pe.edit_event_id = e.id
where p.id is null
   or (p.snapshot_event_count is not null and pe.edit_event_id is null)
   or (p.snapshot_event_count is null and e.created_at >= p.started_at)";

const ITEM_PENDING_CHANGES_SQL: &str = "with latest_publish as (
    select id, started_at, snapshot_event_count
    from (
        select id, started_at, snapshot_event_count, created_at
        from autograph_publish_jobs
        where status = 'succeeded'
        order by started_at desc, created_at desc, id desc
    )
    where rownum = 1
)
select
    count(*),
    cast(round((cast(min(e.created_at) as date) - date '1970-01-01') * 86400) as number(19))
from autograph_edit_events e
left join latest_publish p on 1 = 1
left join autograph_publish_job_events pe
    on pe.publish_job_id = p.id
   and pe.edit_event_id = e.id
where e.item_id = :1
  and (
    p.id is null
    or (p.snapshot_event_count is not null and pe.edit_event_id is null)
    or (p.snapshot_event_count is null and e.created_at >= p.started_at)
  )";

#[derive(Clone)]
pub struct OracleCatalogRepository {
    user: String,
    password: String,
    connect_string: String,
    storage_namespace: String,
    bucket_name: String,
}

impl OracleCatalogRepository {
    pub fn new(
        user: String,
        password: String,
        connect_string: String,
        storage_namespace: String,
        bucket_name: String,
    ) -> Self {
        Self {
            user,
            password,
            connect_string,
            storage_namespace,
            bucket_name,
        }
    }

    async fn with_connection<T, F>(&self, operation: F) -> Result<T, String>
    where
        T: Send + 'static,
        F: FnOnce(Connection) -> Result<T, String> + Send + 'static,
    {
        let repository = self.clone();
        task::spawn_blocking(move || {
            let connection = oracle_connection::connect(
                &repository.user,
                &repository.password,
                &repository.connect_string,
            )
            .map_err(|error| format!("connect to Oracle catalog: {error}"))?;
            operation(connection)
        })
        .await
        .map_err(|error| format!("join Oracle catalog task: {error}"))?
    }
}

#[async_trait]
impl CatalogRepository for OracleCatalogRepository {
    async fn create(&self, input: AutographItemInput) -> Result<AutographItem, String> {
        validate_required_fields(&input.title, &input.signer, &input.category)?;
        let id = Uuid::new_v4();
        self.with_connection(move |connection| {
            let signer_credits =
                resolve_oracle_signer_credits(&connection, &input.signer_credits, &input.signer)?;
            let legacy_signer = compact_signer_text(&signer_credits);
            let legacy_category = input.format.clone();
            let tags = normalize_unique_string_list(input.tags);
            let characters = normalize_unique_string_list(input.characters);
            let franchises = normalize_unique_string_list(input.franchises);
            let product_line = normalize_optional_string(input.product_line);
            let set_name = normalize_optional_string(input.set_name);
            if input.format.trim().is_empty() {
                return Err("format is required".to_owned());
            }
            if !matches!(input.language.as_str(), "English" | "Japanese" | "Chinese") {
                return Err("language must be English, Japanese, or Chinese".to_owned());
            }
            let id_text = id.to_string();
            let status = publication_status_text(input.publication_status);
            let origin = item_origin_text(input.origin);
            connection
                .execute(
                    "insert into autograph_items (
                        id, title, signer, description, category, object_reference,
                        event_name, event_location, source, inscription,
                        certification_company, certification_id, estimated_year,
                        format, origin, language, product_line, set_name,
                        publication_status
                    ) values (
                        :1, :2, :3, :4, :5, :6, :7, :8, :9, :10, :11, :12, :13,
                        :14, :15, :16, :17, :18, :19
                    )",
                    &[
                        &id_text,
                        &input.title,
                        &legacy_signer,
                        &input.description,
                        &legacy_category,
                        &input.object_reference,
                        &input.event_name,
                        &input.event_location,
                        &input.source,
                        &input.inscription,
                        &input.certification_company,
                        &input.certification_id,
                        &input.estimated_year,
                        &input.format,
                        &origin,
                        &input.language,
                        &product_line,
                        &set_name,
                        &status,
                    ],
                )
                .map_err(|error| format!("insert Oracle catalog item: {error}"))?;
            replace_tags(&connection, id, &tags)?;
            replace_signer_credits(&connection, id, &signer_credits)?;
            replace_characters(&connection, id, &characters)?;
            replace_franchises(&connection, id, &franchises)?;
            let event = AutographEditEvent::new(
                id,
                EditEventKind::Created,
                format!("Created autograph item `{}`", input.title),
                Vec::new(),
                now_epoch_seconds(),
            );
            insert_edit_event(&connection, &event)?;
            connection
                .commit()
                .map_err(|error| format!("commit Oracle catalog item: {error}"))?;
            load_item(&connection, id)?
                .ok_or_else(|| "created Oracle item was not found".to_owned())
        })
        .await
    }

    async fn update(&self, id: Uuid, input: AutographItemUpdate) -> Result<AutographItem, String> {
        self.with_connection(move |connection| {
            let mut item = load_item(&connection, id)?
                .ok_or_else(|| "autograph item was not found".to_owned())?;
            let input_updates_tags = input.tags.is_some();
            let input_updates_signer_credits = input.signer_credits.is_some();
            let input_updates_characters = input.characters.is_some();
            let input_updates_franchises = input.franchises.is_some();
            let has_persisted_signer_credits = has_persisted_signer_credits(&connection, id)?;
            let resolved_signer_credits = if let Some(signer_inputs) = input.signer_credits.as_ref()
            {
                Some(resolve_oracle_signer_credits(
                    &connection,
                    signer_inputs,
                    &item.signer,
                )?)
            } else {
                None
            };
            let field_diffs = apply_update(&mut item, input, resolved_signer_credits);
            item.signer = compact_signer_text(&item.signer_credits);
            item.category = item.format.clone();
            validate_required_fields(&item.title, &item.signer, &item.category)?;
            if item.format.trim().is_empty() {
                return Err("format is required".to_owned());
            }
            if !matches!(item.language.as_str(), "English" | "Japanese" | "Chinese") {
                return Err("language must be English, Japanese, or Chinese".to_owned());
            }
            if field_diffs.is_empty() {
                return Ok(item);
            }
            let id_text = id.to_string();
            let status = publication_status_text(item.publication_status);
            let origin = item_origin_text(item.origin);
            let statement = connection
                .execute(
                    "update autograph_items set
                        title = :1, signer = :2, description = :3, category = :4,
                        object_reference = :5, event_name = :6, event_location = :7,
                        source = :8, inscription = :9, certification_company = :10,
                        certification_id = :11, estimated_year = :12,
                        format = :13, origin = :14, language = :15,
                        product_line = :16, set_name = :17,
                        publication_status = :18, updated_at = current_timestamp
                    where id = :19",
                    &[
                        &item.title,
                        &item.signer,
                        &item.description,
                        &item.category,
                        &item.object_reference,
                        &item.event_name,
                        &item.event_location,
                        &item.source,
                        &item.inscription,
                        &item.certification_company,
                        &item.certification_id,
                        &item.estimated_year,
                        &item.format,
                        &origin,
                        &item.language,
                        &item.product_line,
                        &item.set_name,
                        &status,
                        &id_text,
                    ],
                )
                .map_err(|error| format!("update Oracle catalog item: {error}"))?;
            let rows_updated = statement.rows_affected();
            if rows_updated == 0 {
                return Err("autograph item was not found".to_owned());
            }
            if input_updates_tags {
                replace_tags(&connection, id, &item.tags)?;
            }
            let signer_credits_changed = field_diffs.iter().any(|diff| diff.field == "signers");
            if should_sync_signer_credits(
                input_updates_signer_credits,
                has_persisted_signer_credits,
                signer_credits_changed,
            ) {
                sync_signer_credits(&connection, id, &item.signer_credits)?;
            }
            if input_updates_characters {
                replace_characters(&connection, id, &item.characters)?;
            }
            if input_updates_franchises {
                replace_franchises(&connection, id, &item.franchises)?;
            }
            let kind = event_kind_for_diffs(&field_diffs);
            let event = AutographEditEvent::new(
                id,
                kind,
                event_summary(kind, &field_diffs),
                field_diffs,
                now_epoch_seconds(),
            );
            insert_edit_event(&connection, &event)?;
            connection
                .commit()
                .map_err(|error| format!("commit Oracle catalog update: {error}"))?;
            load_item(&connection, id)?
                .ok_or_else(|| "updated Oracle item was not found".to_owned())
        })
        .await
    }

    async fn get(&self, id: Uuid) -> Result<Option<AutographItem>, String> {
        self.with_connection(move |connection| load_item(&connection, id))
            .await
    }

    async fn list(&self) -> Result<Vec<AutographItem>, String> {
        self.with_connection(move |connection| load_items(&connection, None))
            .await
    }

    async fn list_published(&self) -> Result<Vec<AutographItem>, String> {
        self.with_connection(move |connection| {
            load_items(&connection, Some(PublicationStatus::Published))
        })
        .await
    }

    async fn attach_image(
        &self,
        item_id: Uuid,
        image: AutographImage,
    ) -> Result<AutographItem, String> {
        let storage_namespace = self.storage_namespace.clone();
        let bucket_name = self.bucket_name.clone();
        self.with_connection(move |connection| {
            let existing_item = load_item(&connection, item_id)?
                .ok_or_else(|| "autograph item was not found".to_owned())?;
            if image.is_primary && existing_item.images.iter().any(|image| image.is_primary) {
                let item_id_text = item_id.to_string();
                connection
                    .execute(
                        "update autograph_images set is_primary = 'N', updated_at = current_timestamp where item_id = :1",
                        &[&item_id_text],
                    )
                    .map_err(|error| format!("clear Oracle primary image: {error}"))?;
            }
            if existing_item.id != item_id {
                return Err("autograph item was not found".to_owned());
            }
            let item_id_text = item_id.to_string();
            let image_id = image.id.to_string();
            let byte_size = image.byte_size as i64;
            let is_primary = if image.is_primary { "Y" } else { "N" };
            connection
                .execute(
                    "insert into autograph_images (
                        id, item_id, storage_namespace, bucket_name, object_key,
                        original_filename, content_type, byte_size, checksum, etag,
                        is_primary, sort_order, alt_text
                    ) values (:1, :2, :3, :4, :5, :6, :7, :8, :9, :10, :11, :12, :13)",
                    &[
                        &image_id,
                        &item_id_text,
                        &storage_namespace,
                        &bucket_name,
                        &image.object_key,
                        &image.original_filename,
                        &image.content_type,
                        &byte_size,
                        &image.checksum,
                        &image.etag,
                        &is_primary,
                        &image.sort_order,
                        &image.alt_text,
                    ],
                )
                .map_err(|error| format!("insert Oracle catalog image: {error}"))?;
            connection
                .execute(
                    "update autograph_items set updated_at = current_timestamp where id = :1",
                    &[&item_id_text],
                )
                .map_err(|error| format!("touch Oracle catalog item for image upload: {error}"))?;
            let event = AutographEditEvent::new(
                item_id,
                EditEventKind::ImageAdded,
                "Image added",
                Vec::new(),
                now_epoch_seconds(),
            );
            insert_edit_event(&connection, &event)?;
            connection
                .commit()
                .map_err(|error| format!("commit Oracle catalog image: {error}"))?;
            load_item(&connection, item_id)?
                .ok_or_else(|| "updated Oracle item was not found".to_owned())
        })
        .await
    }

    async fn set_primary_image(
        &self,
        item_id: Uuid,
        image_id: Uuid,
    ) -> Result<AutographItem, String> {
        self.with_connection(move |connection| {
            let item_id_text = item_id.to_string();
            let image_id_text = image_id.to_string();
            let exists: i64 = query_row_value(
                &connection,
                "select count(*) from autograph_images where id = :1 and item_id = :2",
                &[&image_id_text, &item_id_text],
                "primary image existence",
            )?;
            if exists != 1 { return Err("autograph image was not found".to_owned()); }
            connection.execute(
                "update autograph_images set is_primary = case when id = :1 then 'Y' else 'N' end, updated_at = current_timestamp where item_id = :2",
                &[&image_id_text, &item_id_text],
            ).map_err(|error| format!("set Oracle primary image: {error}"))?;
            connection.execute("update autograph_items set updated_at = current_timestamp where id = :1", &[&item_id_text])
                .map_err(|error| format!("touch Oracle catalog item for primary image: {error}"))?;
            let event = AutographEditEvent::new(item_id, EditEventKind::PrimaryImageChanged, "Primary image changed", Vec::new(), now_epoch_seconds());
            insert_edit_event(&connection, &event)?;
            connection.commit().map_err(|error| format!("commit Oracle primary image: {error}"))?;
            load_item(&connection, item_id)?.ok_or_else(|| "autograph item was not found".to_owned())
        }).await
    }

    async fn remove_image_metadata(
        &self,
        item_id: Uuid,
        image_id: Uuid,
    ) -> Result<AutographItem, String> {
        self.with_connection(move |connection| {
            let item_id_text = item_id.to_string();
            let image_id_text = image_id.to_string();
            let image = load_image(&connection, item_id, image_id)?
                .ok_or_else(|| "autograph image was not found".to_owned())?;
            let statement = connection
                .execute(
                    "delete from autograph_images where id = :1 and item_id = :2",
                    &[&image_id_text, &item_id_text],
                )
                .map_err(|error| format!("delete Oracle catalog image metadata: {error}"))?;
            let rows_deleted = statement.rows_affected();
            if rows_deleted == 0 {
                return Err("autograph image was not found".to_owned());
            }
            if image.is_primary {
                promote_first_remaining_image(&connection, item_id)?;
            }
            connection
                .execute(
                    "update autograph_items set updated_at = current_timestamp where id = :1",
                    &[&item_id_text],
                )
                .map_err(|error| format!("touch Oracle catalog item for image removal: {error}"))?;
            let event = AutographEditEvent::new(
                item_id,
                EditEventKind::ImageRemoved,
                "Image removed",
                Vec::new(),
                now_epoch_seconds(),
            );
            insert_edit_event(&connection, &event)?;
            connection
                .commit()
                .map_err(|error| format!("commit Oracle image metadata removal: {error}"))?;
            load_item(&connection, item_id)?
                .ok_or_else(|| "autograph item was not found".to_owned())
        })
        .await
    }

    async fn replace_image_metadata(
        &self,
        item_id: Uuid,
        image_id: Uuid,
        input: ImageReplacementInput,
    ) -> Result<AutographItem, String> {
        let storage_namespace = self.storage_namespace.clone();
        let bucket_name = self.bucket_name.clone();
        self.with_connection(move |connection| {
            let existing = load_image(&connection, item_id, image_id)?
                .ok_or_else(|| "autograph image was not found".to_owned())?;
            let item_id_text = item_id.to_string();
            let image_id_text = image_id.to_string();
            let byte_size = input.image.byte_size as i64;
            let is_primary = if existing.is_primary { "Y" } else { "N" };
            let statement = connection
                .execute(
                    "update autograph_images set
                        storage_namespace = :1,
                        bucket_name = :2,
                        object_key = :3,
                        original_filename = :4,
                        content_type = :5,
                        byte_size = :6,
                        checksum = :7,
                        etag = :8,
                        is_primary = :9,
                        sort_order = :10,
                        alt_text = :11,
                        updated_at = current_timestamp
                    where id = :12 and item_id = :13",
                    &[
                        &storage_namespace,
                        &bucket_name,
                        &input.image.object_key,
                        &input.image.original_filename,
                        &input.image.content_type,
                        &byte_size,
                        &input.image.checksum,
                        &input.image.etag,
                        &is_primary,
                        &existing.sort_order,
                        &input.image.alt_text,
                        &image_id_text,
                        &item_id_text,
                    ],
                )
                .map_err(|error| format!("replace Oracle catalog image metadata: {error}"))?;
            let rows_updated = statement.rows_affected();
            if rows_updated == 0 {
                return Err("autograph image was not found".to_owned());
            }
            connection
                .execute(
                    "update autograph_items set updated_at = current_timestamp where id = :1",
                    &[&item_id_text],
                )
                .map_err(|error| {
                    format!("touch Oracle catalog item for image replacement: {error}")
                })?;
            let event = AutographEditEvent::new(
                item_id,
                EditEventKind::ImageReplaced,
                "Image replaced",
                Vec::new(),
                now_epoch_seconds(),
            );
            insert_edit_event(&connection, &event)?;
            connection
                .commit()
                .map_err(|error| format!("commit Oracle image metadata replacement: {error}"))?;
            load_item(&connection, item_id)?
                .ok_or_else(|| "autograph item was not found".to_owned())
        })
        .await
    }

    async fn record_cleanup_event(
        &self,
        event: ImageCleanupEvent,
    ) -> Result<ImageCleanupEvent, String> {
        self.with_connection(move |connection| {
            insert_cleanup_event(&connection, &event)?;
            let edit_event = AutographEditEvent::new(
                event.item_id,
                EditEventKind::CleanupChanged,
                "Cleanup status changed",
                Vec::new(),
                event.created_at_epoch_seconds,
            );
            insert_edit_event(&connection, &edit_event)?;
            connection
                .commit()
                .map_err(|error| format!("commit Oracle cleanup event: {error}"))?;
            Ok(event)
        })
        .await
    }

    async fn cleanup_warnings(&self, item_id: Uuid) -> Result<Vec<CleanupWarning>, String> {
        self.with_connection(move |connection| load_cleanup_warnings(&connection, item_id))
            .await
    }

    async fn mark_cleanup_retry_succeeded(
        &self,
        item_id: Uuid,
        image_id: Uuid,
        target_object_key: &str,
    ) -> Result<bool, String> {
        let target_object_key = target_object_key.to_owned();
        self.with_connection(move |connection| {
            let item_id_text = item_id.to_string();
            let image_id_text = image_id.to_string();
            let statement = connection
                .execute(
                    "update autograph_cleanup_events set
                        status = 'retrySucceeded',
                        resolved_at = current_timestamp
                    where item_id = :1
                      and image_id = :2
                      and target_object_key = :3
                      and status = 'deleteFailed'",
                    &[&item_id_text, &image_id_text, &target_object_key],
                )
                .map_err(|error| format!("mark Oracle cleanup retry succeeded: {error}"))?;
            let rows_updated = statement.rows_affected();
            if rows_updated == 0 {
                return Ok(false);
            }
            let event = AutographEditEvent::new(
                item_id,
                EditEventKind::CleanupChanged,
                "Cleanup retry succeeded",
                Vec::new(),
                now_epoch_seconds(),
            );
            insert_edit_event(&connection, &event)?;
            connection
                .commit()
                .map_err(|error| format!("commit Oracle cleanup retry: {error}"))?;
            Ok(true)
        })
        .await
    }

    async fn history(&self, item_id: Uuid) -> Result<Vec<AutographEditEvent>, String> {
        self.with_connection(move |connection| load_history(&connection, item_id))
            .await
    }

    async fn pending_changes(&self) -> Result<PendingChangeSummary, String> {
        self.with_connection(move |connection| {
            let mut rows = connection
                .query(GLOBAL_PENDING_CHANGES_SQL, &[])
                .map_err(|error| format!("read Oracle pending changes: {error}"))?;
            let Some(row) = rows.next() else {
                return Ok(PendingChangeSummary::default());
            };
            let row = row.map_err(|error| format!("read Oracle pending changes row: {error}"))?;
            Ok(PendingChangeSummary {
                count: row_value::<Option<i64>>(&row, 0, "pending change count")?.unwrap_or(0)
                    as usize,
                oldest_changed_at_epoch_seconds: row_value(&row, 1, "oldest pending change")?,
            })
        })
        .await
    }

    async fn pending_changes_for_item(
        &self,
        item_id: Uuid,
    ) -> Result<PendingChangeSummary, String> {
        self.with_connection(move |connection| {
            let item_id = item_id.to_string();
            let mut rows = connection
                .query(ITEM_PENDING_CHANGES_SQL, &[&item_id])
                .map_err(|error| format!("read Oracle item pending changes: {error}"))?;
            let Some(row) = rows.next() else {
                return Ok(PendingChangeSummary::default());
            };
            let row =
                row.map_err(|error| format!("read Oracle item pending changes row: {error}"))?;
            Ok(PendingChangeSummary {
                count: row_value::<Option<i64>>(&row, 0, "item pending change count")?.unwrap_or(0)
                    as usize,
                oldest_changed_at_epoch_seconds: row_value(&row, 1, "oldest item pending change")?,
            })
        })
        .await
    }

    async fn begin_publish_boundary(&self) -> Result<PublishBoundary, String> {
        self.with_connection(move |connection| {
            let started_at_epoch_seconds = now_epoch_seconds();
            let mut rows = connection
                .query("select id from autograph_edit_events", &[])
                .map_err(|error| format!("snapshot Oracle publish edit events: {error}"))?;
            let mut included_event_ids = BTreeSet::new();
            for row in &mut rows {
                let row =
                    row.map_err(|error| format!("read Oracle publish edit event row: {error}"))?;
                included_event_ids.insert(parse_uuid(&row_value::<String>(
                    &row,
                    0,
                    "publish edit event id",
                )?)?);
            }
            Ok(PublishBoundary {
                started_at_epoch_seconds,
                included_event_ids,
            })
        })
        .await
    }

    async fn record_successful_publish(
        &self,
        mode: &str,
        release_id: Option<&str>,
        publish_boundary: PublishBoundary,
        _started_at_epoch_seconds: Option<i64>,
        finished_at_epoch_seconds: i64,
    ) -> Result<(), String> {
        let mode = mode.to_owned();
        let release_id = release_id.map(str::to_owned);
        self.with_connection(move |connection| {
            let id = Uuid::new_v4().to_string();
            let status = "succeeded";
            let started_at_epoch_seconds = publish_boundary.started_at_epoch_seconds;
            let snapshot_event_count = publish_boundary.included_event_ids.len() as i64;
            connection
                .execute(
                    "insert into autograph_publish_jobs (
                        id, publish_mode, status, release_id, snapshot_event_count, started_at, finished_at
                    ) values (
                        :1, :2, :3, :4, :5,
                        timestamp '1970-01-01 00:00:00' + numtodsinterval(:6, 'SECOND'),
                        timestamp '1970-01-01 00:00:00' + numtodsinterval(:7, 'SECOND')
                    )",
                    &[
                        &id,
                        &mode,
                        &status,
                        &release_id,
                        &snapshot_event_count,
                        &started_at_epoch_seconds,
                        &finished_at_epoch_seconds,
                    ],
                )
                .map_err(|error| format!("insert Oracle publish job: {error}"))?;
            insert_publish_job_events(&connection, &id, &publish_boundary.included_event_ids)?;
            connection
                .commit()
                .map_err(|error| format!("commit Oracle publish job: {error}"))?;
            Ok(())
        })
        .await
    }

    async fn record_event(&self, event: AutographEditEvent) -> Result<AutographEditEvent, String> {
        self.with_connection(move |connection| {
            insert_edit_event(&connection, &event)?;
            connection
                .commit()
                .map_err(|error| format!("commit Oracle catalog edit event: {error}"))?;
            Ok(event)
        })
        .await
    }

    async fn signer_suggestions(&self, query: String) -> Result<Vec<SignerSuggestion>, String> {
        self.with_connection(move |connection| {
            let normalized_query = normalize_signer_name(&query);
            if normalized_query.is_empty() {
                return Ok(Vec::new());
            }
            let mut suggestions = load_signer_suggestion_profiles(&connection, &normalized_query)?
                .into_iter()
                .filter_map(|profile| {
                    signer_match_rank(&normalized_query, &profile.normalized_name).map(|rank| {
                        (
                            rank,
                            profile.display_name.clone(),
                            SignerSuggestion {
                                profile,
                                possible_duplicate: rank == 0 || rank >= 2,
                            },
                        )
                    })
                })
                .collect::<Vec<_>>();
            suggestions
                .sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
            Ok(suggestions
                .into_iter()
                .map(|(_, _, suggestion)| suggestion)
                .take(10)
                .collect())
        })
        .await
    }

    async fn taxonomy_suggestions(&self) -> Result<TaxonomySuggestions, String> {
        self.with_connection(move |connection| load_taxonomy_suggestions(&connection))
            .await
    }

    async fn update_signer_profile(
        &self,
        signer_id: Uuid,
        input: SignerProfileUpdateInput,
    ) -> Result<SignerProfile, String> {
        self.with_connection(move |connection| {
            let before = load_signer_profile_by_id(&connection, signer_id)?
                .ok_or_else(|| "signer profile was not found".to_owned())?;
            let mut updated = before.clone();
            apply_signer_profile_update(&mut updated, input, now_epoch_seconds())?;
            if let Some(conflict) =
                load_signer_profile_by_normalized_name(&connection, &updated.normalized_name)?
                && conflict.id != signer_id
            {
                return Err("signer normalized name already exists".to_owned());
            }
            let field_diffs = signer_profile_field_diffs(&before, &updated);
            let linked_item_updates = if field_diffs.is_empty() {
                Vec::new()
            } else {
                signer_profile_linked_item_updates(&connection, signer_id, &updated)?
            };
            let credit = SignerCredit {
                signer: updated.clone(),
                sort_order: 0,
                item_role: None,
                item_context: None,
            };
            upsert_signer_profile(&connection, &credit)?;
            if !field_diffs.is_empty() {
                let summary = format!(
                    "Updated signer profile {} -> {}",
                    before.display_name, updated.display_name
                );
                for (item_id, signer) in linked_item_updates {
                    touch_item_legacy_signer_and_history_with_signer(
                        &connection,
                        item_id,
                        &signer,
                        &summary,
                        field_diffs.clone(),
                    )?;
                }
            }
            connection
                .commit()
                .map_err(|error| format!("commit Oracle signer profile update: {error}"))?;
            Ok(updated)
        })
        .await
    }

    async fn merge_signer_profiles(
        &self,
        source_signer_id: Uuid,
        target_signer_id: Uuid,
    ) -> Result<SignerMergeResult, String> {
        self.with_connection(move |connection| {
            if source_signer_id == target_signer_id {
                return Err("source and target signer profiles must differ".to_owned());
            }
            let source = load_signer_profile_by_id(&connection, source_signer_id)?
                .ok_or_else(|| "source signer profile was not found".to_owned())?;
            let target = load_signer_profile_by_id(&connection, target_signer_id)?
                .ok_or_else(|| "target signer profile was not found".to_owned())?;
            let affected_item_ids = load_item_ids_for_signer(&connection, source_signer_id)?;
            let source_id = source_signer_id.to_string();
            let target_id = target_signer_id.to_string();
            connection
                .execute(
                    "delete from autograph_item_signers source_credit
                    where source_credit.signer_id = :1
                      and exists (
                        select 1 from autograph_item_signers target_credit
                        where target_credit.item_id = source_credit.item_id
                          and target_credit.signer_id = :2
                      )",
                    &[&source_id, &target_id],
                )
                .map_err(|error| format!("delete duplicate Oracle signer credits: {error}"))?;
            connection
                .execute(
                    "update autograph_item_signers set signer_id = :1 where signer_id = :2",
                    &[&target_id, &source_id],
                )
                .map_err(|error| format!("merge Oracle signer credits: {error}"))?;
            connection
                .execute("delete from autograph_signers where id = :1", &[&source_id])
                .map_err(|error| format!("delete merged Oracle signer profile: {error}"))?;
            let summary = format!(
                "Merged signer {} into {}",
                source.display_name, target.display_name
            );
            let field_diffs = vec![FieldDiff {
                field: "signers".to_owned(),
                before: serde_json::to_value(&source).unwrap_or(Value::Null),
                after: serde_json::to_value(&target).unwrap_or(Value::Null),
            }];
            for item_id in &affected_item_ids {
                touch_item_legacy_signer_and_history(
                    &connection,
                    *item_id,
                    &summary,
                    field_diffs.clone(),
                )?;
            }
            connection
                .commit()
                .map_err(|error| format!("commit Oracle signer merge: {error}"))?;
            Ok(SignerMergeResult {
                source_signer_id,
                target_signer_id,
                updated_item_count: affected_item_ids.len(),
            })
        })
        .await
    }
}

fn load_item(connection: &Connection, id: Uuid) -> Result<Option<AutographItem>, String> {
    let id_text = id.to_string();
    let mut rows = connection
        .query(LOAD_ITEM_SQL, &[&id_text])
        .map_err(|error| format!("read Oracle catalog item: {error}"))?;
    let Some(row) = rows.next() else {
        return Ok(None);
    };
    let row = row.map_err(|error| format!("read Oracle catalog item row: {error}"))?;
    let mut item = item_from_row(id, &row, 0)?;
    item.tags = load_tags(connection, id)?;
    item.signer_credits = load_signer_credits(connection, id)?;
    if item.signer_credits.is_empty() {
        item.signer_credits = legacy_signer_credits(&item.signer, item.created_at_epoch_seconds);
    }
    item.characters = load_characters(connection, id)?;
    item.franchises = load_franchises(connection, id)?;
    item.images = load_images(connection, id)?;
    Ok(Some(item))
}

fn load_items(
    connection: &Connection,
    publication_status: Option<PublicationStatus>,
) -> Result<Vec<AutographItem>, String> {
    let sql = list_items_sql(publication_status);
    let mut rows = connection
        .query(&sql, &[])
        .map_err(|error| format!("list Oracle catalog items: {error}"))?;
    let mut items = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle catalog item row: {error}"))?;
        let id = parse_uuid(&row_value::<String>(&row, 0, "item id")?)?;
        items.push(item_from_row(id, &row, 1)?);
    }

    let mut tags = load_tags_for_items(connection, publication_status)?;
    let mut signer_credits = load_signer_credits_for_items(connection, publication_status)?;
    let mut characters = load_ordered_values_for_items(
        connection,
        publication_status,
        "autograph_item_characters",
        "character_name",
        "characters",
    )?;
    let mut franchises = load_ordered_values_for_items(
        connection,
        publication_status,
        "autograph_item_franchises",
        "franchise",
        "franchises",
    )?;
    let mut images = load_images_for_items(connection, publication_status)?;

    for item in &mut items {
        item.tags = tags.remove(&item.id).unwrap_or_default();
        item.signer_credits = signer_credits.remove(&item.id).unwrap_or_default();
        if item.signer_credits.is_empty() {
            item.signer_credits =
                legacy_signer_credits(&item.signer, item.created_at_epoch_seconds);
        }
        item.characters = characters.remove(&item.id).unwrap_or_default();
        item.franchises = franchises.remove(&item.id).unwrap_or_default();
        item.images = images.remove(&item.id).unwrap_or_default();
    }

    Ok(items)
}

fn list_items_sql(publication_status: Option<PublicationStatus>) -> String {
    match publication_status {
        Some(PublicationStatus::Published) => {
            format!(
                "select id, {} from autograph_items where publication_status = 'published' order by title, id",
                ITEM_SELECT_COLUMNS
            )
        }
        Some(PublicationStatus::Draft) => {
            format!(
                "select id, {} from autograph_items where publication_status = 'draft' order by title, id",
                ITEM_SELECT_COLUMNS
            )
        }
        Some(PublicationStatus::Archived) => {
            format!(
                "select id, {} from autograph_items where publication_status = 'archived' order by title, id",
                ITEM_SELECT_COLUMNS
            )
        }
        None => {
            format!(
                "select id, {} from autograph_items order by title, id",
                ITEM_SELECT_COLUMNS
            )
        }
    }
}

fn item_from_row(id: Uuid, row: &Row, offset: usize) -> Result<AutographItem, String> {
    let signer: String = row_value(row, offset + 1, "signer")?;
    let created_at_epoch_seconds =
        row_value::<Option<i64>>(row, offset + 18, "created at")?.unwrap_or_default();
    Ok(AutographItem {
        id,
        title: row_value(row, offset, "title")?,
        signer: signer.clone(),
        description: row_value(row, offset + 2, "description")?,
        category: row_value(row, offset + 3, "category")?,
        signer_credits: Vec::new(),
        characters: Vec::new(),
        format: row_value(row, offset + 13, "format")?,
        origin: parse_item_origin(&row_value::<String>(row, offset + 14, "origin")?)?,
        franchises: Vec::new(),
        product_line: row_value(row, offset + 16, "product line")?,
        set_name: row_value(row, offset + 17, "set name")?,
        language: row_value(row, offset + 15, "language")?,
        object_reference: row_value(row, offset + 4, "object reference")?,
        event_name: row_value(row, offset + 5, "event name")?,
        event_location: row_value(row, offset + 6, "event location")?,
        source: row_value(row, offset + 7, "source")?,
        inscription: row_value(row, offset + 8, "inscription")?,
        certification_company: row_value(row, offset + 9, "certification company")?,
        certification_id: row_value(row, offset + 10, "certification id")?,
        estimated_year: row_value(row, offset + 11, "estimated year")?,
        publication_status: parse_publication_status(&row_value::<String>(
            row,
            offset + 12,
            "publication status",
        )?)?,
        tags: Vec::new(),
        images: Vec::new(),
        created_at_epoch_seconds,
        updated_at_epoch_seconds: row_value::<Option<i64>>(row, offset + 19, "updated at")?
            .unwrap_or_default(),
    })
}

fn legacy_signer_credits(signer: &str, created_at_epoch_seconds: i64) -> Vec<SignerCredit> {
    vec![SignerCredit {
        signer: SignerProfile {
            id: Uuid::nil(),
            display_name: signer.to_owned(),
            normalized_name: normalize_signer_name(signer),
            default_role: None,
            wikipedia_url: None,
            imdb_url: None,
            created_at_epoch_seconds,
            updated_at_epoch_seconds: created_at_epoch_seconds,
        },
        sort_order: 0,
        item_role: None,
        item_context: None,
    }]
}

fn resolve_oracle_signer_credits(
    connection: &Connection,
    inputs: &[SignerCreditInput],
    fallback_signer: &str,
) -> Result<Vec<SignerCredit>, String> {
    let now = now_epoch_seconds();
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
        let profile = resolve_oracle_signer_profile(connection, input, now)?;
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

fn resolve_oracle_signer_profile(
    connection: &Connection,
    input: &SignerCreditInput,
    now: i64,
) -> Result<SignerProfile, String> {
    if let Some(signer_id) = input.signer_id {
        let profile = load_signer_profile_by_id(connection, signer_id)?
            .ok_or_else(|| "signer profile was not found".to_owned())?;
        validate_signer_id_display_name(&profile, input)?;
        return Ok(profile);
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
    if let Some(profile) = load_signer_profile_by_normalized_name(connection, &normalized_name)? {
        return Ok(profile);
    }
    let mut profile = SignerProfile {
        id: Uuid::new_v4(),
        display_name: display_name.to_owned(),
        normalized_name,
        default_role: None,
        wikipedia_url: None,
        imdb_url: None,
        created_at_epoch_seconds: now,
        updated_at_epoch_seconds: now,
    };
    apply_signer_input_to_profile(&mut profile, input, now)?;
    upsert_signer_profile(
        connection,
        &SignerCredit {
            signer: profile,
            sort_order: 0,
            item_role: None,
            item_context: None,
        },
    )
}

fn validate_signer_id_display_name(
    profile: &SignerProfile,
    input: &SignerCreditInput,
) -> Result<(), String> {
    let Some(display_name) = input.display_name.as_deref().map(str::trim) else {
        return Ok(());
    };
    if display_name.is_empty() {
        return Ok(());
    }
    let normalized_name = normalize_signer_name(display_name);
    if normalized_name != profile.normalized_name {
        return Err(
            "signerId cannot be combined with a conflicting displayName; choose the matching signer or create a new signer"
                .to_owned(),
        );
    }
    Ok(())
}

fn load_signer_profile_by_id(
    connection: &Connection,
    signer_id: Uuid,
) -> Result<Option<SignerProfile>, String> {
    let signer_id = signer_id.to_string();
    load_signer_profile_by_query(
        connection,
        "select
            id, display_name, normalized_name, default_role, wikipedia_url, imdb_url,
            cast(round((cast(created_at as date) - date '1970-01-01') * 86400) as number(19)),
            cast(round((cast(updated_at as date) - date '1970-01-01') * 86400) as number(19))
        from autograph_signers where id = :1",
        &[&signer_id],
        "id",
    )
}

fn load_signer_profile_by_normalized_name(
    connection: &Connection,
    normalized_name: &str,
) -> Result<Option<SignerProfile>, String> {
    load_signer_profile_by_query(
        connection,
        "select
            id, display_name, normalized_name, default_role, wikipedia_url, imdb_url,
            cast(round((cast(created_at as date) - date '1970-01-01') * 86400) as number(19)),
            cast(round((cast(updated_at as date) - date '1970-01-01') * 86400) as number(19))
        from autograph_signers where normalized_name = :1",
        &[&normalized_name],
        "normalized name",
    )
}

fn load_signer_profile_by_query(
    connection: &Connection,
    sql: &str,
    params: &[&dyn ToDbValue],
    lookup: &str,
) -> Result<Option<SignerProfile>, String> {
    let mut rows = connection
        .query(sql, params)
        .map_err(|error| format!("read Oracle signer profile by {lookup}: {error}"))?;
    let Some(row) = rows.next() else {
        return Ok(None);
    };
    let row = row.map_err(|error| format!("read Oracle signer profile row: {error}"))?;
    Ok(Some(signer_profile_from_row(&row, 0)?))
}

fn load_signer_suggestion_profiles(
    connection: &Connection,
    normalized_query: &str,
) -> Result<Vec<SignerProfile>, String> {
    let prefix_query = format!("{normalized_query}%");
    let contains_query = format!("%{normalized_query}%");
    let mut rows = connection
        .query(
            "select
                id, display_name, normalized_name, default_role, wikipedia_url, imdb_url,
                cast(round((cast(created_at as date) - date '1970-01-01') * 86400) as number(19)),
                cast(round((cast(updated_at as date) - date '1970-01-01') * 86400) as number(19))
            from autograph_signers
            where normalized_name = :1
               or normalized_name like :2
               or normalized_name like :3
            order by case when normalized_name = :1 then 0 else 1 end, display_name, id
            fetch first 10 rows only",
            &[&normalized_query, &prefix_query, &contains_query],
        )
        .map_err(|error| format!("read Oracle signer suggestion profiles: {error}"))?;
    let mut profiles = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle signer suggestion row: {error}"))?;
        profiles.push(signer_profile_from_row(&row, 0)?);
    }
    Ok(profiles)
}

fn load_all_signer_profiles(connection: &Connection) -> Result<Vec<SignerProfile>, String> {
    let mut rows = connection
        .query(
            "select
                id, display_name, normalized_name, default_role, wikipedia_url, imdb_url,
                cast(round((cast(created_at as date) - date '1970-01-01') * 86400) as number(19)),
                cast(round((cast(updated_at as date) - date '1970-01-01') * 86400) as number(19))
            from autograph_signers
            order by display_name, id",
            &[],
        )
        .map_err(|error| format!("read Oracle signer profiles: {error}"))?;
    let mut profiles = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle signer profile row: {error}"))?;
        profiles.push(signer_profile_from_row(&row, 0)?);
    }
    Ok(profiles)
}

fn load_item_ids_for_signer(connection: &Connection, signer_id: Uuid) -> Result<Vec<Uuid>, String> {
    let signer_id = signer_id.to_string();
    let mut rows = connection
        .query(
            "select item_id from autograph_item_signers where signer_id = :1 order by item_id",
            &[&signer_id],
        )
        .map_err(|error| format!("read Oracle signer linked items: {error}"))?;
    let mut item_ids = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle signer linked item row: {error}"))?;
        item_ids.push(parse_uuid(&row_value::<String>(
            &row,
            0,
            "signer linked item id",
        )?)?);
    }
    Ok(item_ids)
}

fn signer_profile_linked_item_updates(
    connection: &Connection,
    signer_id: Uuid,
    updated_profile: &SignerProfile,
) -> Result<Vec<(Uuid, String)>, String> {
    load_item_ids_for_signer(connection, signer_id)?
        .into_iter()
        .map(|item_id| {
            let mut item = load_item(connection, item_id)?
                .ok_or_else(|| "linked Oracle signer item was not found".to_owned())?;
            for credit in &mut item.signer_credits {
                if credit.signer.id == signer_id {
                    credit.signer = updated_profile.clone();
                }
            }
            Ok((item_id, compact_signer_text(&item.signer_credits)))
        })
        .collect()
}

fn touch_item_legacy_signer_and_history(
    connection: &Connection,
    item_id: Uuid,
    summary: &str,
    field_diffs: Vec<FieldDiff>,
) -> Result<(), String> {
    let item = load_item(connection, item_id)?
        .ok_or_else(|| "linked Oracle signer item was not found".to_owned())?;
    let signer = compact_signer_text(&item.signer_credits);
    touch_item_legacy_signer_and_history_with_signer(
        connection,
        item_id,
        &signer,
        summary,
        field_diffs,
    )
}

fn touch_item_legacy_signer_and_history_with_signer(
    connection: &Connection,
    item_id: Uuid,
    signer: &str,
    summary: &str,
    field_diffs: Vec<FieldDiff>,
) -> Result<(), String> {
    let item_id_text = item_id.to_string();
    connection
        .execute(
            "update autograph_items set signer = :1, updated_at = current_timestamp where id = :2",
            &[&signer, &item_id_text],
        )
        .map_err(|error| format!("touch Oracle signer linked item: {error}"))?;
    insert_edit_event(
        connection,
        &AutographEditEvent::new(
            item_id,
            EditEventKind::MetadataUpdated,
            summary.to_owned(),
            field_diffs,
            now_epoch_seconds(),
        ),
    )
}

fn load_taxonomy_suggestions(connection: &Connection) -> Result<TaxonomySuggestions, String> {
    let mut suggestions = TaxonomySuggestions {
        signers: load_all_signer_profiles(connection)?,
        origins: vec![ItemOrigin::Official, ItemOrigin::Custom],
        ..Default::default()
    };
    suggestions.characters = load_distinct_strings(
        connection,
        "select distinct character_name from autograph_item_characters order by character_name",
        "characters",
    )?;
    suggestions.formats = load_distinct_strings(
        connection,
        "select distinct format from autograph_items where format is not null order by format",
        "formats",
    )?;
    suggestions.franchises = load_distinct_strings(
        connection,
        "select distinct franchise from autograph_item_franchises order by franchise",
        "franchises",
    )?;
    suggestions.product_lines = load_distinct_strings(
        connection,
        "select distinct product_line from autograph_items where product_line is not null order by product_line",
        "product lines",
    )?;
    suggestions.set_names = load_distinct_strings(
        connection,
        "select distinct set_name from autograph_items where set_name is not null order by set_name",
        "set names",
    )?;
    suggestions.languages = load_distinct_strings(
        connection,
        "select distinct language from autograph_items where language is not null order by language",
        "languages",
    )?;
    suggestions.roles = load_distinct_strings(
        connection,
        "select distinct role_value from (
            select default_role as role_value from autograph_signers where default_role is not null
            union
            select item_role as role_value from autograph_item_signers where item_role is not null
        ) order by role_value",
        "roles",
    )?;
    suggestions.tags = load_distinct_strings(
        connection,
        "select distinct tag from autograph_item_tags order by tag",
        "tags",
    )?;
    Ok(suggestions)
}

fn load_distinct_strings(
    connection: &Connection,
    sql: &str,
    label: &str,
) -> Result<Vec<String>, String> {
    let mut rows = connection
        .query(sql, &[])
        .map_err(|error| format!("read Oracle taxonomy {label}: {error}"))?;
    let mut values = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle taxonomy {label} row: {error}"))?;
        values.push(row_value(&row, 0, label)?);
    }
    Ok(values)
}

fn child_status_filter(publication_status: Option<PublicationStatus>) -> &'static str {
    match publication_status {
        Some(PublicationStatus::Published) => " where i.publication_status = 'published'",
        Some(PublicationStatus::Draft) => " where i.publication_status = 'draft'",
        Some(PublicationStatus::Archived) => " where i.publication_status = 'archived'",
        None => "",
    }
}

fn load_tags_for_items(
    connection: &Connection,
    publication_status: Option<PublicationStatus>,
) -> Result<BTreeMap<Uuid, Vec<String>>, String> {
    let sql = format!(
        "select t.item_id, t.tag
         from autograph_item_tags t
         join autograph_items i on i.id = t.item_id
         {}
         order by t.item_id, t.tag",
        child_status_filter(publication_status)
    );
    let mut rows = connection
        .query(&sql, &[])
        .map_err(|error| format!("bulk read Oracle catalog tags: {error}"))?;
    let mut tags = BTreeMap::<Uuid, Vec<String>>::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("bulk read Oracle catalog tag row: {error}"))?;
        let item_id = parse_uuid(&row_value::<String>(&row, 0, "tag item id")?)?;
        tags.entry(item_id)
            .or_default()
            .push(row_value(&row, 1, "tag")?);
    }
    Ok(tags)
}

fn load_signer_credits_for_items(
    connection: &Connection,
    publication_status: Option<PublicationStatus>,
) -> Result<BTreeMap<Uuid, Vec<SignerCredit>>, String> {
    let sql = format!(
        "select
            cis.item_id,
            s.id, s.display_name, s.normalized_name, s.default_role,
            s.wikipedia_url, s.imdb_url,
            cast(round((cast(s.created_at as date) - date '1970-01-01') * 86400) as number(19)),
            cast(round((cast(s.updated_at as date) - date '1970-01-01') * 86400) as number(19)),
            cis.sort_order, cis.item_role, cis.item_context
        from autograph_item_signers cis
        join autograph_items i on i.id = cis.item_id
        join autograph_signers s on s.id = cis.signer_id
        {}
        order by cis.item_id, cis.sort_order, s.display_name, s.id",
        child_status_filter(publication_status)
    );
    let mut rows = connection
        .query(&sql, &[])
        .map_err(|error| format!("bulk read Oracle catalog signer credits: {error}"))?;
    let mut credits = BTreeMap::<Uuid, Vec<SignerCredit>>::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("bulk read Oracle signer credit row: {error}"))?;
        let item_id = parse_uuid(&row_value::<String>(&row, 0, "signer credit item id")?)?;
        credits.entry(item_id).or_default().push(SignerCredit {
            signer: signer_profile_from_row(&row, 1)?,
            sort_order: row_value(&row, 9, "signer credit sort order")?,
            item_role: row_value(&row, 10, "signer credit item role")?,
            item_context: row_value(&row, 11, "signer credit item context")?,
        });
    }
    Ok(credits)
}

fn load_ordered_values_for_items(
    connection: &Connection,
    publication_status: Option<PublicationStatus>,
    table: &str,
    column: &str,
    label: &str,
) -> Result<BTreeMap<Uuid, Vec<String>>, String> {
    let sql = format!(
        "select child.item_id, child.{column}
         from {table} child
         join autograph_items i on i.id = child.item_id
         {}
         order by child.item_id, child.sort_order, child.{column}",
        child_status_filter(publication_status)
    );
    let mut rows = connection
        .query(&sql, &[])
        .map_err(|error| format!("bulk read Oracle catalog {label}: {error}"))?;
    let mut values = BTreeMap::<Uuid, Vec<String>>::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("bulk read Oracle catalog {label} row: {error}"))?;
        let item_id = parse_uuid(&row_value::<String>(&row, 0, "ordered value item id")?)?;
        values
            .entry(item_id)
            .or_default()
            .push(row_value(&row, 1, label)?);
    }
    Ok(values)
}

fn load_images_for_items(
    connection: &Connection,
    publication_status: Option<PublicationStatus>,
) -> Result<BTreeMap<Uuid, Vec<AutographImage>>, String> {
    let sql = load_images_for_items_sql(publication_status);
    let mut rows = connection
        .query(&sql, &[])
        .map_err(|error| format!("bulk read Oracle catalog images: {error}"))?;
    let mut images = BTreeMap::<Uuid, Vec<AutographImage>>::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("bulk read Oracle catalog image row: {error}"))?;
        let item_id = parse_uuid(&row_value::<String>(&row, 0, "image item id")?)?;
        images
            .entry(item_id)
            .or_default()
            .push(image_from_row(&row, 1)?);
    }
    Ok(images)
}

fn load_images_for_items_sql(publication_status: Option<PublicationStatus>) -> String {
    format!(
        "select
            img.item_id, img.id, img.object_key, img.original_filename, img.content_type,
            img.byte_size, img.checksum, img.etag, img.is_primary, img.sort_order, img.alt_text
         from autograph_images img
         join autograph_items i on i.id = img.item_id
         {}
         order by img.item_id, img.sort_order, img.id",
        child_status_filter(publication_status)
    )
}

fn apply_signer_input_to_profile(
    profile: &mut SignerProfile,
    input: &SignerCreditInput,
    now: i64,
) -> Result<(), String> {
    let mut changed = false;
    if let Some(display_name) = normalize_optional_string(input.display_name.clone()) {
        let normalized_name = normalize_signer_name(&display_name);
        if profile.display_name != display_name || profile.normalized_name != normalized_name {
            profile.display_name = display_name;
            profile.normalized_name = normalized_name;
            changed = true;
        }
    }
    let normalized_role = normalize_optional_string(input.default_role.clone());
    if profile.default_role != normalized_role {
        profile.default_role = normalized_role;
        changed = true;
    }
    for (current, incoming, field) in [
        (
            &mut profile.wikipedia_url,
            input.wikipedia_url.clone(),
            "wikipediaUrl",
        ),
        (&mut profile.imdb_url, input.imdb_url.clone(), "imdbUrl"),
    ] {
        let normalized = normalize_profile_link(incoming, field)?;
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

fn signer_profile_from_row(row: &Row, offset: usize) -> Result<SignerProfile, String> {
    Ok(SignerProfile {
        id: parse_uuid(&row_value::<String>(row, offset, "signer id")?)?,
        display_name: row_value(row, offset + 1, "signer display name")?,
        normalized_name: row_value(row, offset + 2, "signer normalized name")?,
        default_role: row_value(row, offset + 3, "signer default role")?,
        wikipedia_url: row_value(row, offset + 4, "signer wikipedia url")?,
        imdb_url: row_value(row, offset + 5, "signer imdb url")?,
        created_at_epoch_seconds: row_value::<Option<i64>>(row, offset + 6, "signer created at")?
            .unwrap_or_default(),
        updated_at_epoch_seconds: row_value::<Option<i64>>(row, offset + 7, "signer updated at")?
            .unwrap_or_default(),
    })
}

fn load_tags(connection: &Connection, id: Uuid) -> Result<Vec<String>, String> {
    let id_text = id.to_string();
    let mut rows = connection
        .query(
            "select tag from autograph_item_tags where item_id = :1 order by tag",
            &[&id_text],
        )
        .map_err(|error| format!("read Oracle catalog tags: {error}"))?;
    let mut tags = Vec::new();
    for row in &mut rows {
        tags.push(
            row.map_err(|error| format!("read Oracle catalog tag row: {error}"))?
                .get(0)
                .map_err(|error| format!("read Oracle catalog tag: {error}"))?,
        );
    }
    Ok(tags)
}

fn load_signer_credits(connection: &Connection, id: Uuid) -> Result<Vec<SignerCredit>, String> {
    let id_text = id.to_string();
    let mut rows = connection
        .query(
            "select
                s.id, s.display_name, s.normalized_name, s.default_role,
                s.wikipedia_url, s.imdb_url,
                cast(round((cast(s.created_at as date) - date '1970-01-01') * 86400) as number(19)),
                cast(round((cast(s.updated_at as date) - date '1970-01-01') * 86400) as number(19)),
                cis.sort_order, cis.item_role, cis.item_context
            from autograph_item_signers cis
            join autograph_signers s on s.id = cis.signer_id
            where cis.item_id = :1
            order by cis.sort_order, s.display_name, s.id",
            &[&id_text],
        )
        .map_err(|error| format!("read Oracle catalog signer credits: {error}"))?;
    let mut credits = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle signer credit row: {error}"))?;
        credits.push(SignerCredit {
            signer: signer_profile_from_row(&row, 0)?,
            sort_order: row_value(&row, 8, "signer credit sort order")?,
            item_role: row_value(&row, 9, "signer credit item role")?,
            item_context: row_value(&row, 10, "signer credit item context")?,
        });
    }
    Ok(credits)
}

fn has_persisted_signer_credits(connection: &Connection, id: Uuid) -> Result<bool, String> {
    let id_text = id.to_string();
    let count: i64 = query_row_value(
        connection,
        "select count(*) from autograph_item_signers where item_id = :1",
        &[&id_text],
        "signer credit count",
    )?;
    Ok(count > 0)
}

fn should_sync_signer_credits(
    input_updates_signer_credits: bool,
    has_persisted_signer_credits: bool,
    signer_credits_changed: bool,
) -> bool {
    input_updates_signer_credits && (signer_credits_changed || !has_persisted_signer_credits)
}

fn load_characters(connection: &Connection, id: Uuid) -> Result<Vec<String>, String> {
    load_ordered_values(
        connection,
        id,
        "select character_name from autograph_item_characters where item_id = :1 order by sort_order, character_name",
        "characters",
    )
}

fn load_franchises(connection: &Connection, id: Uuid) -> Result<Vec<String>, String> {
    load_ordered_values(
        connection,
        id,
        "select franchise from autograph_item_franchises where item_id = :1 order by sort_order, franchise",
        "franchises",
    )
}

fn load_ordered_values(
    connection: &Connection,
    id: Uuid,
    sql: &str,
    label: &str,
) -> Result<Vec<String>, String> {
    let id_text = id.to_string();
    let mut rows = connection
        .query(sql, &[&id_text])
        .map_err(|error| format!("read Oracle catalog {label}: {error}"))?;
    let mut values = Vec::new();
    for row in &mut rows {
        values.push(
            row.map_err(|error| format!("read Oracle catalog {label} row: {error}"))?
                .get(0)
                .map_err(|error| format!("read Oracle catalog {label} value: {error}"))?,
        );
    }
    Ok(values)
}

fn load_images(connection: &Connection, id: Uuid) -> Result<Vec<AutographImage>, String> {
    let id_text = id.to_string();
    let sql = format!(
        "select {IMAGE_SELECT_COLUMNS}
            from autograph_images where item_id = :1 order by sort_order, id"
    );
    let mut rows = connection
        .query(&sql, &[&id_text])
        .map_err(|error| format!("read Oracle catalog images: {error}"))?;
    let mut images = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle catalog image row: {error}"))?;
        images.push(image_from_row(&row, 0)?);
    }
    Ok(images)
}

fn load_image(
    connection: &Connection,
    item_id: Uuid,
    image_id: Uuid,
) -> Result<Option<AutographImage>, String> {
    let item_id_text = item_id.to_string();
    let image_id_text = image_id.to_string();
    let sql = format!(
        "select {IMAGE_SELECT_COLUMNS}
            from autograph_images where item_id = :1 and id = :2"
    );
    let mut rows = connection
        .query(&sql, &[&item_id_text, &image_id_text])
        .map_err(|error| format!("read Oracle catalog image: {error}"))?;
    let Some(row) = rows.next() else {
        return Ok(None);
    };
    let row = row.map_err(|error| format!("read Oracle catalog image row: {error}"))?;
    Ok(Some(image_from_row(&row, 0)?))
}

fn image_from_row(row: &Row, offset: usize) -> Result<AutographImage, String> {
    Ok(AutographImage {
        id: parse_uuid(&row_value::<String>(row, offset, "image id")?)?,
        object_key: row_value(row, offset + 1, "image object key")?,
        original_filename: row_value::<Option<String>>(row, offset + 2, "image original filename")?
            .unwrap_or_else(|| "upload".to_owned()),
        content_type: row_value(row, offset + 3, "image content type")?,
        byte_size: row_value::<Option<i64>>(row, offset + 4, "image byte size")?.unwrap_or(0)
            as usize,
        checksum: row_value(row, offset + 5, "image checksum")?,
        etag: row_value(row, offset + 6, "image etag")?,
        is_primary: row_value::<String>(row, offset + 7, "image primary flag")? == "Y",
        sort_order: row_value(row, offset + 8, "image sort order")?,
        alt_text: row_value(row, offset + 9, "image alt text")?,
        adjustment: None,
    })
}

fn promote_first_remaining_image(connection: &Connection, item_id: Uuid) -> Result<(), String> {
    let item_id_text = item_id.to_string();
    let next_primary = query_row_value::<String>(
        connection,
        "select id from autograph_images where item_id = :1 order by sort_order, id fetch first 1 row only",
        &[&item_id_text],
        "next primary image",
    )
    .ok();
    if let Some(next_primary) = next_primary {
        connection
            .execute(
                "update autograph_images set
                    is_primary = case when id = :1 then 'Y' else 'N' end,
                    updated_at = current_timestamp
                where item_id = :2",
                &[&next_primary, &item_id_text],
            )
            .map_err(|error| format!("promote Oracle primary image after removal: {error}"))?;
    }
    Ok(())
}

fn load_history(connection: &Connection, item_id: Uuid) -> Result<Vec<AutographEditEvent>, String> {
    let item_id_text = item_id.to_string();
    let mut rows = connection
        .query(
            "select
                id, event_type, summary, field_diffs_json,
                cast(round((cast(created_at as date) - date '1970-01-01') * 86400) as number(19))
            from autograph_edit_events
            where item_id = :1
            order by created_at desc, id desc",
            &[&item_id_text],
        )
        .map_err(|error| format!("read Oracle catalog edit history: {error}"))?;
    let mut events = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle catalog edit history row: {error}"))?;
        events.push(event_from_row(item_id, &row)?);
    }
    Ok(events)
}

fn load_cleanup_warnings(
    connection: &Connection,
    item_id: Uuid,
) -> Result<Vec<CleanupWarning>, String> {
    let item_id_text = item_id.to_string();
    let mut rows = connection
        .query(
            "select image_id, target_object_key, operation, status, admin_message
            from autograph_cleanup_events
            where item_id = :1 and status = 'deleteFailed'
            order by created_at desc, id desc",
            &[&item_id_text],
        )
        .map_err(|error| format!("read Oracle cleanup warnings: {error}"))?;
    let mut warnings = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle cleanup warning row: {error}"))?;
        warnings.push(CleanupWarning {
            image_id: parse_uuid(&row_value::<String>(&row, 0, "cleanup image id")?)?,
            target_object_key: row_value(&row, 1, "cleanup target object key")?,
            operation: row_value(&row, 2, "cleanup operation")?,
            status: row_value::<String>(&row, 3, "cleanup status")?.parse::<CleanupStatus>()?,
            admin_message: row_value(&row, 4, "cleanup admin message")?,
        });
    }
    Ok(warnings)
}

fn event_from_row(item_id: Uuid, row: &Row) -> Result<AutographEditEvent, String> {
    let field_diffs_json = row_value::<Option<String>>(row, 3, "edit event field diffs")?
        .unwrap_or_else(|| "[]".to_owned());
    let field_diffs = serde_json::from_str::<Vec<FieldDiff>>(&field_diffs_json)
        .map_err(|error| format!("parse Oracle catalog edit event field diffs: {error}"))?;
    Ok(AutographEditEvent {
        id: parse_uuid(&row_value::<String>(row, 0, "edit event id")?)?,
        item_id,
        kind: row_value::<String>(row, 1, "edit event type")?.parse::<EditEventKind>()?,
        summary: row_value(row, 2, "edit event summary")?,
        field_diffs,
        created_at_epoch_seconds: row_value::<Option<i64>>(row, 4, "edit event created at")?
            .unwrap_or_default(),
    })
}

fn replace_tags(connection: &Connection, id: Uuid, tags: &[String]) -> Result<(), String> {
    let id_text = id.to_string();
    connection
        .execute(
            "delete from autograph_item_tags where item_id = :1",
            &[&id_text],
        )
        .map_err(|error| format!("clear Oracle catalog tags: {error}"))?;
    for tag in tags {
        connection
            .execute(
                "insert into autograph_item_tags (item_id, tag) values (:1, :2)",
                &[&id_text, tag],
            )
            .map_err(|error| format!("insert Oracle catalog tag: {error}"))?;
    }
    Ok(())
}

fn replace_signer_credits(
    connection: &Connection,
    id: Uuid,
    credits: &[SignerCredit],
) -> Result<(), String> {
    let id_text = id.to_string();
    connection
        .execute(
            "delete from autograph_item_signers where item_id = :1",
            &[&id_text],
        )
        .map_err(|error| format!("clear Oracle catalog signer credits: {error}"))?;
    for (index, credit) in credits.iter().enumerate() {
        let signer_id = credit.signer.id.to_string();
        let sort_order = index as i32;
        connection
            .execute(
                "insert into autograph_item_signers (
                    item_id, signer_id, sort_order, item_role, item_context
                ) values (
                    :1, :2, :3, :4, :5
                )",
                &[
                    &id_text,
                    &signer_id,
                    &sort_order,
                    &credit.item_role,
                    &credit.item_context,
                ],
            )
            .map_err(|error| format!("insert Oracle catalog signer credit: {error}"))?;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SignerCreditRow {
    signer_id: String,
    sort_order: i32,
    item_role: Option<String>,
    item_context: Option<String>,
}

fn sync_signer_credits(
    connection: &Connection,
    id: Uuid,
    credits: &[SignerCredit],
) -> Result<(), String> {
    let id_text = id.to_string();
    let desired_rows = desired_signer_credit_rows(credits);
    let existing_rows = load_signer_credit_rows_for_update(connection, id)?;
    if existing_rows == desired_rows {
        return Ok(());
    }

    let desired_ids = desired_rows
        .iter()
        .map(|row| row.signer_id.clone())
        .collect::<BTreeSet<_>>();
    for row in existing_rows
        .iter()
        .filter(|row| !desired_ids.contains(&row.signer_id))
    {
        connection
            .execute(
                "delete from autograph_item_signers where item_id = :1 and signer_id = :2",
                &[&id_text, &row.signer_id],
            )
            .map_err(|error| format!("delete Oracle catalog signer credit: {error}"))?;
    }

    for row in &desired_rows {
        let statement = connection
            .execute(
                "update autograph_item_signers set
                    sort_order = :3,
                    item_role = :4,
                    item_context = :5
                where item_id = :1 and signer_id = :2",
                &[
                    &id_text,
                    &row.signer_id,
                    &row.sort_order,
                    &row.item_role,
                    &row.item_context,
                ],
            )
            .map_err(|error| format!("update Oracle catalog signer credit: {error}"))?;
        let rows_updated = statement.rows_affected();
        if rows_updated == 0 {
            connection
                .execute(
                    "insert into autograph_item_signers (
                        item_id, signer_id, sort_order, item_role, item_context
                    ) values (
                        :1, :2, :3, :4, :5
                    )",
                    &[
                        &id_text,
                        &row.signer_id,
                        &row.sort_order,
                        &row.item_role,
                        &row.item_context,
                    ],
                )
                .map_err(|error| format!("insert Oracle catalog signer credit: {error}"))?;
        }
    }

    Ok(())
}

fn desired_signer_credit_rows(credits: &[SignerCredit]) -> Vec<SignerCreditRow> {
    let mut rows = credits
        .iter()
        .enumerate()
        .map(|(index, credit)| SignerCreditRow {
            signer_id: credit.signer.id.to_string(),
            sort_order: index as i32,
            item_role: credit.item_role.clone(),
            item_context: credit.item_context.clone(),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.signer_id.cmp(&right.signer_id));
    rows
}

fn load_signer_credit_rows_for_update(
    connection: &Connection,
    id: Uuid,
) -> Result<Vec<SignerCreditRow>, String> {
    let id_text = id.to_string();
    let mut rows = connection
        .query(SIGNER_CREDIT_ROWS_FOR_UPDATE_SQL, &[&id_text])
        .map_err(|error| format!("lock Oracle catalog signer credits: {error}"))?;
    let mut credits = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle signer credit lock row: {error}"))?;
        credits.push(SignerCreditRow {
            signer_id: row_value(&row, 0, "signer credit signer id")?,
            sort_order: row_value(&row, 1, "signer credit sort order")?,
            item_role: row_value(&row, 2, "signer credit item role")?,
            item_context: row_value(&row, 3, "signer credit item context")?,
        });
    }
    Ok(credits)
}

fn upsert_signer_profile(
    connection: &Connection,
    credit: &SignerCredit,
) -> Result<SignerProfile, String> {
    let normalized_name = normalize_signer_name(&credit.signer.display_name);
    if normalized_name.is_empty() {
        return Err("signer displayName is required".to_owned());
    }
    let requested_id = credit.signer.id.to_string();
    let mut rows = connection
        .query(
            "select
                id, display_name, normalized_name, default_role, wikipedia_url, imdb_url,
                cast(round((cast(created_at as date) - date '1970-01-01') * 86400) as number(19)),
                cast(round((cast(updated_at as date) - date '1970-01-01') * 86400) as number(19))
            from autograph_signers
            where id = :1 or normalized_name = :2
            order by case when id = :1 then 0 else 1 end
            fetch first 1 row only",
            &[&requested_id, &normalized_name],
        )
        .map_err(|error| format!("read Oracle signer profile: {error}"))?;
    if let Some(row) = rows.next() {
        let row = row.map_err(|error| format!("read Oracle signer profile row: {error}"))?;
        let existing = signer_profile_from_row(&row, 0)?;
        let id_text = existing.id.to_string();
        connection
            .execute(
                "update autograph_signers set
                    display_name = :1,
                    normalized_name = :2,
                    default_role = :3,
                    wikipedia_url = :4,
                    imdb_url = :5,
                    updated_at = current_timestamp
                where id = :6",
                &[
                    &credit.signer.display_name,
                    &normalized_name,
                    &credit.signer.default_role,
                    &credit.signer.wikipedia_url,
                    &credit.signer.imdb_url,
                    &id_text,
                ],
            )
            .map_err(|error| format!("update Oracle signer profile: {error}"))?;
        return Ok(SignerProfile {
            id: existing.id,
            display_name: credit.signer.display_name.clone(),
            normalized_name,
            default_role: credit.signer.default_role.clone(),
            wikipedia_url: credit.signer.wikipedia_url.clone(),
            imdb_url: credit.signer.imdb_url.clone(),
            created_at_epoch_seconds: existing.created_at_epoch_seconds,
            updated_at_epoch_seconds: now_epoch_seconds(),
        });
    }

    connection
        .execute(
            "insert into autograph_signers (
                id, display_name, normalized_name, default_role, wikipedia_url, imdb_url
            ) values (
                :1, :2, :3, :4, :5, :6
            )",
            &[
                &requested_id,
                &credit.signer.display_name,
                &normalized_name,
                &credit.signer.default_role,
                &credit.signer.wikipedia_url,
                &credit.signer.imdb_url,
            ],
        )
        .map_err(|error| format!("insert Oracle signer profile: {error}"))?;
    Ok(SignerProfile {
        normalized_name,
        ..credit.signer.clone()
    })
}

fn replace_characters(
    connection: &Connection,
    id: Uuid,
    characters: &[String],
) -> Result<(), String> {
    replace_ordered_values(
        connection,
        id,
        characters,
        "delete from autograph_item_characters where item_id = :1",
        "insert into autograph_item_characters (item_id, character_name, sort_order) values (:1, :2, :3)",
        "characters",
    )
}

fn replace_franchises(
    connection: &Connection,
    id: Uuid,
    franchises: &[String],
) -> Result<(), String> {
    replace_ordered_values(
        connection,
        id,
        franchises,
        "delete from autograph_item_franchises where item_id = :1",
        "insert into autograph_item_franchises (item_id, franchise, sort_order) values (:1, :2, :3)",
        "franchises",
    )
}

fn replace_ordered_values(
    connection: &Connection,
    id: Uuid,
    values: &[String],
    delete_sql: &str,
    insert_sql: &str,
    label: &str,
) -> Result<(), String> {
    let id_text = id.to_string();
    connection
        .execute(delete_sql, &[&id_text])
        .map_err(|error| format!("clear Oracle catalog {label}: {error}"))?;
    for (index, value) in values.iter().enumerate() {
        let sort_order = index as i32;
        connection
            .execute(insert_sql, &[&id_text, value, &sort_order])
            .map_err(|error| format!("insert Oracle catalog {label}: {error}"))?;
    }
    Ok(())
}

fn insert_edit_event(connection: &Connection, event: &AutographEditEvent) -> Result<(), String> {
    let id_text = event.id.to_string();
    let item_id_text = event.item_id.to_string();
    let event_type = event.kind.as_str();
    let field_diffs_json = serde_json::to_string(&event.field_diffs)
        .map_err(|error| format!("serialize Oracle catalog edit event field diffs: {error}"))?;
    let created_at_epoch_seconds = event.created_at_epoch_seconds;
    connection
        .execute(
            "insert into autograph_edit_events (
                id, item_id, event_type, summary, field_diffs_json, created_at
            ) values (
                :1, :2, :3, :4, :5, 
                timestamp '1970-01-01 00:00:00' + numtodsinterval(:6, 'SECOND')
            )",
            &[
                &id_text,
                &item_id_text,
                &event_type,
                &event.summary,
                &field_diffs_json,
                &created_at_epoch_seconds,
            ],
        )
        .map_err(|error| format!("insert Oracle catalog edit event: {error}"))?;
    Ok(())
}

fn insert_publish_job_events(
    connection: &Connection,
    publish_job_id: &str,
    included_event_ids: &BTreeSet<Uuid>,
) -> Result<(), String> {
    for event_id in included_event_ids {
        let edit_event_id = event_id.to_string();
        connection
            .execute(
                "insert into autograph_publish_job_events (
                    publish_job_id, edit_event_id
                ) values (
                    :1, :2
                )",
                &[&publish_job_id, &edit_event_id],
            )
            .map_err(|error| format!("insert Oracle publish job event snapshot: {error}"))?;
    }
    Ok(())
}

fn insert_cleanup_event(connection: &Connection, event: &ImageCleanupEvent) -> Result<(), String> {
    let id_text = event.id.to_string();
    let item_id_text = event.item_id.to_string();
    let image_id_text = event.image_id.to_string();
    let status = event.status.as_str();
    let created_at_epoch_seconds = event.created_at_epoch_seconds;
    connection
        .execute(
            "insert into autograph_cleanup_events (
                id, item_id, image_id, target_object_key, operation, status, admin_message, created_at
            ) values (
                :1, :2, :3, :4, :5, :6, :7,
                timestamp '1970-01-01 00:00:00' + numtodsinterval(:8, 'SECOND')
            )",
            &[
                &id_text,
                &item_id_text,
                &image_id_text,
                &event.target_object_key,
                &event.operation,
                &status,
                &event.admin_message,
                &created_at_epoch_seconds,
            ],
        )
        .map_err(|error| format!("insert Oracle cleanup event: {error}"))?;
    Ok(())
}

fn publication_status_text(status: PublicationStatus) -> &'static str {
    match status {
        PublicationStatus::Draft => "draft",
        PublicationStatus::Published => "published",
        PublicationStatus::Archived => "archived",
    }
}

fn parse_publication_status(status: &str) -> Result<PublicationStatus, String> {
    match status {
        "draft" => Ok(PublicationStatus::Draft),
        "published" => Ok(PublicationStatus::Published),
        "archived" => Ok(PublicationStatus::Archived),
        _ => Err(format!("unsupported Oracle publication status: {status}")),
    }
}

fn item_origin_text(origin: ItemOrigin) -> &'static str {
    match origin {
        ItemOrigin::Official => "Official",
        ItemOrigin::Custom => "Custom",
    }
}

fn parse_item_origin(origin: &str) -> Result<ItemOrigin, String> {
    match origin {
        "Official" => Ok(ItemOrigin::Official),
        "Custom" => Ok(ItemOrigin::Custom),
        _ => Err(format!("unsupported Oracle item origin: {origin}")),
    }
}

fn parse_uuid(value: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|error| format!("parse Oracle UUID: {error}"))
}

fn row_value<T: FromDbValue>(row: &Row, index: usize, name: &str) -> Result<T, String> {
    row.get(index)
        .map_err(|error| format!("read Oracle catalog {name}: {error}"))
}

fn query_row_value<T: FromDbValue>(
    connection: &Connection,
    sql: &str,
    params: &[&dyn ToDbValue],
    name: &str,
) -> Result<T, String> {
    let row = connection
        .query_row(sql, params)
        .map_err(|error| format!("read Oracle catalog {name}: {error}"))?;
    row_value(&row, 0, name)
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
        let value = value.trim().to_owned();
        if value.is_empty() { None } else { Some(value) }
    })
}

fn normalize_string_list(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .filter_map(|value| {
            let value = value.trim().to_owned();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_phase7_helper_functions_are_available() {
        let _: fn(&Connection, Uuid) -> Result<Vec<SignerCredit>, String> = load_signer_credits;
        let _: fn(&Connection, Uuid, &[SignerCredit]) -> Result<(), String> =
            replace_signer_credits;
        let _: fn(&Connection, Uuid, &[SignerCredit]) -> Result<(), String> = sync_signer_credits;
        let _: fn(&Connection, &SignerCredit) -> Result<SignerProfile, String> =
            upsert_signer_profile;
        let _: fn(&Connection, Uuid) -> Result<Vec<SignerCreditRow>, String> =
            load_signer_credit_rows_for_update;
        let _: fn(&Connection, Uuid) -> Result<Vec<String>, String> = load_characters;
        let _: fn(&Connection, Uuid, &[String]) -> Result<(), String> = replace_characters;
        let _: fn(&Connection, Uuid) -> Result<Vec<String>, String> = load_franchises;
        let _: fn(&Connection, Uuid, &[String]) -> Result<(), String> = replace_franchises;
    }

    #[test]
    fn oracle_load_item_selects_phase7_taxonomy_fields() {
        for required_fragment in ["format", "origin", "language", "product_line", "set_name"] {
            assert!(
                LOAD_ITEM_SQL.contains(required_fragment),
                "Oracle load item SQL missing `{required_fragment}`"
            );
        }
    }

    #[test]
    fn oracle_bulk_list_sql_filters_items_by_publication_status() {
        let published_sql = list_items_sql(Some(PublicationStatus::Published));
        assert!(published_sql.contains("from autograph_items"));
        assert!(published_sql.contains("where publication_status = 'published'"));
        assert!(published_sql.contains("order by title, id"));
        assert!(!published_sql.contains("publication_status = 'draft'"));
        assert!(!published_sql.contains("publication_status = 'archived'"));

        let all_sql = list_items_sql(None);
        assert!(all_sql.contains("from autograph_items"));
        assert!(!all_sql.contains("where publication_status"));
    }

    #[test]
    fn oracle_bulk_child_filters_match_item_publication_status() {
        assert_eq!(
            child_status_filter(Some(PublicationStatus::Published)),
            " where i.publication_status = 'published'"
        );
        assert_eq!(
            child_status_filter(Some(PublicationStatus::Draft)),
            " where i.publication_status = 'draft'"
        );
        assert_eq!(
            child_status_filter(Some(PublicationStatus::Archived)),
            " where i.publication_status = 'archived'"
        );
        assert_eq!(child_status_filter(None), "");
    }

    #[test]
    fn oracle_image_loaders_select_cache_source_metadata() {
        let image_sql = load_images_for_items_sql(Some(PublicationStatus::Published));
        for required_fragment in ["img.byte_size", "img.checksum", "img.etag"] {
            assert!(
                image_sql.contains(required_fragment),
                "Oracle bulk image SQL missing `{required_fragment}`"
            );
        }
        assert!(image_sql.contains("where i.publication_status = 'published'"));

        for required_fragment in ["byte_size", "checksum", "etag"] {
            assert!(
                IMAGE_SELECT_COLUMNS.contains(required_fragment),
                "Oracle single-item image SQL missing `{required_fragment}`"
            );
        }
    }

    #[test]
    fn oracle_profile_links_normalize_to_compact_ids() {
        assert_eq!(normalize_profile_link(None, "wikipediaUrl").unwrap(), None);
        assert_eq!(
            normalize_profile_link(Some("".to_owned()), "imdbUrl").unwrap(),
            None
        );
        assert_eq!(
            normalize_profile_link(Some("https://w.wiki/Hamill1".to_owned()), "wikipediaUrl")
                .unwrap()
                .as_deref(),
            Some("Hamill1")
        );
        assert_eq!(
            normalize_profile_link(
                Some("https://www.imdb.com/name/nm0000434/".to_owned()),
                "imdbUrl"
            )
            .unwrap()
            .as_deref(),
            Some("nm0000434")
        );

        assert_eq!(
            normalize_profile_link(Some("javascript:alert(1)".to_owned()), "wikipediaUrl")
                .unwrap_err(),
            "wikipediaUrl must be a w.wiki short ID or https://w.wiki/ URL"
        );
        assert_eq!(
            normalize_profile_link(
                Some("https://example.test/wiki/Mark_Hamill".to_owned()),
                "wikipediaUrl",
            )
            .unwrap_err(),
            "wikipediaUrl must be a w.wiki short ID or https://w.wiki/ URL"
        );
        assert_eq!(
            normalize_profile_link(
                Some("https://wikipedia.org.example.test/name".to_owned()),
                "wikipediaUrl",
            )
            .unwrap_err(),
            "wikipediaUrl must be a w.wiki short ID or https://w.wiki/ URL"
        );
        assert_eq!(
            normalize_profile_link(
                Some("https://example.test/name/nm0000434/".to_owned()),
                "imdbUrl",
            )
            .unwrap_err(),
            "imdbUrl must be an IMDb name ID or https://www.imdb.com/name/ URL"
        );
    }

    #[test]
    fn oracle_skips_unchanged_submitted_signer_credit_writeback() {
        assert!(!should_sync_signer_credits(false, false, false));
        assert!(!should_sync_signer_credits(false, true, true));
        assert!(!should_sync_signer_credits(true, true, false));
        assert!(should_sync_signer_credits(true, true, true));
        assert!(should_sync_signer_credits(true, false, false));
    }

    #[test]
    fn oracle_signer_credit_sync_uses_stable_row_order() {
        assert!(SIGNER_CREDIT_ROWS_FOR_UPDATE_SQL.contains("order by signer_id"));
        assert!(SIGNER_CREDIT_ROWS_FOR_UPDATE_SQL.contains("for update"));

        let first_id = Uuid::parse_str("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb").unwrap();
        let second_id = Uuid::parse_str("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa").unwrap();
        let rows = desired_signer_credit_rows(&[
            signer_credit(first_id, Some("actor"), None),
            signer_credit(second_id, None, Some("Princess Leia")),
        ]);

        assert_eq!(
            rows,
            vec![
                SignerCreditRow {
                    signer_id: second_id.to_string(),
                    sort_order: 1,
                    item_role: None,
                    item_context: Some("Princess Leia".to_owned()),
                },
                SignerCreditRow {
                    signer_id: first_id.to_string(),
                    sort_order: 0,
                    item_role: Some("actor".to_owned()),
                    item_context: None,
                },
            ]
        );
    }

    fn signer_credit(
        signer_id: Uuid,
        item_role: Option<&str>,
        item_context: Option<&str>,
    ) -> SignerCredit {
        SignerCredit {
            signer: SignerProfile {
                id: signer_id,
                display_name: signer_id.to_string(),
                normalized_name: signer_id.to_string(),
                default_role: None,
                wikipedia_url: None,
                imdb_url: None,
                created_at_epoch_seconds: 0,
                updated_at_epoch_seconds: 0,
            },
            sort_order: 0,
            item_role: item_role.map(str::to_owned),
            item_context: item_context.map(str::to_owned),
        }
    }

    #[test]
    fn oracle_pending_queries_use_snapshot_membership_before_timestamp_fallback() {
        for sql in [GLOBAL_PENDING_CHANGES_SQL, ITEM_PENDING_CHANGES_SQL] {
            assert!(sql.contains("from autograph_publish_jobs"));
            assert!(sql.contains("where status = 'succeeded'"));
            assert!(sql.contains("order by started_at desc, created_at desc, id desc"));
            assert!(sql.contains("snapshot_event_count"));
            assert!(sql.contains("left join autograph_publish_job_events pe"));
            assert!(sql.contains("pe.publish_job_id = p.id"));
            assert!(sql.contains("pe.edit_event_id = e.id"));
            assert!(
                sql.contains("p.snapshot_event_count is not null and pe.edit_event_id is null")
            );
            assert!(
                sql.contains("p.snapshot_event_count is null and e.created_at >= p.started_at")
            );
            assert!(!sql.contains("pe.edit_event_id is null and e.created_at >= p.started_at"));
        }
    }

    #[test]
    fn oracle_item_pending_query_uses_same_snapshot_exclusion_as_global_query() {
        assert!(ITEM_PENDING_CHANGES_SQL.contains("where e.item_id = :1"));

        for required_fragment in [
            "left join autograph_publish_job_events pe",
            "pe.publish_job_id = p.id",
            "pe.edit_event_id = e.id",
            "p.snapshot_event_count is not null and pe.edit_event_id is null",
            "p.snapshot_event_count is null and e.created_at >= p.started_at",
        ] {
            assert!(
                ITEM_PENDING_CHANGES_SQL.contains(required_fragment),
                "item pending query missing `{required_fragment}`"
            );
            assert!(
                GLOBAL_PENDING_CHANGES_SQL.contains(required_fragment),
                "global pending query missing `{required_fragment}`"
            );
        }
    }
}
