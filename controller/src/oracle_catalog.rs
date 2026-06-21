use async_trait::async_trait;
use oracle::{Connection, Row};
use tokio::task;
use uuid::Uuid;

use crate::catalog::{
    AutographEditEvent, AutographImage, AutographItem, AutographItemInput, AutographItemUpdate,
    CatalogRepository, EditEventKind, FieldDiff, PendingChangeSummary, PublicationStatus,
    apply_update, event_kind_for_diffs, event_summary, now_epoch_seconds, validate_required_fields,
};

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
            let connection = Connection::connect(
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
            let id_text = id.to_string();
            let status = publication_status_text(input.publication_status);
            connection
                .execute(
                    "insert into autograph_items (
                        id, title, signer, description, category, object_reference,
                        event_name, event_location, source, inscription,
                        certification_company, certification_id, estimated_year,
                        publication_status
                    ) values (
                        :1, :2, :3, :4, :5, :6, :7, :8, :9, :10, :11, :12, :13, :14
                    )",
                    &[
                        &id_text,
                        &input.title,
                        &input.signer,
                        &input.description,
                        &input.category,
                        &input.object_reference,
                        &input.event_name,
                        &input.event_location,
                        &input.source,
                        &input.inscription,
                        &input.certification_company,
                        &input.certification_id,
                        &input.estimated_year,
                        &status,
                    ],
                )
                .map_err(|error| format!("insert Oracle catalog item: {error}"))?;
            replace_tags(&connection, id, &input.tags)?;
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
            let field_diffs = apply_update(&mut item, input);
            validate_required_fields(&item.title, &item.signer, &item.category)?;
            if field_diffs.is_empty() {
                return Ok(item);
            }
            let id_text = id.to_string();
            let status = publication_status_text(item.publication_status);
            let statement = connection
                .execute(
                    "update autograph_items set
                        title = :1, signer = :2, description = :3, category = :4,
                        object_reference = :5, event_name = :6, event_location = :7,
                        source = :8, inscription = :9, certification_company = :10,
                        certification_id = :11, estimated_year = :12,
                        publication_status = :13, updated_at = current_timestamp
                    where id = :14",
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
                        &status,
                        &id_text,
                    ],
                )
                .map_err(|error| format!("update Oracle catalog item: {error}"))?;
            let rows_updated = statement
                .row_count()
                .map_err(|error| format!("read Oracle catalog update row count: {error}"))?;
            if rows_updated == 0 {
                return Err("autograph item was not found".to_owned());
            }
            replace_tags(&connection, id, &item.tags)?;
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
        self.with_connection(move |connection| {
            let mut rows = connection
                .query("select id from autograph_items order by title, id", &[])
                .map_err(|error| format!("list Oracle catalog item ids: {error}"))?;
            let mut ids = Vec::new();
            for row in &mut rows {
                ids.push(parse_uuid(
                    &row.map_err(|error| format!("read Oracle catalog item id row: {error}"))?
                        .get::<_, String>(0)
                        .map_err(|error| format!("read Oracle catalog item id: {error}"))?,
                )?);
            }
            ids.into_iter()
                .map(|id| {
                    load_item(&connection, id)?
                        .ok_or_else(|| "listed Oracle item was not found".to_owned())
                })
                .collect()
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
                        original_filename, content_type, byte_size, is_primary,
                        sort_order, alt_text
                    ) values (:1, :2, :3, :4, :5, :6, :7, :8, :9, :10, :11)",
                    &[
                        &image_id,
                        &item_id_text,
                        &storage_namespace,
                        &bucket_name,
                        &image.object_key,
                        &image.original_filename,
                        &image.content_type,
                        &byte_size,
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

    async fn history(&self, item_id: Uuid) -> Result<Vec<AutographEditEvent>, String> {
        self.with_connection(move |connection| load_history(&connection, item_id))
            .await
    }

    async fn pending_changes(&self) -> Result<PendingChangeSummary, String> {
        self.with_connection(move |connection| pending_changes(&connection))
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
}

fn load_item(connection: &Connection, id: Uuid) -> Result<Option<AutographItem>, String> {
    let id_text = id.to_string();
    let mut rows = connection
        .query(
            "select
                title, signer, description, category, object_reference,
                event_name, event_location, source, inscription,
                certification_company, certification_id, estimated_year,
                publication_status,
                cast(round((cast(created_at as date) - date '1970-01-01') * 86400) as number(19)),
                cast(round((cast(updated_at as date) - date '1970-01-01') * 86400) as number(19))
            from autograph_items where id = :1",
            &[&id_text],
        )
        .map_err(|error| format!("read Oracle catalog item: {error}"))?;
    let Some(row) = rows.next() else {
        return Ok(None);
    };
    let row = row.map_err(|error| format!("read Oracle catalog item row: {error}"))?;
    let mut item = item_from_row(id, &row)?;
    item.tags = load_tags(connection, id)?;
    item.images = load_images(connection, id)?;
    Ok(Some(item))
}

fn item_from_row(id: Uuid, row: &Row) -> Result<AutographItem, String> {
    Ok(AutographItem {
        id,
        title: row_value(row, 0, "title")?,
        signer: row_value(row, 1, "signer")?,
        description: row_value(row, 2, "description")?,
        category: row_value(row, 3, "category")?,
        object_reference: row_value(row, 4, "object reference")?,
        event_name: row_value(row, 5, "event name")?,
        event_location: row_value(row, 6, "event location")?,
        source: row_value(row, 7, "source")?,
        inscription: row_value(row, 8, "inscription")?,
        certification_company: row_value(row, 9, "certification company")?,
        certification_id: row_value(row, 10, "certification id")?,
        estimated_year: row_value(row, 11, "estimated year")?,
        publication_status: parse_publication_status(&row_value::<String>(
            row,
            12,
            "publication status",
        )?)?,
        tags: Vec::new(),
        images: Vec::new(),
        created_at_epoch_seconds: row_value::<Option<i64>>(row, 13, "created at")?
            .unwrap_or_default(),
        updated_at_epoch_seconds: row_value::<Option<i64>>(row, 14, "updated at")?
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

fn load_images(connection: &Connection, id: Uuid) -> Result<Vec<AutographImage>, String> {
    let id_text = id.to_string();
    let mut rows = connection
        .query(
            "select
                id, object_key, original_filename, content_type, byte_size,
                is_primary, sort_order, alt_text
            from autograph_images where item_id = :1 order by sort_order, id",
            &[&id_text],
        )
        .map_err(|error| format!("read Oracle catalog images: {error}"))?;
    let mut images = Vec::new();
    for row in &mut rows {
        let row = row.map_err(|error| format!("read Oracle catalog image row: {error}"))?;
        images.push(AutographImage {
            id: parse_uuid(&row_value::<String>(&row, 0, "image id")?)?,
            object_key: row_value(&row, 1, "image object key")?,
            original_filename: row_value::<Option<String>>(&row, 2, "image original filename")?
                .unwrap_or_else(|| "upload".to_owned()),
            content_type: row_value(&row, 3, "image content type")?,
            byte_size: row_value::<Option<i64>>(&row, 4, "image byte size")?.unwrap_or(0) as usize,
            is_primary: row_value::<String>(&row, 5, "image primary flag")? == "Y",
            sort_order: row_value(&row, 6, "image sort order")?,
            alt_text: row_value(&row, 7, "image alt text")?,
        });
    }
    Ok(images)
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

fn event_from_row(item_id: Uuid, row: &Row) -> Result<AutographEditEvent, String> {
    let field_diffs_json = row_value::<Option<String>>(row, 3, "edit event field diffs")?
        .unwrap_or_else(|| "[]".to_owned());
    let field_diffs = serde_json::from_str::<Vec<FieldDiff>>(&field_diffs_json)
        .map_err(|error| format!("parse Oracle catalog edit event field diffs: {error}"))?;
    Ok(AutographEditEvent {
        id: parse_uuid(&row_value::<String>(row, 0, "edit event id")?)?,
        item_id,
        kind: EditEventKind::from_str(&row_value::<String>(row, 1, "edit event type")?)?,
        summary: row_value(row, 2, "edit event summary")?,
        field_diffs,
        created_at_epoch_seconds: row_value::<Option<i64>>(row, 4, "edit event created at")?
            .unwrap_or_default(),
    })
}

fn pending_changes(connection: &Connection) -> Result<PendingChangeSummary, String> {
    let mut rows = connection
        .query(
            "select
                count(*),
                min(cast(round((cast(created_at as date) - date '1970-01-01') * 86400) as number(19)))
            from autograph_edit_events
            where created_at > coalesce(
                (select max(finished_at) from autograph_publish_jobs where status = 'succeeded'),
                timestamp '1970-01-01 00:00:00'
            )",
            &[],
        )
        .map_err(|error| format!("read Oracle catalog pending changes: {error}"))?;
    let Some(row) = rows.next() else {
        return Ok(PendingChangeSummary::default());
    };
    let row = row.map_err(|error| format!("read Oracle catalog pending changes row: {error}"))?;
    let count: i64 = row
        .get(0)
        .map_err(|error| format!("read Oracle catalog pending change count: {error}"))?;
    let oldest_changed_at_epoch_seconds: Option<i64> = row
        .get(1)
        .map_err(|error| format!("read Oracle catalog oldest pending change: {error}"))?;
    Ok(PendingChangeSummary {
        count: count.max(0) as usize,
        oldest_changed_at_epoch_seconds,
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

fn insert_edit_event(connection: &Connection, event: &AutographEditEvent) -> Result<(), String> {
    let id_text = event.id.to_string();
    let item_id_text = event.item_id.to_string();
    let event_type = event.kind.as_str();
    let field_diffs_json = serde_json::to_string(&event.field_diffs)
        .map_err(|error| format!("serialize Oracle catalog edit event field diffs: {error}"))?;
    connection
        .execute(
            "insert into autograph_edit_events (
                id, item_id, event_type, summary, field_diffs_json, created_at
            ) values (:1, :2, :3, :4, :5, current_timestamp)",
            &[
                &id_text,
                &item_id_text,
                &event_type,
                &event.summary,
                &field_diffs_json,
            ],
        )
        .map_err(|error| format!("insert Oracle catalog edit event: {error}"))?;
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

fn parse_uuid(value: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|error| format!("parse Oracle UUID: {error}"))
}

fn row_value<T: oracle::sql_type::FromSql>(
    row: &Row,
    index: usize,
    name: &str,
) -> Result<T, String> {
    row.get(index)
        .map_err(|error| format!("read Oracle catalog {name}: {error}"))
}
