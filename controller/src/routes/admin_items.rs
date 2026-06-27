use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    catalog::{AutographEditEvent, AutographItem, FieldDiff},
    catalog_admin::{AdminCatalogRepositoryExt, AdminItemFilter},
};

use super::{AppState, authenticate, item_response_with_state};

pub(super) async fn list_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(filter): Query<AdminItemFilter>,
) -> Response {
    if authenticate(&state, &headers).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match state.repository.as_ref().list_admin_items(filter).await {
        Ok(items) => {
            let mut summaries = Vec::with_capacity(items.len());
            // Phase 06-02 intentionally derives per-item pending markers from existing
            // item history. This keeps the API simple for the current small admin
            // catalog; a future publish-boundary store can replace this with a bulk
            // repository query when the catalog size makes the N+1 lookup material.
            for item in items {
                let has_pending_changes = pending_marker(&state, item.id).await.has_pending_changes;
                summaries.push(AdminItemSummaryResponse::from_item(
                    item,
                    has_pending_changes,
                ));
            }
            Json(summaries).into_response()
        }
        Err(error) => {
            tracing::error!(error = %error, "failed to list admin catalog items");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub(super) async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if authenticate(&state, &headers).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Ok(id) = Uuid::parse_str(&id) else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    match state.repository.get(id).await {
        Ok(Some(item)) => Json(item_response_with_state(&state, item).await).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(item_id = %id, error = %error, "failed to get admin catalog item");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub(super) async fn item_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if authenticate(&state, &headers).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Ok(id) = Uuid::parse_str(&id) else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    match state.repository.get(id).await {
        Ok(Some(_)) => {}
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(
                item_id = %id,
                error = %error,
                "failed to check item before history lookup"
            );
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    match state.repository.history(id).await {
        Ok(events) => Json(ItemHistoryResponse {
            item_id: id,
            events: events.into_iter().map(EditEventResponse::from).collect(),
        })
        .into_response(),
        Err(error) => {
            tracing::error!(
                item_id = %id,
                error = %error,
                "failed to load admin catalog item history"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// Until publish-boundary persistence exists, this marker means "this item has
// recorded admin edit history" rather than "this item differs from the last
// completed static release."
pub(super) async fn pending_marker(state: &AppState, item_id: Uuid) -> PendingMarkerResponse {
    match state.repository.history(item_id).await {
        Ok(events) => PendingMarkerResponse::from_events(&events),
        Err(error) => {
            tracing::warn!(
                item_id = %item_id,
                error = %error,
                "failed to load pending marker history"
            );
            PendingMarkerResponse::default()
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PendingMarkerResponse {
    pub has_pending_changes: bool,
    pub count: usize,
    pub oldest_changed_at_epoch_seconds: Option<i64>,
}

impl PendingMarkerResponse {
    fn from_events(events: &[AutographEditEvent]) -> Self {
        Self {
            has_pending_changes: !events.is_empty(),
            count: events.len(),
            oldest_changed_at_epoch_seconds: events
                .iter()
                .map(|event| event.created_at_epoch_seconds)
                .min(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminItemSummaryResponse {
    id: Uuid,
    title: String,
    signer: String,
    category: String,
    tags: Vec<String>,
    publication_status: crate::catalog::PublicationStatus,
    image_count: usize,
    has_pending_changes: bool,
    updated_at_epoch_seconds: i64,
}

impl AdminItemSummaryResponse {
    fn from_item(item: AutographItem, has_pending_changes: bool) -> Self {
        Self {
            id: item.id,
            title: item.title,
            signer: item.signer,
            category: item.category,
            tags: item.tags,
            publication_status: item.publication_status,
            image_count: item.images.len(),
            has_pending_changes,
            updated_at_epoch_seconds: item.updated_at_epoch_seconds,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemHistoryResponse {
    item_id: Uuid,
    events: Vec<EditEventResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EditEventResponse {
    id: Uuid,
    event_type: String,
    created_at_epoch_seconds: i64,
    summary: String,
    field_diffs: Vec<FieldDiffResponse>,
}

impl From<AutographEditEvent> for EditEventResponse {
    fn from(event: AutographEditEvent) -> Self {
        Self {
            id: event.id,
            event_type: event.kind.as_str().to_owned(),
            created_at_epoch_seconds: event.created_at_epoch_seconds,
            summary: event.summary,
            field_diffs: event
                .field_diffs
                .into_iter()
                .map(FieldDiffResponse::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FieldDiffResponse {
    field: String,
    before: Value,
    after: Value,
}

impl From<FieldDiff> for FieldDiffResponse {
    fn from(diff: FieldDiff) -> Self {
        Self {
            field: diff.field,
            before: diff.before,
            after: diff.after,
        }
    }
}
