use std::{env, sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthState,
    catalog::{
        AutographImage, AutographItem, AutographItemInput, AutographItemUpdate, CatalogRepository,
        MemoryCatalogRepository, PublicationStatus, REQUIRED_FIELDS_ERROR,
    },
    config::ControllerConfig,
    media::{LocalMediaStore, PrivateMediaStore},
    publisher::{LocalPublisher, PublishMode},
    storage_keys::build_original_object_key,
};

mod admin_items;

const SESSION_COOKIE: &str = "autographs_admin_session";

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
            let pending = admin_items::pending_marker(&state, item_id).await;
            (
                StatusCode::CREATED,
                Json(ItemResponse::from_item_with_pending(item, pending)),
            )
                .into_response()
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
            let pending = admin_items::pending_marker(&state, id).await;
            Json(ItemResponse::from_item_with_pending(item, pending)).into_response()
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
    match state.repository.get(item_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    let mut filename = None;
    let mut content_type = None;
    let mut body = None;
    let mut alt_text = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            filename = field.file_name().map(str::to_owned);
            content_type = field.content_type().map(str::to_owned);
            body = field.bytes().await.ok();
        } else if field.name() == Some("altText") {
            alt_text = field.text().await.ok();
        }
    }
    let Some(body) = body else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_owned());
    if !matches!(
        content_type.as_str(),
        "image/jpeg" | "image/png" | "image/webp"
    ) || body.len() > 20 * 1024 * 1024
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
        is_primary: true,
        sort_order: 0,
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
            let pending = admin_items::pending_marker(&state, item_id).await;
            (
                StatusCode::CREATED,
                Json(ItemResponse::from_item_with_pending(item, pending)),
            )
                .into_response()
        }
        Err(error) => {
            tracing::error!(%item_id, %image_id, %object_key, ?error, "failed to attach uploaded image metadata");
            let _ = state.media.delete(&object_key).await;
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
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
            let pending = admin_items::pending_marker(&state, id).await;
            Json(ItemResponse::from_item_with_pending(item, pending)).into_response()
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
        }
    }

    fn from_item_with_pending(
        item: AutographItem,
        pending_changes: admin_items::PendingMarkerResponse,
    ) -> Self {
        Self {
            pending_changes: Some(pending_changes),
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
