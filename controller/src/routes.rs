use std::{env, sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthState,
    catalog::{
        AutographImage, AutographItem, AutographItemInput, AutographItemUpdate, CatalogRepository,
        CleanupStatus, CleanupWarning, ImageCleanupEvent, ImageReplacementInput,
        MemoryCatalogRepository, PublicationStatus, REQUIRED_FIELDS_ERROR, now_epoch_seconds,
    },
    config::ControllerConfig,
    media::{LocalMediaStore, PrivateMediaStore},
    publisher::{LocalPublisher, PublishMode},
    storage_keys::build_original_object_key,
};

mod admin_items;

const SESSION_COOKIE: &str = "autographs_admin_session";
const MAX_IMAGE_UPLOAD_BYTES: usize = 20 * 1024 * 1024;

#[derive(Clone)]
pub struct AppState {
    config: ControllerConfig,
    auth: AuthState,
    repository: Arc<dyn CatalogRepository>,
    media: Arc<dyn PrivateMediaStore>,
    publisher: Arc<LocalPublisher>,
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    service: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminHealthResponse {
    ok: bool,
    service: &'static str,
    controller_db_provider: String,
    controller_media_storage_provider: String,
    oracle_configured: bool,
    media_configured: bool,
    static_release_configured: bool,
}

#[derive(Deserialize)]
struct LoginRequest {
    password: String,
}

pub fn router(config: ControllerConfig) -> Router {
    router_with_stores(
        config,
        Arc::new(MemoryCatalogRepository::default()),
        Arc::new(LocalMediaStore::new("/tmp/autographs-controller-media")),
    )
}

pub fn runtime_router(config: ControllerConfig) -> Result<Router, String> {
    config.validate_runtime_auth()?;
    let repository: Arc<dyn CatalogRepository> =
        match provider("AUTOGRAPHS_CONTROLLER_DB_PROVIDER").as_str() {
            "local" => {
                tracing::info!("configuring local in-memory catalog repository");
                Arc::new(MemoryCatalogRepository::default())
            }
            "oracle" => production_repository()?,
            provider => {
                return Err(format!(
                    "AUTOGRAPHS_CONTROLLER_DB_PROVIDER must be local or oracle, got {provider}"
                ));
            }
        };
    let media: Arc<dyn PrivateMediaStore> = match provider(
        "AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER",
    )
    .as_str()
    {
        "local" => {
            let root = env::var("AUTOGRAPHS_CONTROLLER_LOCAL_MEDIA_ROOT")
                .unwrap_or_else(|_| "/tmp/autographs-controller-media".to_owned());
            tracing::info!(%root, "configuring local media store");
            Arc::new(LocalMediaStore::new(root))
        }
        "oci-instance-principal" => production_media_store()?,
        provider => {
            return Err(format!(
                "AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER must be local or oci-instance-principal, got {provider}"
            ));
        }
    };
    Ok(router_with_stores(config, repository, media))
}

fn provider(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| "local".to_owned())
}

#[cfg(feature = "production-persistence")]
fn production_repository() -> Result<Arc<dyn CatalogRepository>, String> {
    use crate::{oracle_catalog::OracleCatalogRepository, oracle_schema};

    tracing::info!("configuring Oracle catalog repository");

    let oracle_user = required_env("ORACLE_DB_USER")?;
    let oracle_password = required_env("ORACLE_DB_PASSWORD")?;
    let oracle_connect_string = required_env("ORACLE_DB_CONNECT_STRING")?;

    oracle_schema::ensure_initialized(&oracle_user, &oracle_password, &oracle_connect_string)?;

    tracing::info!("Oracle catalog schema is ready");

    Ok(Arc::new(OracleCatalogRepository::new(
        oracle_user,
        oracle_password,
        oracle_connect_string,
        required_env("OCI_MEDIA_NAMESPACE")?,
        required_env("OCI_MEDIA_BUCKET_NAME")?,
    )))
}

