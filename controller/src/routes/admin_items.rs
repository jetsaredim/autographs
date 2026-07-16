use std::time::Instant;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    catalog::{
        AutographEditEvent, AutographItem, FieldDiff, FieldPatch, ItemOrigin, PendingChangeSummary,
        SignerMergeResult, SignerProfile, SignerProfileUpdateInput, SignerSuggestion,
        TaxonomySuggestions,
    },
    catalog_admin::{AdminCatalogRepositoryExt, AdminItemFilter},
};

use super::{AppState, authorize_admin_session, item_response_with_state};

pub(super) async fn list_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(filter): Query<AdminItemFilter>,
) -> Response {
    if let Err(status) = authorize_admin_session(&state, &Method::GET, &headers) {
        tracing::warn!(status = %status, "rejected admin item list request");
        return status.into_response();
    }

    let started = Instant::now();
    let changes_filter = filter.changes.clone();
    match state.repository.as_ref().list_admin_items(filter).await {
        Ok(items) => {
            let loaded_count = items.len();
            let mut summaries = Vec::with_capacity(items.len());
            // Phase 06-02 intentionally derives per-item pending markers from existing
            // item history. This keeps the API simple for the current small admin
            // catalog; a future publish-boundary store can replace this with a bulk
            // repository query when the catalog size makes the N+1 lookup material.
            for item in items {
                let pending = pending_marker(&state, item.id).await;
                if !changes_filter_matches(pending.has_pending_changes, &changes_filter) {
                    continue;
                }
                summaries.push(AdminItemSummaryResponse::from_item(
                    item,
                    pending.has_pending_changes,
                ));
            }
            tracing::info!(
                loaded_count,
                returned_count = summaries.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "listed admin catalog items"
            );
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
    if let Err(status) = authorize_admin_session(&state, &Method::GET, &headers) {
        tracing::warn!(status = %status, "rejected admin item get request");
        return status.into_response();
    }
    let Ok(id) = Uuid::parse_str(&id) else {
        tracing::warn!("rejected admin item get request with malformed item id");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let started = Instant::now();
    match state.repository.get(id).await {
        Ok(Some(item)) => {
            tracing::info!(
                item_id = %id,
                image_count = item.images.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "loaded admin catalog item"
            );
            Json(item_response_with_state(&state, item).await).into_response()
        }
        Ok(None) => {
            tracing::warn!(item_id = %id, "admin catalog item not found");
            StatusCode::NOT_FOUND.into_response()
        }
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
    if let Err(status) = authorize_admin_session(&state, &Method::GET, &headers) {
        tracing::warn!(status = %status, "rejected item history request");
        return status.into_response();
    }
    let Ok(id) = Uuid::parse_str(&id) else {
        tracing::warn!("rejected item history request with malformed item id");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let started = Instant::now();
    match state.repository.get(id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            tracing::warn!(item_id = %id, "item history requested for missing item");
            return StatusCode::NOT_FOUND.into_response();
        }
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
        Ok(events) => {
            tracing::info!(
                item_id = %id,
                event_count = events.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "loaded admin catalog item history"
            );
            Json(ItemHistoryResponse {
                item_id: id,
                events: events.into_iter().map(EditEventResponse::from).collect(),
            })
            .into_response()
        }
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

pub(super) async fn list_signers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminSignerQuery>,
) -> Response {
    if let Err(status) = authorize_admin_session(&state, &Method::GET, &headers) {
        tracing::warn!(status = %status, "rejected signer suggestions request");
        return status.into_response();
    }

    let started = Instant::now();
    match state
        .repository
        .signer_suggestions(query.query.unwrap_or_default())
        .await
    {
        Ok(suggestions) => {
            tracing::info!(
                suggestion_count = suggestions.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "loaded signer suggestions"
            );
            Json(AdminSignerSuggestionsResponse {
                suggestions: suggestions
                    .into_iter()
                    .map(AdminSignerSuggestionResponse::from)
                    .collect(),
            })
            .into_response()
        }
        Err(error) => {
            tracing::error!(error = %error, "failed to load signer suggestions");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub(super) async fn update_signer(
    State(state): State<AppState>,
    Path(id): Path<String>,
    method: Method,
    headers: HeaderMap,
    Json(input): Json<AdminSignerUpdateRequest>,
) -> Response {
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected signer profile update request");
        return status.into_response();
    }
    let Ok(id) = Uuid::parse_str(&id) else {
        tracing::warn!("rejected signer profile update request with malformed signer id");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let started = Instant::now();
    match state
        .repository
        .update_signer_profile(id, input.into())
        .await
    {
        Ok(profile) => {
            tracing::info!(
                signer_id = %id,
                elapsed_ms = started.elapsed().as_millis(),
                "updated signer profile"
            );
            Json(AdminSignerProfileResponse::from(profile)).into_response()
        }
        Err(error) if error.contains("not found") => {
            tracing::warn!(signer_id = %id, "signer profile update requested for missing signer");
            StatusCode::NOT_FOUND.into_response()
        }
        Err(error) => {
            tracing::error!(signer_id = %id, error = %error, "failed to update signer profile");
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}

pub(super) async fn merge_signers(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    Json(input): Json<AdminSignerMergeRequest>,
) -> Response {
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected signer merge request");
        return status.into_response();
    }
    if input.source_signer_id == input.target_signer_id {
        tracing::warn!(
            source_signer_id = %input.source_signer_id,
            target_signer_id = %input.target_signer_id,
            "rejected signer merge request with identical source and target"
        );
        return StatusCode::BAD_REQUEST.into_response();
    }

    let started = Instant::now();
    match state
        .repository
        .merge_signer_profiles(input.source_signer_id, input.target_signer_id)
        .await
    {
        Ok(result) => {
            tracing::info!(
                source_signer_id = %result.source_signer_id,
                target_signer_id = %result.target_signer_id,
                affected_item_count = result.updated_item_count,
                elapsed_ms = started.elapsed().as_millis(),
                "merged signer profiles"
            );
            Json(AdminSignerMergeResponse::from(result)).into_response()
        }
        Err(error) if error.contains("not found") => {
            tracing::warn!(
                source_signer_id = %input.source_signer_id,
                target_signer_id = %input.target_signer_id,
                "signer merge requested for missing signer"
            );
            StatusCode::NOT_FOUND.into_response()
        }
        Err(error) => {
            tracing::error!(error = %error, "failed to merge signer profiles");
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}

pub(super) async fn taxonomy_suggestions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Err(status) = authorize_admin_session(&state, &Method::GET, &headers) {
        tracing::warn!(status = %status, "rejected taxonomy suggestions request");
        return status.into_response();
    }

    let started = Instant::now();
    match state.repository.taxonomy_suggestions().await {
        Ok(suggestions) => {
            tracing::info!(
                character_count = suggestions.characters.len(),
                franchise_count = suggestions.franchises.len(),
                product_line_count = suggestions.product_lines.len(),
                format_count = suggestions.formats.len(),
                tag_count = suggestions.tags.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "loaded taxonomy suggestions"
            );
            Json(AdminTaxonomySuggestionsResponse::from(suggestions)).into_response()
        }
        Err(error) => {
            tracing::error!(error = %error, "failed to load taxonomy suggestions");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub(super) async fn pending_marker(state: &AppState, item_id: Uuid) -> PendingMarkerResponse {
    match state.repository.pending_changes_for_item(item_id).await {
        Ok(summary) => PendingMarkerResponse::from_summary(summary),
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
    fn from_summary(summary: PendingChangeSummary) -> Self {
        Self {
            has_pending_changes: summary.count > 0,
            count: summary.count,
            oldest_changed_at_epoch_seconds: summary.oldest_changed_at_epoch_seconds,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AdminSignerQuery {
    query: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminSignerSuggestionsResponse {
    suggestions: Vec<AdminSignerSuggestionResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminSignerSuggestionResponse {
    profile: AdminSignerProfileResponse,
    possible_duplicate: bool,
}

impl From<SignerSuggestion> for AdminSignerSuggestionResponse {
    fn from(suggestion: SignerSuggestion) -> Self {
        Self {
            profile: AdminSignerProfileResponse::from(suggestion.profile),
            possible_duplicate: suggestion.possible_duplicate,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AdminSignerUpdateRequest {
    #[serde(default)]
    display_name: FieldPatch<String>,
    #[serde(default)]
    default_role: FieldPatch<String>,
    #[serde(default)]
    wikipedia_url: FieldPatch<String>,
    #[serde(default)]
    imdb_url: FieldPatch<String>,
}

impl From<AdminSignerUpdateRequest> for SignerProfileUpdateInput {
    fn from(request: AdminSignerUpdateRequest) -> Self {
        Self {
            display_name: request.display_name,
            default_role: request.default_role,
            wikipedia_url: request.wikipedia_url,
            imdb_url: request.imdb_url,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AdminSignerMergeRequest {
    source_signer_id: Uuid,
    target_signer_id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminSignerMergeResponse {
    source_signer_id: Uuid,
    target_signer_id: Uuid,
    updated_item_count: usize,
}

impl From<SignerMergeResult> for AdminSignerMergeResponse {
    fn from(result: SignerMergeResult) -> Self {
        Self {
            source_signer_id: result.source_signer_id,
            target_signer_id: result.target_signer_id,
            updated_item_count: result.updated_item_count,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AdminSignerProfileResponse {
    id: Uuid,
    display_name: String,
    normalized_name: String,
    default_role: Option<String>,
    wikipedia_url: Option<String>,
    imdb_url: Option<String>,
    created_at_epoch_seconds: i64,
    updated_at_epoch_seconds: i64,
}

impl From<SignerProfile> for AdminSignerProfileResponse {
    fn from(profile: SignerProfile) -> Self {
        Self {
            id: profile.id,
            display_name: profile.display_name,
            normalized_name: profile.normalized_name,
            default_role: profile.default_role,
            wikipedia_url: profile.wikipedia_url,
            imdb_url: profile.imdb_url,
            created_at_epoch_seconds: profile.created_at_epoch_seconds,
            updated_at_epoch_seconds: profile.updated_at_epoch_seconds,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminTaxonomySuggestionsResponse {
    signers: Vec<AdminSignerProfileResponse>,
    characters: Vec<String>,
    formats: Vec<String>,
    origins: Vec<ItemOrigin>,
    franchises: Vec<String>,
    product_lines: Vec<String>,
    set_names: Vec<String>,
    languages: Vec<String>,
    roles: Vec<String>,
    tags: Vec<String>,
}

impl From<TaxonomySuggestions> for AdminTaxonomySuggestionsResponse {
    fn from(suggestions: TaxonomySuggestions) -> Self {
        Self {
            signers: suggestions
                .signers
                .into_iter()
                .map(AdminSignerProfileResponse::from)
                .collect(),
            characters: suggestions.characters,
            formats: suggestions.formats,
            origins: suggestions.origins,
            franchises: suggestions.franchises,
            product_lines: suggestions.product_lines,
            set_names: suggestions.set_names,
            languages: suggestions.languages,
            roles: suggestions.roles,
            tags: suggestions.tags,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminItemSummaryResponse {
    id: Uuid,
    title: String,
    signer_text: String,
    signer_names: Vec<String>,
    signer_ids: Vec<Uuid>,
    format: String,
    franchises: Vec<String>,
    product_line: Option<String>,
    language: String,
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
            signer_text: item.signer,
            signer_names: item
                .signer_credits
                .iter()
                .map(|credit| credit.signer.display_name.clone())
                .collect(),
            signer_ids: item
                .signer_credits
                .iter()
                .map(|credit| credit.signer.id)
                .collect(),
            format: item.format,
            franchises: item.franchises,
            product_line: item.product_line,
            language: item.language,
            tags: item.tags,
            publication_status: item.publication_status,
            image_count: item.images.len(),
            has_pending_changes,
            updated_at_epoch_seconds: item.updated_at_epoch_seconds,
        }
    }
}

fn changes_filter_matches(has_pending_changes: bool, query: &Option<String>) -> bool {
    let Some(query) = query
        .as_deref()
        .map(str::trim)
        .filter(|query| !query.is_empty())
    else {
        return true;
    };
    match query.to_lowercase().as_str() {
        "pending" | "changed" | "true" | "yes" | "1" => has_pending_changes,
        "none" | "clean" | "false" | "no" | "0" => !has_pending_changes,
        _ => true,
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
