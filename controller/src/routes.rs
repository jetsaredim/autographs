use std::{sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    auth::{AuthState, LoginError},
    catalog::{
        AutographImage, AutographItem, AutographItemInput, AutographItemUpdate, CatalogRepository,
        CleanupStatus, CleanupWarning, ImageCleanupEvent, ImageReplacementInput,
        MemoryCatalogRepository, PublicationStatus, REQUIRED_FIELDS_ERROR, now_epoch_seconds,
    },
    config::ControllerConfig,
    image_adjustments::ImageAdjustment,
    media::{LocalMediaStore, PrivateMediaStore},
    publisher::{LocalPublisher, PublishMode, PublishStatus, ReleaseRetentionPolicy},
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
    release: ReleaseVersionResponse,
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

pub fn runtime_router(mut config: ControllerConfig) -> Result<Router, String> {
    config.validate_runtime_auth()?;
    let repository: Arc<dyn CatalogRepository> =
        match provider("AUTOGRAPHS_CONTROLLER_DB_PROVIDER").as_str() {
            "local" => {
                tracing::info!("configuring local in-memory catalog repository");
                Arc::new(MemoryCatalogRepository::default())
            }
            "oracle" => production_repository(&mut config)?,
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
            // ast-grep-ignore: no-distributed-env-read
            let root = std::env::var("AUTOGRAPHS_CONTROLLER_LOCAL_MEDIA_ROOT")
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
    // ast-grep-ignore: no-distributed-env-read
    std::env::var(name).unwrap_or_else(|_| "local".to_owned())
}

#[cfg(feature = "production-persistence")]
fn production_repository(
    config: &mut ControllerConfig,
) -> Result<Arc<dyn CatalogRepository>, String> {
    use crate::{oracle_catalog::OracleCatalogRepository, oracle_heartbeat, oracle_schema};

    tracing::info!("configuring Oracle catalog repository");

    let connection_settings = take_oracle_connection_settings(config)?;

    oracle_schema::ensure_initialized(&connection_settings)?;

    tracing::info!("Oracle catalog schema is ready");

    oracle_heartbeat::spawn(Arc::clone(&connection_settings))?;

    Ok(Arc::new(OracleCatalogRepository::with_connection_settings(
        connection_settings,
        required_env("OCI_MEDIA_NAMESPACE")?,
        required_env("OCI_MEDIA_BUCKET_NAME")?,
    )))
}

#[cfg(feature = "production-persistence")]
fn take_oracle_connection_settings(
    config: &mut ControllerConfig,
) -> Result<Arc<crate::oracle_connection::OracleConnectionSettings>, String> {
    let user = take_required_config(&mut config.oracle_user, "ORACLE_DB_USER")?;
    let credential = take_required_config(&mut config.oracle_password, "ORACLE_DB_PASSWORD")?;
    let connect_string = take_required_config(
        &mut config.oracle_connect_string,
        "ORACLE_DB_CONNECT_STRING",
    )?;
    let credential_provider = Arc::new(
        match (
            config.oracle_password_vault_secret_id.take(),
            config.oracle_password_vault_version.take(),
        ) {
            (Some(secret_id), Some(vault_version)) => {
                crate::oracle_credentials::DatabaseCredentialProvider::with_oci_vault_refresh(
                    credential,
                    vault_version,
                    secret_id,
                )?
            }
            (None, None) => crate::oracle_credentials::DatabaseCredentialProvider::new(credential),
            _ => {
                return Err(
                "Oracle database Vault secret ID and resolved version must be configured together"
                    .to_owned(),
            );
            }
        },
    );

    Ok(Arc::new(
        crate::oracle_connection::OracleConnectionSettings::with_credential_provider(
            user,
            credential_provider,
            connect_string,
            config.oracle_wallet_dir.take(),
            config.oracle_wallet_password.take(),
        ),
    ))
}

#[cfg(not(feature = "production-persistence"))]
fn production_repository(
    _config: &mut ControllerConfig,
) -> Result<Arc<dyn CatalogRepository>, String> {
    Err("Oracle controller persistence requires the production-persistence feature".to_owned())
}

#[cfg(feature = "production-persistence")]
fn take_required_config(value: &mut Option<String>, name: &str) -> Result<String, String> {
    value
        .take()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{name} is required"))
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
    // ast-grep-ignore: no-distributed-env-read
    std::env::var(name)
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
    let retention_policy = ReleaseRetentionPolicy::new(
        config.static_promoted_release_retain_count,
        config.static_failed_candidate_retain_count,
    );
    let generator_metadata = config.publish_generator_metadata();
    router_with_services(
        config,
        repository,
        media,
        Arc::new(LocalPublisher::with_generator_metadata(
            static_release_root,
            retention_policy,
            generator_metadata,
        )),
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
        .route("/admin/api/status", get(admin_status))
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
        .route("/admin/api/signers", get(admin_items::list_signers))
        .route("/admin/api/signers/{id}", patch(admin_items::update_signer))
        .route("/admin/api/signers/merge", post(admin_items::merge_signers))
        .route(
            "/admin/api/taxonomy/suggestions",
            get(admin_items::taxonomy_suggestions),
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
        release: ReleaseVersionResponse::from_config(&state.config),
        oracle_configured: state.config.oracle_configured,
        media_configured: state.config.media_configured,
        static_release_configured: state.config.static_release_configured,
    })
}

async fn admin_status(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(status) = authorize_admin_session(&state, &Method::GET, &headers) {
        tracing::warn!(status = %status, "rejected admin status request");
        return status.into_response();
    }

    let started = Instant::now();
    let pending_changes = match state.repository.pending_changes().await {
        Ok(summary) => summary,
        Err(error) => {
            tracing::error!(error = %error, "failed to load pending changes for admin status");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let cleanup_warnings = match cleanup_warning_entries(&state).await {
        Ok(warnings) => warnings,
        Err(error) => {
            tracing::error!(error = %error, "failed to load cleanup warnings for admin status");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let release_retention = match state.publisher.retention_status() {
        Ok(status) => status,
        Err(error) => {
            tracing::error!(error = %error, "failed to load release retention status");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    tracing::info!(
        pending_change_count = pending_changes.count,
        cleanup_warning_count = cleanup_warnings.len(),
        promoted_release_count = release_retention.promoted_release_count,
        failed_candidate_count = release_retention.failed_candidate_count,
        elapsed_ms = started.elapsed().as_millis(),
        "loaded admin status"
    );

    Json(AdminStatusResponse {
        providers: ProviderModesResponse {
            database: provider("AUTOGRAPHS_CONTROLLER_DB_PROVIDER"),
            media: provider("AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER"),
        },
        controller: ControllerStatusResponse {
            ok: true,
            release: ReleaseVersionResponse::from_config(&state.config),
            oracle_configured: state.config.oracle_configured,
            media_configured: state.config.media_configured,
            static_release_configured: state.config.static_release_configured,
        },
        publish: PublishSummaryResponse::from(state.publisher.status()),
        pending_changes: PendingChangesResponse {
            count: pending_changes.count,
            oldest_changed_at_epoch_seconds: pending_changes.oldest_changed_at_epoch_seconds,
            has_pending_changes: pending_changes.count > 0,
        },
        cleanup: CleanupSummaryResponse {
            warning_count: cleanup_warnings.len(),
            has_warnings: !cleanup_warnings.is_empty(),
            warnings: cleanup_warnings,
        },
        release_retention: ReleaseRetentionResponse {
            active_release_id: release_retention.active_release_id,
            promoted_release_retain_count: release_retention.promoted_release_retain_count,
            promoted_release_count: release_retention.promoted_release_count,
            failed_candidate_retain_count: release_retention.failed_candidate_retain_count,
            failed_candidate_count: release_retention.failed_candidate_count,
        },
        live_smoke_guidance:
            "Run live smoke from docs/static-runtime-runbook.md when Oracle/Object Storage behavior changes.",
        cleanup_guidance: "Cleanup warnings must be resolved before trusting a publish batch.",
    })
    .into_response()
}

async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Response {
    let started = Instant::now();
    match state.auth.login(&payload.password) {
        Ok(session) => {
            tracing::info!(
                elapsed_ms = started.elapsed().as_millis(),
                secure_cookies = state.config.secure_cookies,
                "admin login succeeded"
            );
            let mut response = StatusCode::NO_CONTENT.into_response();
            response.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&session_cookie(&session, state.config.secure_cookies))
                    .expect("session cookie header"),
            );
            response
        }
        Err(LoginError::InvalidCredential) => {
            tracing::warn!(
                elapsed_ms = started.elapsed().as_millis(),
                "admin login rejected"
            );
            (StatusCode::UNAUTHORIZED, "Invalid admin credentials.").into_response()
        }
        Err(LoginError::Locked) => (
            {
                tracing::warn!(
                    elapsed_ms = started.elapsed().as_millis(),
                    "admin login rejected by lockout"
                );
                StatusCode::TOO_MANY_REQUESTS
            },
            "Too many login attempts. Wait and try again.",
        )
            .into_response(),
    }
}

async fn logout(State(state): State<AppState>, method: Method, headers: HeaderMap) -> Response {
    let auth = match authenticate(&state, &headers) {
        Some(auth) => auth,
        None => {
            tracing::warn!("rejected admin logout request");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };
    if !csrf_allowed(&state, &method, &headers, &auth) {
        tracing::warn!("rejected admin logout request by csrf guard");
        return StatusCode::FORBIDDEN.into_response();
    }
    if let AuthKind::Session(session) = auth {
        state.auth.logout(&session);
    }
    tracing::info!("admin logout succeeded");
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
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected create catalog item request");
        return status.into_response();
    }

    let started = Instant::now();
    tracing::info!(
        tag_count = input.tags.len(),
        signer_credit_count = input.signer_credits.len(),
        character_count = input.characters.len(),
        franchise_count = input.franchises.len(),
        publication_status = ?input.publication_status,
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
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
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
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected upload image request");
        return status.into_response();
    }
    let Ok(item_id) = Uuid::parse_str(&id) else {
        tracing::warn!("rejected upload image request with malformed item id");
        return StatusCode::BAD_REQUEST.into_response();
    };
    let existing_item = match state.repository.get(item_id).await {
        Ok(Some(item)) => item,
        Ok(None) => {
            tracing::warn!(%item_id, "rejected upload image request for missing item");
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(error) => {
            tracing::error!(%item_id, error = %error, "failed to load item before image upload");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
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
        tracing::warn!(%item_id, "rejected image upload without image body");
        return StatusCode::BAD_REQUEST.into_response();
    };
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_owned());
    if !matches!(
        content_type.as_str(),
        "image/jpeg" | "image/png" | "image/webp"
    ) || body.len() > MAX_IMAGE_UPLOAD_BYTES
    {
        tracing::warn!(
            %item_id,
            content_type = %content_type,
            byte_size = body.len(),
            "rejected image upload by content type or size"
        );
        return StatusCode::BAD_REQUEST.into_response();
    }
    if !valid_image_upload(&content_type, &body) {
        tracing::warn!(
            %item_id,
            content_type = %content_type,
            byte_size = body.len(),
            "rejected image upload by image validation"
        );
        return StatusCode::BAD_REQUEST.into_response();
    }

    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item_id, image_id);
    let started = Instant::now();
    tracing::info!(
        %item_id,
        %image_id,
        content_type = %content_type,
        byte_size = body.len(),
        requested_primary = requested_primary,
        "uploading catalog image"
    );
    if let Err(error) = state.media.write(&object_key, &body).await {
        tracing::error!(
            %item_id,
            %image_id,
            error_kind = classify_media_error(&error),
            "failed to write uploaded image to private media store"
        );
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let image = AutographImage {
        id: image_id,
        object_key: object_key.clone(),
        original_filename: filename.unwrap_or_else(|| "upload".to_owned()),
        content_type,
        byte_size: body.len(),
        checksum: Some(image_checksum(&body)),
        etag: None,
        is_primary: existing_item.images.is_empty() || requested_primary.unwrap_or(false),
        sort_order: existing_item
            .images
            .iter()
            .map(|image| image.sort_order)
            .max()
            .unwrap_or(-1)
            + 1,
        alt_text,
        adjustment: None,
    };
    match state.repository.attach_image(item_id, image).await {
        Ok(item) => {
            tracing::info!(
                %item_id,
                %image_id,
                image_count = item.images.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "uploaded catalog image"
            );
            let response = item_response_with_state(&state, item).await;
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(error) => {
            tracing::error!(%item_id, %image_id, error = %error, "failed to attach uploaded image metadata");
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
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected set primary image request");
        return status.into_response();
    }
    let (Ok(item_id), Ok(image_id)) = (Uuid::parse_str(&id), Uuid::parse_str(&image_id)) else {
        tracing::warn!("rejected set primary image request with malformed id");
        return StatusCode::BAD_REQUEST.into_response();
    };
    let started = Instant::now();
    tracing::info!(%item_id, %image_id, "setting primary image");
    match state.repository.set_primary_image(item_id, image_id).await {
        Ok(item) => {
            tracing::info!(
                %item_id,
                %image_id,
                image_count = item.images.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "set primary image"
            );
            Json(item_response_with_state(&state, item).await).into_response()
        }
        Err(error) => {
            tracing::error!(%item_id, %image_id, error = %error, "failed to set primary image");
            repository_update_error_status(&error).into_response()
        }
    }
}

async fn delete_image(
    State(state): State<AppState>,
    Path((id, image_id)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected delete image request");
        return status.into_response();
    }
    let (Ok(item_id), Ok(image_id)) = (Uuid::parse_str(&id), Uuid::parse_str(&image_id)) else {
        tracing::warn!("rejected delete image request with malformed id");
        return StatusCode::BAD_REQUEST.into_response();
    };
    let started = Instant::now();
    tracing::info!(%item_id, %image_id, "deleting catalog image");
    let item = match state.repository.get(item_id).await {
        Ok(Some(item)) => item,
        Ok(None) => {
            tracing::warn!(%item_id, %image_id, "rejected delete image request for missing item");
            return StatusCode::NOT_FOUND.into_response();
        }
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
        tracing::warn!(%item_id, %image_id, "rejected delete image request for missing image");
        return StatusCode::NOT_FOUND.into_response();
    };

    if let Err(error) = state.media.delete(&image.object_key).await {
        tracing::warn!(
            %item_id,
            %image_id,
            error_kind = classify_media_error(&error),
            "private image delete failed"
        );
        return cleanup_warning_response(&state, item_id, image_id, &image.object_key, "delete")
            .await;
    }

    match state
        .repository
        .remove_image_metadata(item_id, image_id)
        .await
    {
        Ok(item) => {
            tracing::info!(
                %item_id,
                %image_id,
                image_count = item.images.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "deleted catalog image"
            );
            Json(item_response_with_state(&state, item).await).into_response()
        }
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
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected replace image request");
        return status.into_response();
    }
    let (Ok(item_id), Ok(image_id)) = (Uuid::parse_str(&id), Uuid::parse_str(&image_id)) else {
        tracing::warn!("rejected replace image request with malformed id");
        return StatusCode::BAD_REQUEST.into_response();
    };
    let started = Instant::now();
    tracing::info!(%item_id, %image_id, "replacing catalog image");
    let item = match state.repository.get(item_id).await {
        Ok(Some(item)) => item,
        Ok(None) => {
            tracing::warn!(%item_id, %image_id, "rejected replace image request for missing item");
            return StatusCode::NOT_FOUND.into_response();
        }
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
        tracing::warn!(%item_id, %image_id, "rejected replace image request for missing image");
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(upload) = parse_image_multipart(multipart).await else {
        tracing::warn!(%item_id, %image_id, "rejected replace image request without image body");
        return StatusCode::BAD_REQUEST.into_response();
    };
    if upload.body.len() > MAX_IMAGE_UPLOAD_BYTES
        || !valid_image_upload(&upload.content_type, &upload.body)
    {
        tracing::warn!(
            %item_id,
            %image_id,
            content_type = %upload.content_type,
            byte_size = upload.body.len(),
            "rejected replacement image by validation"
        );
        return StatusCode::BAD_REQUEST.into_response();
    }

    let replacement_id = Uuid::new_v4();
    let replacement_key = build_original_object_key(item_id, replacement_id);
    tracing::info!(
        %item_id,
        %image_id,
        %replacement_id,
        content_type = %upload.content_type,
        byte_size = upload.body.len(),
        "writing replacement image"
    );
    if let Err(error) = state.media.write(&replacement_key, &upload.body).await {
        tracing::error!(
            %item_id,
            %replacement_id,
            error_kind = classify_media_error(&error),
            "failed to write replacement image"
        );
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let replacement = AutographImage {
        id: image_id,
        object_key: replacement_key.clone(),
        original_filename: upload.filename.unwrap_or_else(|| "upload".to_owned()),
        content_type: upload.content_type,
        byte_size: upload.body.len(),
        checksum: Some(image_checksum(&upload.body)),
        etag: None,
        is_primary: existing_image.is_primary,
        sort_order: existing_image.sort_order,
        alt_text: upload.alt_text,
        adjustment: None,
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
        tracing::warn!(
            %item_id,
            %image_id,
            error_kind = classify_media_error(&error),
            "old private image cleanup failed after replacement"
        );
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
                    tracing::warn!(
                        %item_id,
                        %image_id,
                        error_kind = classify_media_error(&delete_error),
                        "failed to delete replacement object after cleanup warning persistence failure"
                    );
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
    tracing::info!(
        %item_id,
        %image_id,
        cleanup_warning_created = response.cleanup_warning.is_some(),
        elapsed_ms = started.elapsed().as_millis(),
        "replaced catalog image"
    );
    Json(response).into_response()
}

async fn retry_image_cleanup(
    State(state): State<AppState>,
    Path((id, image_id)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected cleanup retry request");
        return status.into_response();
    }
    let (Ok(item_id), Ok(image_id)) = (Uuid::parse_str(&id), Uuid::parse_str(&image_id)) else {
        tracing::warn!("rejected cleanup retry request with malformed id");
        return StatusCode::BAD_REQUEST.into_response();
    };
    let started = Instant::now();
    tracing::info!(%item_id, %image_id, "retrying private image cleanup");
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
        tracing::warn!(
            %item_id,
            %image_id,
            operation = %cleanup_warning.operation,
            error_kind = classify_media_error(&error),
            "private image cleanup retry failed"
        );
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
    tracing::info!(
        %item_id,
        %image_id,
        operation = %cleanup_warning.operation,
        removed_metadata = removed.is_some(),
        elapsed_ms = started.elapsed().as_millis(),
        "private image cleanup retry succeeded"
    );
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

fn image_checksum(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn set_publication(
    State(state): State<AppState>,
    Path(id): Path<String>,
    method: Method,
    headers: HeaderMap,
    Json(input): Json<PublicationRequest>,
) -> Response {
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
        tracing::warn!(status = %status, "rejected publication update request");
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
    if let Err(status) = authorize_admin_session(&state, &method, &headers) {
        tracing::warn!(status = %status, mode = ?mode, "rejected static publish request");
        return status.into_response();
    }

    let started = Instant::now();
    let publish_guard = state.publisher.acquire_publish_lock().await;
    let publish_boundary = match state.repository.begin_publish_boundary().await {
        Ok(boundary) => boundary,
        Err(error) => {
            tracing::error!(error = %error, "failed to capture publish boundary");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    tracing::info!(mode = ?mode, "publishing static release");
    match state
        .publisher
        .publish_locked(
            state.repository.as_ref(),
            state.media.as_ref(),
            mode,
            &publish_guard,
        )
        .await
    {
        Ok(status) => {
            let finished_at_epoch_seconds = status
                .finished_at_epoch_seconds
                .unwrap_or_else(now_epoch_seconds);
            if let Err(error) = state
                .repository
                .record_successful_publish(
                    publish_mode_label(mode),
                    status.release_id.as_deref(),
                    publish_boundary,
                    status.started_at_epoch_seconds,
                    finished_at_epoch_seconds,
                )
                .await
            {
                tracing::error!(error = %error, "failed to record successful publish job");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            tracing::info!(
                mode = ?mode,
                release_id = status.release_id.as_deref().unwrap_or("<none>"),
                stage = status.stage.as_deref().unwrap_or("succeeded"),
                artifact_count = status.artifact_count,
                byte_size = status.byte_size,
                item_count = status.item_count,
                image_count = status.image_count,
                derivative_count = status.derivative_count,
                elapsed_ms = started.elapsed().as_millis(),
                "published static release"
            );
            (StatusCode::CREATED, Json(status)).into_response()
        }
        Err(error) => {
            let status = state.publisher.status();
            let stage = status.stage.as_deref().unwrap_or("failed");
            let error_kind = crate::publisher::classify_publish_error(stage, &error);
            tracing::error!(
                mode = ?mode,
                stage,
                error_kind,
                "failed to publish static release"
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(status)).into_response()
        }
    }
}

async fn publish_status(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(status) = authorize_admin_session(&state, &Method::GET, &headers) {
        return status.into_response();
    }
    Json(state.publisher.status()).into_response()
}

fn publish_mode_label(mode: PublishMode) -> &'static str {
    match mode {
        PublishMode::Full => "full",
        PublishMode::Incremental => "incremental",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, header},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    #[cfg(feature = "production-persistence")]
    #[test]
    fn oracle_settings_move_out_of_router_config_without_changing_health_flag() {
        let mut config = ControllerConfig::for_test(true);
        config.oracle_configured = true;
        config.oracle_user = Some("ADMIN".to_owned());
        config.oracle_password = Some("database-password".to_owned());
        config.oracle_connect_string = Some("autographsdb_medium".to_owned());
        config.oracle_wallet_dir = Some("/opt/autographs/wallet".to_owned());
        config.oracle_wallet_password = Some("wallet-password".to_owned());

        let settings = take_oracle_connection_settings(&mut config).unwrap();

        assert_eq!(settings.user(), "ADMIN");
        assert_eq!(settings.connect_string(), "autographsdb_medium");
        assert!(config.oracle_user.is_none());
        assert!(config.oracle_password.is_none());
        assert!(config.oracle_connect_string.is_none());
        assert!(config.oracle_wallet_dir.is_none());
        assert!(config.oracle_wallet_password.is_none());
        assert!(config.oracle_configured);
    }

    #[tokio::test]
    async fn queued_publish_captures_boundary_after_waiting_for_publish_lock() {
        let repository = Arc::new(MemoryCatalogRepository::default());
        let media_root = tempfile::tempdir().expect("media root");
        let static_root = tempfile::tempdir().expect("static root");
        let media = Arc::new(LocalMediaStore::new(media_root.path()));
        let item = repository
            .create(test_item_input(
                "Queued Publish Item",
                PublicationStatus::Published,
            ))
            .await
            .expect("create item");
        let image_id = Uuid::new_v4();
        let object_key = build_original_object_key(item.id, image_id);
        media
            .write(&object_key, &png_fixture())
            .await
            .expect("write media");
        repository
            .attach_image(
                item.id,
                AutographImage {
                    id: image_id,
                    object_key,
                    original_filename: "private-queued.png".to_owned(),
                    content_type: "image/png".to_owned(),
                    byte_size: 128,
                    checksum: None,
                    etag: None,
                    is_primary: true,
                    sort_order: 0,
                    alt_text: None,
                    adjustment: None,
                },
            )
            .await
            .expect("attach image");

        let mut config = ControllerConfig::for_test(false);
        config.static_release_root = static_root.path().to_path_buf();
        let publisher = Arc::new(LocalPublisher::with_retention_policy(
            static_root.path(),
            ReleaseRetentionPolicy::default(),
        ));
        let publish_guard = publisher.acquire_publish_lock().await;
        let app = router_with_services(config, repository, media.clone(), Arc::clone(&publisher));

        let publish = spawn_publish(app.clone()).await;
        patch_item_title(&app, item.id, "Queued Publish Item Updated").await;
        drop(publish_guard);

        assert_eq!(
            publish.await.expect("queued publish").status(),
            StatusCode::CREATED
        );
        let status = admin_status(&app).await;
        assert_eq!(status["pendingChanges"]["count"], 0);
        assert_eq!(status["pendingChanges"]["hasPendingChanges"], false);
    }

    async fn spawn_publish(app: Router) -> tokio::task::JoinHandle<axum::response::Response> {
        let cookie = admin_cookie(&app).await;
        tokio::spawn(async move {
            app.oneshot(
                Request::post("/admin/api/publish/full")
                    .header(header::COOKIE, cookie)
                    .header(header::ORIGIN, "https://autographs.example.test")
                    .body(Body::empty())
                    .expect("publish request"),
            )
            .await
            .expect("publish response")
        })
    }

    async fn patch_item_title(app: &Router, item_id: Uuid, title: &str) {
        let response = app
            .clone()
            .oneshot(
                Request::patch(format!("/admin/api/items/{item_id}"))
                    .header(header::COOKIE, admin_cookie(app).await)
                    .header(header::ORIGIN, "https://autographs.example.test")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "title": title }).to_string()))
                    .expect("patch request"),
            )
            .await
            .expect("patch response");
        assert_eq!(response.status(), StatusCode::OK);
    }

    async fn admin_status(app: &Router) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::get("/admin/api/status")
                    .header(header::COOKIE, admin_cookie(app).await)
                    .header(header::ORIGIN, "https://autographs.example.test")
                    .body(Body::empty())
                    .expect("status request"),
            )
            .await
            .expect("status response");
        assert_eq!(response.status(), StatusCode::OK);
        response_json(response).await
    }

    async fn admin_cookie(app: &Router) -> String {
        let response = app
            .clone()
            .oneshot(
                Request::post("/admin/api/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"password":"local-test-password"}"#))
                    .expect("login request"),
            )
            .await
            .expect("login response");
        response
            .headers()
            .get(header::SET_COOKIE)
            .expect("set cookie")
            .to_str()
            .expect("cookie text")
            .split(';')
            .next()
            .expect("cookie pair")
            .to_owned()
    }

    async fn response_json(response: axum::response::Response) -> Value {
        serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("body"),
        )
        .expect("json")
    }

    fn png_fixture() -> Vec<u8> {
        let mut body = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(8, 8, image::Rgb([1, 2, 3])))
            .write_to(&mut body, ImageFormat::Png)
            .expect("write png");
        body.into_inner()
    }

    fn test_item_input(title: &str, publication_status: PublicationStatus) -> AutographItemInput {
        AutographItemInput {
            title: title.to_owned(),
            signer: "Rosario Dawson".to_owned(),
            description: None,
            category: "Photos".to_owned(),
            tags: vec!["ahsoka".to_owned()],
            signer_credits: Vec::new(),
            characters: Vec::new(),
            format: "Photo".to_owned(),
            origin: crate::catalog::ItemOrigin::Official,
            franchises: Vec::new(),
            product_line: None,
            set_name: None,
            language: "English".to_owned(),
            object_reference: None,
            event_name: None,
            event_location: None,
            source: None,
            inscription: None,
            certification_company: None,
            certification_id: None,
            estimated_year: None,
            publication_status,
        }
    }
}

async fn cleanup_warning_entries(
    state: &AppState,
) -> Result<Vec<CleanupWarningSummaryResponse>, String> {
    let items = state.repository.list().await?;
    let mut warnings = Vec::new();
    for item in items {
        for warning in state.repository.cleanup_warnings(item.id).await? {
            warnings.push(CleanupWarningSummaryResponse {
                item_id: item.id,
                title: item.title.clone(),
                image_id: warning.image_id,
                operation: warning.operation,
                status: warning.status,
                admin_message: warning.admin_message,
            });
        }
    }
    Ok(warnings)
}

pub(super) fn authorize_admin_session(
    state: &AppState,
    method: &Method,
    headers: &HeaderMap,
) -> Result<String, StatusCode> {
    let auth = authenticate(state, headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let AuthKind::Session(session) = &auth else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    csrf_allowed(state, method, headers, &auth)
        .then_some(session.clone())
        .ok_or(StatusCode::FORBIDDEN)
}

fn repository_update_error_status(error: &str) -> StatusCode {
    if error == REQUIRED_FIELDS_ERROR
        || error.contains("required")
        || error.contains("must be")
        || error.contains("must point to")
        || error.contains("duplicate signer credits")
        || error.contains("not allowed")
        || error.contains("cannot be")
    {
        StatusCode::BAD_REQUEST
    } else if error.contains("not found") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn classify_media_error(error: &str) -> &'static str {
    let normalized = error.to_ascii_lowercase();
    if normalized.contains("not found") || normalized.contains("404") {
        "media_not_found"
    } else if normalized.contains("unauthorized")
        || normalized.contains("forbidden")
        || normalized.contains("permission")
        || normalized.contains("401")
        || normalized.contains("403")
    {
        "media_auth"
    } else if normalized.contains("timeout") || normalized.contains("timed out") {
        "media_timeout"
    } else if normalized.contains("image") || normalized.contains("content type") {
        "media_validation"
    } else {
        "media_io"
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
    signer_credits: Vec<SignerCreditResponse>,
    characters: Vec<String>,
    format: String,
    origin: crate::catalog::ItemOrigin,
    franchises: Vec<String>,
    product_line: Option<String>,
    set_name: Option<String>,
    language: String,
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
struct SignerCreditResponse {
    signer: admin_items::AdminSignerProfileResponse,
    sort_order: i32,
    item_role: Option<String>,
    item_context: Option<String>,
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
    adjustment: Option<ImageAdjustment>,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminStatusResponse {
    providers: ProviderModesResponse,
    controller: ControllerStatusResponse,
    publish: PublishSummaryResponse,
    pending_changes: PendingChangesResponse,
    cleanup: CleanupSummaryResponse,
    release_retention: ReleaseRetentionResponse,
    live_smoke_guidance: &'static str,
    cleanup_guidance: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderModesResponse {
    database: String,
    media: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ControllerStatusResponse {
    ok: bool,
    release: ReleaseVersionResponse,
    oracle_configured: bool,
    media_configured: bool,
    static_release_configured: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseVersionResponse {
    repo_version: Option<String>,
    controller_version: Option<String>,
    controller_image: Option<String>,
    source_revision: Option<String>,
}

impl ReleaseVersionResponse {
    fn from_config(config: &ControllerConfig) -> Self {
        Self {
            repo_version: config.repo_version.clone(),
            controller_version: config.controller_version.clone(),
            controller_image: config.controller_image.clone(),
            source_revision: config.source_revision.clone(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PublishSummaryResponse {
    state: String,
    release_id: Option<String>,
    artifact_count: usize,
    byte_size: usize,
    started_at_epoch_seconds: Option<i64>,
    finished_at_epoch_seconds: Option<i64>,
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PendingChangesResponse {
    count: usize,
    oldest_changed_at_epoch_seconds: Option<i64>,
    has_pending_changes: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupSummaryResponse {
    warning_count: usize,
    has_warnings: bool,
    warnings: Vec<CleanupWarningSummaryResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupWarningSummaryResponse {
    item_id: Uuid,
    title: String,
    image_id: Uuid,
    operation: String,
    status: CleanupStatus,
    admin_message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseRetentionResponse {
    active_release_id: Option<String>,
    promoted_release_retain_count: usize,
    promoted_release_count: usize,
    failed_candidate_retain_count: usize,
    failed_candidate_count: usize,
}

impl From<PublishStatus> for PublishSummaryResponse {
    fn from(status: PublishStatus) -> Self {
        Self {
            state: status.state,
            release_id: status.release_id,
            artifact_count: status.artifact_count,
            byte_size: status.byte_size,
            started_at_epoch_seconds: status.started_at_epoch_seconds,
            finished_at_epoch_seconds: status.finished_at_epoch_seconds,
            error: status.error,
        }
    }
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
            signer_credits: item
                .signer_credits
                .into_iter()
                .map(|credit| SignerCreditResponse {
                    signer: admin_items::AdminSignerProfileResponse::from(credit.signer),
                    sort_order: credit.sort_order,
                    item_role: credit.item_role,
                    item_context: credit.item_context,
                })
                .collect(),
            characters: item.characters,
            format: item.format,
            origin: item.origin,
            franchises: item.franchises,
            product_line: item.product_line,
            set_name: item.set_name,
            language: item.language,
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
                    adjustment: image.adjustment,
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