#[cfg(not(feature = "production-persistence"))]
fn production_repository() -> Result<Arc<dyn CatalogRepository>, String> {
    Err("Oracle controller persistence requires the production-persistence feature".to_owned())
}

#[cfg(feature = "production-persistence")]
fn production_media_store() -> Result<Arc<dyn PrivateMediaStore>, String> {
    use crate::oci_media::OciInstancePrincipalMediaStore;
    tracing::info!("configuring OCI instance-principal media store");

    Ok(Arc::new(OciInstancePrincipalMediaStore::new(
        required_env("OCI_MEDIA_NAMESPACE")?,
        required_env("OCI_MEDIA_BUCKET_NAME")?,
    )?))
}

#[cfg(not(feature = "production-persistence"))]
fn production_media_store() -> Result<Arc<dyn PrivateMediaStore>, String> {
    Err(
        "OCI instance-principal controller persistence requires the production-persistence feature"
            .to_owned(),
    )
}

#[cfg(feature = "production-persistence")]
fn required_env(name: &str) -> Result<String, String> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{name} is required"))
}

pub fn router_with_stores(
    config: ControllerConfig,
    repository: Arc<dyn CatalogRepository>,
    media: Arc<dyn PrivateMediaStore>,
) -> Router {
    let static_release_root = config.static_release_root.clone();
    router_with_services(
        config,
        repository,
        media,
        Arc::new(LocalPublisher::new(static_release_root)),
    )
}

pub fn router_with_services(
    config: ControllerConfig,
    repository: Arc<dyn CatalogRepository>,
    media: Arc<dyn PrivateMediaStore>,
    publisher: Arc<LocalPublisher>,
) -> Router {
    let auth = AuthState::new(
        config.admin_password.clone(),
        config.admin_password_hash.clone(),
        config.operator_token.clone(),
    );
    let state = AppState {
        config,
        auth,
        repository,
        media,
        publisher,
    };

    Router::new()
        .route("/health", get(health))
        .route("/admin/api/health", get(admin_health))
        .route("/admin/api/login", post(login))
        .route("/admin/api/logout", post(logout))
        .route("/admin/api/protected", get(protected))
        .route("/admin/api/test-mutation", post(protected_mutation))
        .route(
            "/admin/api/items",
            get(admin_items::list_items).post(create_item),
        )
        .route(
            "/admin/api/items/{id}",
            get(admin_items::get_item).patch(update_item),
        )
        .route(
            "/admin/api/items/{id}/history",
            get(admin_items::item_history),
        )
        .route("/admin/api/items/{id}/images", post(upload_image))
        .route(
            "/admin/api/items/{id}/images/{image_id}/primary",
            post(set_primary_image),
        )
        .route(
            "/admin/api/items/{id}/images/{image_id}",
            delete(delete_image).put(replace_image),
        )
        .route(
            "/admin/api/items/{id}/images/{image_id}/cleanup/retry",
            post(retry_image_cleanup),
        )
        .route("/admin/api/items/{id}/publication", post(set_publication))
        .route("/admin/api/publish/incremental", post(publish_incremental))
        .route("/admin/api/publish/full", post(publish_full))
        .route("/admin/api/publish/status", get(publish_status))
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024))
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        service: "autographs-controller",
    })
}

async fn admin_health(State(state): State<AppState>) -> Json<AdminHealthResponse> {
    Json(AdminHealthResponse {
        ok: true,
        service: "autographs-controller",
        controller_db_provider: provider("AUTOGRAPHS_CONTROLLER_DB_PROVIDER"),
        controller_media_storage_provider: provider("AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER"),
        oracle_configured: state.config.oracle_configured,
        media_configured: state.config.media_configured,
        static_release_configured: state.config.static_release_configured,
    })
}

async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Response {
    match state.auth.login(&payload.password) {
        Ok(session) => {
            let mut response = StatusCode::NO_CONTENT.into_response();
            response.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&session_cookie(&session, state.config.secure_cookies))
                    .expect("session cookie header"),
            );
            response
        }
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn logout(State(state): State<AppState>, method: Method, headers: HeaderMap) -> Response {
    let auth = match authenticate(&state, &headers) {
        Some(auth) => auth,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    if !csrf_allowed(&state, &method, &headers, &auth) {
        return StatusCode::FORBIDDEN.into_response();
    }
    if let AuthKind::Session(session) = auth {
        state.auth.logout(&session);
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "autographs_admin_session=; Path=/admin; HttpOnly; SameSite=Strict; Max-Age=0",
        ),
    );
    response
}

async fn protected(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    authenticate(&state, &headers)
        .map(|_| StatusCode::OK)
        .unwrap_or(StatusCode::UNAUTHORIZED)
}

async fn protected_mutation(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
) -> StatusCode {
    let Some(auth) = authenticate(&state, &headers) else {
        return StatusCode::UNAUTHORIZED;
    };

    if csrf_allowed(&state, &method, &headers, &auth) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::FORBIDDEN
    }
}

async fn create_item(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    Json(input): Json<AutographItemInput>,
) -> Response {
    if let Err(status) = authorize_mutation(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected create catalog item request");
        return status.into_response();
    }

    let started = Instant::now();
    tracing::info!(
        title = %input.title,
        category = %input.category,
        "creating catalog item"
    );
    match state.repository.create(input).await {
        Ok(item) => {
            let item_id = item.id;
            tracing::info!(
                item_id = %item_id,
                status = ?item.publication_status,
                elapsed_ms = started.elapsed().as_millis(),
                "created catalog item"
            );
            let response = item_response_with_state(&state, item).await;
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(error) => {
            tracing::error!(error = %error, "failed to create catalog item");
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}

async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
    method: Method,
    headers: HeaderMap,
    Json(input): Json<AutographItemUpdate>,
) -> Response {
    if let Err(status) = authorize_mutation(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected update catalog item request");
        return status.into_response();
    }
    let Ok(id) = Uuid::parse_str(&id) else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let started = Instant::now();
    tracing::info!(item_id = %id, "updating catalog item");
    match state.repository.update(id, input).await {
        Ok(item) => {
            tracing::info!(
                item_id = %id,
                status = ?item.publication_status,
                elapsed_ms = started.elapsed().as_millis(),
                "updated catalog item"
            );
            Json(item_response_with_state(&state, item).await).into_response()
        }
        Err(error) => {
            tracing::error!(item_id = %id, error = %error, "failed to update catalog item");
            repository_update_error_status(&error).into_response()
        }
    }
}

async fn upload_image(
    State(state): State<AppState>,
    Path(id): Path<String>,
    method: Method,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    if let Err(status) = authorize_mutation(&state, &method, &headers) {
        tracing::warn!(status = %status, item_id = %id, "rejected upload image request");
        return status.into_response();
    }
    let Ok(item_id) = Uuid::parse_str(&id) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let existing_item = match state.repository.get(item_id).await {
        Ok(Some(item)) => item,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let mut filename = None;
    let mut content_type = None;
    let mut body = None;
    let mut alt_text = None;
    let mut requested_primary = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            filename = field.file_name().map(str::to_owned);
            content_type = field.content_type().map(str::to_owned);
            body = field.bytes().await.ok();
        } else if field.name() == Some("altText") {
            alt_text = field.text().await.ok();
        } else if field.name() == Some("isPrimary") {
            requested_primary = field
                .text()
                .await
                .ok()
                .and_then(|value| value.parse::<bool>().ok());
        }
    }
    let Some(body) = body else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_owned());
    if !matches!(
        content_type.as_str(),
        "image/jpeg" | "image/png" | "image/webp"
    ) || body.len() > MAX_IMAGE_UPLOAD_BYTES
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if !valid_image_upload(&content_type, &body) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item_id, image_id);
    let started = Instant::now();
    tracing::info!(
        %item_id,
        %image_id,
        %object_key,
        content_type = %content_type,
        byte_size = body.len(),
        "uploading catalog image"
    );
    if let Err(error) = state.media.write(&object_key, &body).await {
        tracing::error!(%item_id, %image_id, %object_key, %error, "failed to write uploaded image to private media store");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let image = AutographImage {
        id: image_id,
        object_key: object_key.clone(),
        original_filename: filename.unwrap_or_else(|| "upload".to_owned()),
        content_type,
        byte_size: body.len(),
        is_primary: existing_item.images.is_empty() || requested_primary.unwrap_or(false),
        sort_order: existing_item
            .images
            .iter()
            .map(|image| image.sort_order)
            .max()
            .unwrap_or(-1)
            + 1,
        alt_text,
    };
    match state.repository.attach_image(item_id, image).await {
        Ok(item) => {
            tracing::info!(
                %item_id,
                %image_id,
                %object_key,
                image_count = item.images.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "uploaded catalog image"
            );
            let response = item_response_with_state(&state, item).await;
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(error) => {
            tracing::error!(%item_id, %image_id, %object_key, ?error, "failed to attach uploaded image metadata");
            let _ = state.media.delete(&object_key).await;
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn set_primary_image(
    State(state): State<AppState>,
    Path((id, image_id)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if let Err(status) = authorize_mutation(&state, &method, &headers) {
        return status.into_response();
    }
    let (Ok(item_id), Ok(image_id)) = (Uuid::parse_str(&id), Uuid::parse_str(&image_id)) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match state.repository.set_primary_image(item_id, image_id).await {
        Ok(item) => Json(item_response_with_state(&state, item).await).into_response(),
        Err(error) => repository_update_error_status(&error).into_response(),
    }
}

async fn delete_image(
    State(state): State<AppState>,
    Path((id, image_id)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if let Err(status) = authorize_mutation(&state, &method, &headers) {
        return status.into_response();
    }
    let (Ok(item_id), Ok(image_id)) = (Uuid::parse_str(&id), Uuid::parse_str(&image_id)) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let item = match state.repository.get(item_id).await {
        Ok(Some(item)) => item,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%item_id, error = %error, "failed to load item before image delete");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let Some(image) = item
        .images
        .iter()
        .find(|image| image.id == image_id)
        .cloned()
    else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if let Err(error) = state.media.delete(&image.object_key).await {
        tracing::warn!(%item_id, %image_id, error = %error, "private image delete failed");
        return cleanup_warning_response(&state, item_id, image_id, &image.object_key, "delete")
            .await;
    }

    match state
        .repository
        .remove_image_metadata(item_id, image_id)
        .await
    {
        Ok(item) => Json(item_response_with_state(&state, item).await).into_response(),
        Err(error) => {
            tracing::error!(%item_id, %image_id, error = %error, "failed to remove image metadata after media delete");
            cleanup_warning_response(&state, item_id, image_id, &image.object_key, "delete").await
        }
    }
}

async fn replace_image(
    State(state): State<AppState>,
    Path((id, image_id)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    if let Err(status) = authorize_mutation(&state, &method, &headers) {
        return status.into_response();
    }
    let (Ok(item_id), Ok(image_id)) = (Uuid::parse_str(&id), Uuid::parse_str(&image_id)) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let item = match state.repository.get(item_id).await {
        Ok(Some(item)) => item,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%item_id, error = %error, "failed to load item before image replacement");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let Some(existing_image) = item
        .images
        .iter()
        .find(|image| image.id == image_id)
        .cloned()
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(upload) = parse_image_multipart(multipart).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if upload.body.len() > MAX_IMAGE_UPLOAD_BYTES
        || !valid_image_upload(&upload.content_type, &upload.body)
    {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let replacement_id = Uuid::new_v4();
    let replacement_key = build_original_object_key(item_id, replacement_id);
    if let Err(error) = state.media.write(&replacement_key, &upload.body).await {
        tracing::error!(%item_id, %replacement_id, error = %error, "failed to write replacement image");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let replacement = AutographImage {
        id: image_id,
        object_key: replacement_key.clone(),
        original_filename: upload.filename.unwrap_or_else(|| "upload".to_owned()),
        content_type: upload.content_type,
        byte_size: upload.body.len(),
        is_primary: existing_image.is_primary,
        sort_order: existing_image.sort_order,
        alt_text: upload.alt_text,
    };

    let item = match state
        .repository
        .replace_image_metadata(
            item_id,
            image_id,
            ImageReplacementInput { image: replacement },
        )
        .await
    {
        Ok(item) => item,
        Err(error) => {
            tracing::error!(%item_id, %image_id, error = %error, "failed to replace image metadata");
            let _ = state.media.delete(&replacement_key).await;
            return repository_update_error_status(&error).into_response();
        }
    };

    let warning = if let Err(error) = state.media.delete(&existing_image.object_key).await {
        tracing::warn!(%item_id, %image_id, error = %error, "old private image cleanup failed after replacement");
        match record_cleanup_warning(
            &state,
            item_id,
            image_id,
            &existing_image.object_key,
            "replace",
        )
        .await
        {
            Ok(warning) => Some(warning),
            Err(error) => {
                tracing::error!(%item_id, %image_id, error = %error, "failed to persist replacement cleanup warning");
                if let Err(rollback_error) = state
                    .repository
                    .replace_image_metadata(
                        item_id,
                        image_id,
                        ImageReplacementInput {
                            image: existing_image.clone(),
                        },
                    )
                    .await
                {
                    tracing::error!(%item_id, %image_id, error = %rollback_error, "failed to roll back replacement metadata after cleanup warning persistence failure");
                }
                if let Err(delete_error) = state.media.delete(&replacement_key).await {
                    tracing::warn!(%item_id, %image_id, error = %delete_error, "failed to delete replacement object after cleanup warning persistence failure");
                }
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    } else {
        None
    };
    let response = ItemResponseWithWarning {
        item: item_response_with_state(&state, item).await,
        cleanup_warning: warning.map(CleanupWarningResponse::from),
    };
    Json(response).into_response()
}

async fn retry_image_cleanup(
    State(state): State<AppState>,
    Path((id, image_id)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if let Err(status) = authorize_mutation(&state, &method, &headers) {
        return status.into_response();
    }
    let (Ok(item_id), Ok(image_id)) = (Uuid::parse_str(&id), Uuid::parse_str(&image_id)) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let item = match state.repository.get(item_id).await {
        Ok(item) => item,
        Err(error) => {
            tracing::error!(%item_id, error = %error, "failed to load item before cleanup retry");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let cleanup_warning = match state.repository.cleanup_warnings(item_id).await {
        Ok(warnings) => warnings
            .into_iter()
            .find(|warning| image_id == warning.image_id),
        Err(error) => {
            tracing::error!(%item_id, %image_id, error = %error, "failed to load cleanup warnings before cleanup retry");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let Some(cleanup_warning) = cleanup_warning else {
        tracing::warn!(%item_id, %image_id, "cleanup retry requested without unresolved cleanup warning");
        return StatusCode::CONFLICT.into_response();
    };
    if let Err(error) = state.media.delete(&cleanup_warning.target_object_key).await {
        tracing::warn!(%item_id, %image_id, error = %error, "private image cleanup retry failed");
        return cleanup_warning_response(
            &state,
            item_id,
            image_id,
            &cleanup_warning.target_object_key,
            &cleanup_warning.operation,
        )
        .await;
    }
    let removed = if cleanup_warning.operation != "replace"
        && item
            .as_ref()
            .is_some_and(|item| item.images.iter().any(|image| image.id == image_id))
    {
        match state
            .repository
            .remove_image_metadata(item_id, image_id)
            .await
        {
            Ok(item) => Some(item),
            Err(error) => {
                tracing::error!(%item_id, %image_id, error = %error, "failed to remove image metadata after cleanup retry");
                return repository_update_error_status(&error).into_response();
            }
        }
    } else {
        None
    };
    let retry_marked = match state
        .repository
        .mark_cleanup_retry_succeeded(item_id, image_id, &cleanup_warning.target_object_key)
        .await
    {
        Ok(updated) => updated,
        Err(error) => {
            tracing::error!(%item_id, %image_id, error = %error, "failed to mark cleanup retry succeeded");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !retry_marked {
        tracing::warn!(%item_id, %image_id, "cleanup retry succeeded but warning was already resolved");
        return StatusCode::CONFLICT.into_response();
    }
    if let Some(item) = removed {
        Json(item_response_with_state(&state, item).await).into_response()
    } else {
        StatusCode::OK.into_response()
    }
}

async fn record_cleanup_warning(
    state: &AppState,
    item_id: Uuid,
    image_id: Uuid,
    target_object_key: &str,
    operation: &str,
) -> Result<CleanupWarning, String> {
    let message = "Private image cleanup needs retry from the admin maintenance action.".to_owned();
    let event = ImageCleanupEvent::new(
        item_id,
        image_id,
        target_object_key,
        operation,
        CleanupStatus::DeleteFailed,
        message.clone(),
        now_epoch_seconds(),
    );
    state
        .repository
        .record_cleanup_event(event)
        .await
        .map_err(|error| format!("record cleanup warning: {error}"))?;
    Ok(CleanupWarning {
        image_id,
        target_object_key: target_object_key.to_owned(),
        operation: operation.to_owned(),
        status: CleanupStatus::DeleteFailed,
        admin_message: message,
    })
}

async fn cleanup_warning_response(
    state: &AppState,
    item_id: Uuid,
    image_id: Uuid,
    target_object_key: &str,
    operation: &str,
) -> Response {
    match record_cleanup_warning(state, item_id, image_id, target_object_key, operation).await {
        Ok(warning) => (
            StatusCode::CONFLICT,
            Json(CleanupWarningEnvelope::from(warning)),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%item_id, %image_id, operation, error = %error, "failed to persist cleanup warning");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub(super) async fn item_response_with_state(
    state: &AppState,
    item: AutographItem,
) -> ItemResponse {
    let item_id = item.id;
    let pending = admin_items::pending_marker(state, item_id).await;
    let cleanup_warnings = match state.repository.cleanup_warnings(item_id).await {
        Ok(warnings) => warnings,
        Err(error) => {
            tracing::warn!(%item_id, error = %error, "failed to load cleanup warnings");
            Vec::new()
        }
    };
    ItemResponse::from_item_with_state(item, pending, cleanup_warnings)
}

struct ParsedImageUpload {
    filename: Option<String>,
    content_type: String,
    body: Vec<u8>,
    alt_text: Option<String>,
}

async fn parse_image_multipart(mut multipart: Multipart) -> Option<ParsedImageUpload> {
    let mut filename = None;
    let mut content_type = None;
    let mut body = None;
    let mut alt_text = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            filename = field.file_name().map(str::to_owned);
            content_type = field.content_type().map(str::to_owned);
            body = field.bytes().await.ok().map(|bytes| bytes.to_vec());
        } else if field.name() == Some("altText") {
            alt_text = field.text().await.ok();
        }
    }
    Some(ParsedImageUpload {
        filename,
        content_type: content_type.unwrap_or_else(|| "application/octet-stream".to_owned()),
        body: body?,
        alt_text,
    })
}

fn valid_image_upload(content_type: &str, body: &[u8]) -> bool {
    let expected = match content_type {
        "image/jpeg" => ImageFormat::Jpeg,
        "image/png" => ImageFormat::Png,
        "image/webp" => ImageFormat::WebP,
        _ => return false,
    };
    image::guess_format(body).is_ok_and(|actual| actual == expected)
        && image::load_from_memory_with_format(body, expected).is_ok()
}

async fn set_publication(
    State(state): State<AppState>,
    Path(id): Path<String>,
    method: Method,
    headers: HeaderMap,
    Json(input): Json<PublicationRequest>,
) -> Response {
    if let Err(status) = authorize_mutation(&state, &method, &headers) {
        tracing::warn!(status = %status, item_id = %id, "rejected publication update request");
        return status.into_response();
    }
    let Ok(id) = Uuid::parse_str(&id) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let started = Instant::now();
    tracing::info!(
        item_id = %id,
        status = ?input.publication_status,
        "updating catalog item publication"
    );
    match state
        .repository
        .update(
            id,
            AutographItemUpdate {
                publication_status: Some(input.publication_status),
                ..Default::default()
            },
        )
        .await
    {
        Ok(item) => {
            tracing::info!(
                item_id = %id,
                status = ?item.publication_status,
                elapsed_ms = started.elapsed().as_millis(),
                "updated catalog item publication"
            );
            Json(item_response_with_state(&state, item).await).into_response()
        }
        Err(error) => {
            tracing::error!(
                item_id = %id,
                error = %error,
                "failed to update catalog item publication"
            );
            repository_update_error_status(&error).into_response()
        }
    }
}

async fn publish_incremental(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    publish(state, method, headers, PublishMode::Incremental).await
}

async fn publish_full(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    publish(state, method, headers, PublishMode::Full).await
}

async fn publish(
    state: AppState,
    method: Method,
    headers: HeaderMap,
    mode: PublishMode,
) -> Response {
    if let Err(status) = authorize_mutation(&state, &method, &headers) {
        tracing::warn!(status = %status, mode = ?mode, "rejected static publish request");
        return status.into_response();
    }

    let started = Instant::now();
    tracing::info!(mode = ?mode, "publishing static release");
    match state
        .publisher
        .publish(state.repository.as_ref(), state.media.as_ref(), mode)
        .await
    {
        Ok(status) => {
            tracing::info!(
                mode = ?mode,
                release_id = status.release_id.as_deref().unwrap_or("<none>"),
                artifact_count = status.artifact_count,
                byte_size = status.byte_size,
                elapsed_ms = started.elapsed().as_millis(),
                "published static release"
            );
            (StatusCode::CREATED, Json(status)).into_response()
        }
        Err(error) => {
            tracing::error!(error = %error, "failed to publish static release");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(state.publisher.status()),
            )
                .into_response()
        }
    }
}

async fn publish_status(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if authenticate(&state, &headers).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    Json(state.publisher.status()).into_response()
}

fn authorize_mutation(
    state: &AppState,
    method: &Method,
    headers: &HeaderMap,
) -> Result<AuthKind, StatusCode> {
    let auth = authenticate(state, headers).ok_or(StatusCode::UNAUTHORIZED)?;
    csrf_allowed(state, method, headers, &auth)
        .then_some(auth)
        .ok_or(StatusCode::FORBIDDEN)
}

fn repository_update_error_status(error: &str) -> StatusCode {
    if error == REQUIRED_FIELDS_ERROR {
        StatusCode::BAD_REQUEST
    } else if error.contains("not found") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublicationRequest {
    publication_status: PublicationStatus,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ItemResponse {
    id: Uuid,
    title: String,
    signer: String,
    description: Option<String>,
    category: String,
    tags: Vec<String>,
    object_reference: Option<String>,
    event_name: Option<String>,
    event_location: Option<String>,
    source: Option<String>,
    inscription: Option<String>,
    certification_company: Option<String>,
    certification_id: Option<String>,
    estimated_year: Option<i32>,
    publication_status: PublicationStatus,
    images: Vec<ImageResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pending_changes: Option<admin_items::PendingMarkerResponse>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cleanup_warnings: Vec<CleanupWarningResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageResponse {
    id: Uuid,
    content_type: String,
    byte_size: usize,
    is_primary: bool,
    sort_order: i32,
    alt_text: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemResponseWithWarning {
    #[serde(flatten)]
    item: ItemResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    cleanup_warning: Option<CleanupWarningResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupWarningResponse {
    image_id: Uuid,
    operation: String,
    status: CleanupStatus,
    admin_message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupWarningEnvelope {
    cleanup_warning: CleanupWarningResponse,
}

impl From<CleanupWarning> for CleanupWarningEnvelope {
    fn from(warning: CleanupWarning) -> Self {
        Self {
            cleanup_warning: CleanupWarningResponse::from(warning),
        }
    }
}

impl From<CleanupWarning> for CleanupWarningResponse {
    fn from(warning: CleanupWarning) -> Self {
        Self {
            image_id: warning.image_id,
            operation: warning.operation,
            status: warning.status,
            admin_message: warning.admin_message,
        }
    }
}

impl ItemResponse {
    fn from_item(item: AutographItem) -> Self {
        Self {
            id: item.id,
            title: item.title,
            signer: item.signer,
            description: item.description,
            category: item.category,
            tags: item.tags,
            object_reference: item.object_reference,
            event_name: item.event_name,
            event_location: item.event_location,
            source: item.source,
            inscription: item.inscription,
            certification_company: item.certification_company,
            certification_id: item.certification_id,
            estimated_year: item.estimated_year,
            publication_status: item.publication_status,
            images: item
                .images
                .into_iter()
                .map(|image| ImageResponse {
                    id: image.id,
                    content_type: image.content_type,
                    byte_size: image.byte_size,
                    is_primary: image.is_primary,
                    sort_order: image.sort_order,
                    alt_text: image.alt_text,
                })
                .collect(),
            pending_changes: None,
            cleanup_warnings: Vec::new(),
        }
    }

    fn from_item_with_state(
        item: AutographItem,
        pending_changes: admin_items::PendingMarkerResponse,
        cleanup_warnings: Vec<CleanupWarning>,
    ) -> Self {
        Self {
            pending_changes: Some(pending_changes),
            cleanup_warnings: cleanup_warnings
                .into_iter()
                .map(CleanupWarningResponse::from)
                .collect(),
            ..Self::from_item(item)
        }
    }
}

impl From<AutographItem> for ItemResponse {
    fn from(item: AutographItem) -> Self {
        Self::from_item(item)
    }
}

enum AuthKind {
    Session(String),
    OperatorToken,
}

fn authenticate(state: &AppState, headers: &HeaderMap) -> Option<AuthKind> {
    if let Some(token) = bearer_token(headers)
        && state.auth.has_operator_token(token)
    {
        return Some(AuthKind::OperatorToken);
    }

    let session = cookie_value(headers, SESSION_COOKIE)?;
    state
        .auth
        .has_session(session)
        .then(|| AuthKind::Session(session.to_owned()))
}

fn csrf_allowed(state: &AppState, method: &Method, headers: &HeaderMap, auth: &AuthKind) -> bool {
    if matches!(auth, AuthKind::OperatorToken) || matches!(*method, Method::GET | Method::HEAD) {
        return true;
    }

    headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|origin| origin == state.config.public_origin)
        || headers
            .get(header::REFERER)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|referer| referer.starts_with(&format!("{}/", state.config.public_origin)))
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix(&format!("{name}=")))
}

fn session_cookie(session: &str, secure: bool) -> String {
    format!(
        "{SESSION_COOKIE}={session}; Path=/admin; HttpOnly; SameSite=Strict{}",
        if secure { "; Secure" } else { "" }
    )
}
