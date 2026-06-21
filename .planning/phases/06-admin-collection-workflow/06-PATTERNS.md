# Phase 6: Admin Collection Workflow - Pattern Map

**Mapped:** 2026-06-21
**Files analyzed:** 17 new/modified files
**Analogs found:** 17 / 17

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `controller/src/catalog.rs` | model/service trait | CRUD, event-driven | `controller/src/catalog.rs` | exact |
| `controller/src/oracle_catalog.rs` | persistence adapter | CRUD, batch | `controller/src/oracle_catalog.rs` | exact |
| `controller/db/schema.sql` | schema | CRUD, event-driven | `controller/db/schema.sql` | exact |
| `controller/src/oracle_schema.rs` | schema bootstrap | batch | `controller/src/oracle_schema.rs` | exact |
| `controller/src/routes.rs` | route/controller | request-response, file-I/O | `controller/src/routes.rs` | exact |
| `controller/src/media.rs` | service trait | file-I/O | `controller/src/media.rs` | exact |
| `controller/src/oci_media.rs` | service adapter | file-I/O | `controller/src/media.rs` | role-match |
| `controller/src/publisher.rs` | service | batch, file-I/O, transform | `controller/src/publisher.rs` | exact |
| `controller/static-admin/index.html` | static component | request-response | `controller/static-admin/index.html` | exact |
| `controller/static-admin/admin.js` | frontend client | request-response, file-I/O | `controller/static-admin/admin.js` | exact |
| `controller/static-admin/admin.css` | static component | presentation | `controller/static-admin/admin.css` | exact |
| `controller/tests/admin_workflow.rs` | test | request-response, CRUD | `controller/tests/seed_content.rs` | role-match |
| `controller/tests/media_cleanup.rs` | test | file-I/O, CRUD | `controller/tests/seed_content.rs` | role-match |
| `controller/tests/publisher.rs` | test | batch, file-I/O | `controller/tests/publisher.rs` | exact |
| `controller/tests/static_admin.rs` | test | static privacy | `controller/tests/static_admin.rs` | exact |
| `docs/configuration-contract.md`, `docs/controller-walkthrough.md`, `docs/deployment-runbook.md`, `docs/static-runtime-runbook.md` | docs | operator workflow | existing docs from research | role-match |
| `deploy/ansible/roles/autographs_deploy/templates/controller.env.j2` | config | runtime env | `deploy/ansible/roles/autographs_deploy/templates/controller.env.j2` | exact |

## Pattern Assignments

### `controller/src/routes.rs` (route/controller, request-response + file-I/O)

**Analog:** `controller/src/routes.rs`

**Imports and state pattern** (lines 1-24, 28-35):
```rust
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::{
    auth::AuthState,
    catalog::{
        AutographImage, AutographItem, AutographItemInput, AutographItemUpdate, CatalogRepository,
        MemoryCatalogRepository, PublicationStatus,
    },
    media::{LocalMediaStore, PrivateMediaStore},
    publisher::{LocalPublisher, PublishMode},
};
```

**Route registration pattern** (lines 196-210):
```rust
Router::new()
    .route("/health", get(health))
    .route("/admin/api/health", get(admin_health))
    .route("/admin/api/login", post(login))
    .route("/admin/api/logout", post(logout))
    .route("/admin/api/items", post(create_item))
    .route("/admin/api/items/{id}", axum::routing::patch(update_item))
    .route("/admin/api/items/{id}/images", post(upload_image))
    .route("/admin/api/items/{id}/publication", post(set_publication))
    .route("/admin/api/publish/incremental", post(publish_incremental))
    .route("/admin/api/publish/full", post(publish_full))
    .route("/admin/api/publish/status", get(publish_status))
    .layer(DefaultBodyLimit::max(25 * 1024 * 1024))
```

**Mutation handler pattern** (lines 291-321):
```rust
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

    match state.repository.create(input).await {
        Ok(item) => (StatusCode::CREATED, Json(ItemResponse::from(item))).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "failed to create catalog item");
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}
```

**Upload/cleanup rollback pattern** (lines 407-447):
```rust
let image_id = Uuid::new_v4();
let object_key = build_original_object_key(item_id, image_id);
if let Err(error) = state.media.write(&object_key, &body).await {
    tracing::error!(%item_id, %image_id, %object_key, %error, "failed to write uploaded image to private media store");
    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
}
let image = AutographImage { id: image_id, object_key: object_key.clone(), /* ... */ };
match state.repository.attach_image(item_id, image).await {
    Ok(item) => (StatusCode::CREATED, Json(ItemResponse::from(item))).into_response(),
    Err(error) => {
        tracing::error!(%item_id, %image_id, %object_key, ?error, "failed to attach uploaded image metadata");
        let _ = state.media.delete(&object_key).await;
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
```

Apply this to new item list/get/history, image delete/replace/primary, pending-status, cleanup-warning, and publish diagnostics routes. Keep internal object keys out of response DTOs.

### `controller/src/catalog.rs` (model/service trait, CRUD + events)

**Analog:** `controller/src/catalog.rs`

**Domain DTO pattern** (lines 18-56):
```rust
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
    pub publication_status: Option<PublicationStatus>,
}
```

**Repository trait pattern** (lines 90-101):
```rust
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
```

Add history, pending-change, image delete/replace/primary, and cleanup event contracts here first, then implement memory and Oracle adapters together. Use a patch-field representation for optional fields that can distinguish unchanged, set, and clear.

### `controller/src/oracle_catalog.rs` (persistence adapter, CRUD + batch)

**Analog:** `controller/src/oracle_catalog.rs`

**Blocking Oracle wrapper pattern** (lines 37-54):
```rust
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
```

**Transactional update pattern** (lines 103-149):
```rust
let mut item = load_item(&connection, id)?
    .ok_or_else(|| "autograph item was not found".to_owned())?;
apply_update(&mut item, input);
validate_input(&item.title, &item.signer, &item.category)?;
let statement = connection
    .execute("update autograph_items set ... updated_at = current_timestamp where id = :14", &[/* ... */])
    .map_err(|error| format!("update Oracle catalog item: {error}"))?;
if statement.row_count().map_err(|error| format!("read Oracle catalog update row count: {error}"))? == 0 {
    return Err("autograph item was not found".to_owned());
}
replace_tags(&connection, id, &item.tags)?;
connection
    .commit()
    .map_err(|error| format!("commit Oracle catalog update: {error}"))?;
```

**Primary image pattern** (lines 190-207):
```rust
let existing_item = load_item(&connection, item_id)?
    .ok_or_else(|| "autograph item was not found".to_owned())?;
if image.is_primary && existing_item.images.iter().any(|image| image.is_primary) {
    connection
        .execute(
            "update autograph_images set is_primary = 'N', updated_at = current_timestamp where item_id = :1",
            &[&item_id_text],
        )
        .map_err(|error| format!("clear Oracle primary image: {error}"))?;
}
```

History/event inserts should be in the same Oracle transaction as metadata/image/publication changes where possible. Cross-system OCI deletes cannot share this transaction, so record cleanup status and retry guidance.

### `controller/db/schema.sql` and `controller/src/oracle_schema.rs` (schema, batch)

**Analog:** `controller/db/schema.sql`, `controller/src/oracle_schema.rs`

**Schema style** (schema lines 7-26, 37-64):
```sql
create table autograph_items (
  id varchar2(36) primary key,
  title varchar2(255) not null,
  signer varchar2(255) not null,
  description clob,
  publication_status varchar2(24) default 'draft' not null,
  created_at timestamp default current_timestamp not null,
  updated_at timestamp default current_timestamp not null,
  constraint autograph_items_publication_ck
    check (publication_status in ('draft', 'published', 'archived'))
);

create table autograph_images (
  id varchar2(36) primary key,
  item_id varchar2(36) not null,
  object_key varchar2(1024) not null,
  is_primary char(1) default 'N' not null,
  primary_item_id generated always as (
    case when is_primary = 'Y' then item_id end
  ) virtual,
  constraint autograph_images_item_fk
    foreign key (item_id) references autograph_items(id) on delete cascade
);
```

**Schema preflight pattern** (`oracle_schema.rs` lines 6-18, 40-68):
```rust
const EXPECTED_TABLES: &[&str] = &[
    "AUTOGRAPH_ITEMS",
    "AUTOGRAPH_ITEM_TAGS",
    "AUTOGRAPH_IMAGES",
    "AUTOGRAPH_PUBLISH_JOBS",
    "AUTOGRAPH_PUBLIC_DERIVATIVES",
];
const REQUIRED_COLUMNS: &[(&str, &str)] = &[
    ("AUTOGRAPH_ITEMS", "PUBLICATION_STATUS"),
    ("AUTOGRAPH_IMAGES", "ORIGINAL_FILENAME"),
];
```

Add new edit-event / cleanup-event tables to the end-state schema and update `EXPECTED_TABLES` / `REQUIRED_COLUMNS` so partially initialized live schemas fail closed with a clear operator error.

### `controller/src/media.rs` and `controller/src/oci_media.rs` (media service, file-I/O)

**Analog:** `controller/src/media.rs`

**Trait and idempotent local delete pattern** (lines 9-14, 61-67):
```rust
#[async_trait]
pub trait PrivateMediaStore: Send + Sync {
    async fn write(&self, object_key: &str, body: &[u8]) -> Result<(), String>;
    async fn read(&self, object_key: &str) -> Result<Vec<u8>, String>;
    async fn delete(&self, object_key: &str) -> Result<(), String>;
}

async fn delete(&self, object_key: &str) -> Result<(), String> {
    match fs::remove_file(self.path_for(object_key)?).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("delete media object: {error}")),
    }
}
```

Use this existing delete contract for normal image delete/replace cleanup. Do not expose object keys through admin DTOs; route/service logs may include them, admin-visible warnings should not.

### `controller/src/publisher.rs` (publisher service, batch + transform)

**Analog:** `controller/src/publisher.rs`

**Publish status and promotion pattern** (lines 440-481):
```rust
let release_id = Uuid::new_v4().to_string();
let candidate = self.root.join("releases").join(&release_id);
self.set_status(PublishStatus {
    state: "running".to_owned(),
    release_id: Some(release_id.clone()),
    started_at_epoch_seconds: Some(started_at_epoch_seconds),
    ..Default::default()
});

let result = self
    .build_candidate(repository, media, mode, &release_id, &candidate)
    .await
    .and_then(|_| validate_candidate(&candidate))
    .and_then(|manifest| {
        promote_candidate(&self.root, &release_id)?;
        Ok(manifest)
    });
```

**Primary-first public image pattern** (lines 562-599, 629-638):
```rust
for (index, image) in primary_first_images(&item.images).into_iter().enumerate() {
    let image_slug = format!("image-{}", index + 1);
    let source = media.read(&image.object_key).await?;
    /* generate public derivatives */
}

fn primary_first_images(images: &[AutographImage]) -> Vec<&AutographImage> {
    let mut ordered = images.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        right
            .is_primary
            .cmp(&left.is_primary)
            .then_with(|| left.sort_order.cmp(&right.sort_order))
            .then_with(|| left.id.cmp(&right.id))
    });
    ordered
}
```

**Retention/privacy pattern** (lines 942-991, 994-1001):
```rust
fn retain_latest_failed_candidate(root: &Path, candidate: &Path) -> Result<(), String> {
    let failed_root = root.join("failed");
    fs::create_dir_all(&failed_root)
        .map_err(|error| format!("create failed release root: {error}"))?;
    for entry in fs::read_dir(&failed_root).map_err(|error| format!("read failed release root: {error}"))? {
        let path = entry.map_err(|error| format!("read failed release entry: {error}"))?.path();
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|error| format!("prune failed candidate: {error}"))?;
        }
    }
    /* retain latest failed candidate */
}
```

Add promoted-release pruning beside failed-candidate retention. Never delete the active `current` target. Keep validation scans rejecting storage identifiers, object keys, image UUIDs, and original filenames.

### `controller/static-admin/*` (static admin shell, request-response)

**Analog:** `controller/static-admin/index.html`, `controller/static-admin/admin.js`, `controller/static-admin/admin.css`

**Same-origin endpoint and fetch pattern** (`admin.js` lines 1-36):
```javascript
const endpoints = {
  health: "/admin/api/health",
  login: "/admin/api/login",
  logout: "/admin/api/logout",
  items: "/admin/api/items",
  publishIncremental: "/admin/api/publish/incremental",
  publishFull: "/admin/api/publish/full",
  publishStatus: "/admin/api/publish/status",
};

const request = async (path, options = {}) => {
  const response = await fetch(path, {
    credentials: "same-origin",
    ...options,
  });
  if (!response.ok) {
    const detail = await response.text();
    throw new Error(detail || `${response.status} ${response.statusText}`);
  }
  if (response.status === 204) {
    return null;
  }
  return response.json();
};
```

**Current form/action pattern** (`index.html` lines 27-70):
```html
<section>
  <h2>Publish status</h2>
  <div class="actions">
    <button id="refresh-status" type="button">Refresh status</button>
    <button id="publish-incremental" type="button">Publish incremental</button>
    <button id="publish-full" type="button">Full rebuild</button>
  </div>
  <pre id="publish-status">Not loaded.</pre>
</section>
```

**Admin CSS pattern** (`admin.css` lines 40-65, 78-88):
```css
form,
.actions {
  display: grid;
  gap: 0.75rem;
}

input,
select,
textarea,
button {
  box-sizing: border-box;
  min-height: 2.5rem;
  border: 1px solid #9c8d77;
  border-radius: 0.2rem;
  padding: 0.55rem 0.65rem;
}

.actions {
  grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
}
```

Preserve no-build static assets and same-origin `/admin/api/*` calls. Replace the seed shell with a first-screen status hub, add-new path, finder/edit path, image list controls, history timeline, and diagnostics panel.

### `controller/tests/*` (tests, request-response + privacy + file-I/O)

**Analog:** `controller/tests/seed_content.rs`, `controller/tests/static_admin.rs`, `controller/tests/auth_and_health.rs`, `controller/tests/publisher.rs`

**Admin API integration pattern** (`seed_content.rs` lines 84-124):
```rust
let repository = Arc::new(MemoryCatalogRepository::default());
let media = Arc::new(LocalMediaStore::new(root.path()));
let app = router_with_stores(
    ControllerConfig::for_test(true),
    repository.clone(),
    media.clone(),
);
let create = app
    .clone()
    .oneshot(
        Request::post("/admin/api/items")
            .header(header::AUTHORIZATION, token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"title":"Signed Jedi Card","signer":"Mark Hamill","category":"Cards"}"#))
            .unwrap(),
    )
    .await
    .unwrap();
assert_eq!(create.status(), StatusCode::CREATED);
```

**Privacy assertion pattern** (`seed_content.rs` lines 132-140; `static_admin.rs` lines 6-23):
```rust
for denied in [
    "storageNamespace",
    "bucketName",
    "objectKey",
    "objectstorage",
    "secret bucket photo.jpg",
] {
    assert!(!rendered.contains(denied), "response leaked {denied}");
}
```

**Auth/session pattern** (`auth_and_health.rs` lines 84-121, 136-158):
```rust
let valid = login(&app, "local-test-password").await;
assert_eq!(valid.status(), StatusCode::NO_CONTENT);
let set_cookie = valid.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
assert!(set_cookie.contains("HttpOnly"));
assert!(set_cookie.contains("SameSite=Strict"));
assert!(set_cookie.contains("Secure"));

let cross_origin = app
    .clone()
    .oneshot(
        Request::post("/admin/api/test-mutation")
            .header(header::COOKIE, cookie)
            .header(header::ORIGIN, "https://attacker.example")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();
assert_eq!(cross_origin.status(), StatusCode::FORBIDDEN);
```

Add focused tests for item list/get/search, clearable optional fields, field-level history events, image delete/replace/primary cleanup, pending unpublished changes, release retention, and static-admin source privacy.

### Deploy and Docs Files (config/docs, operator workflow)

**Analog:** `deploy/ansible/roles/autographs_deploy/templates/controller.env.j2`, `deploy/ansible/roles/autographs_deploy/files/Caddyfile`

**Runtime env pattern** (`controller.env.j2` lines 1-4):
```jinja
AUTOGRAPHS_CONTROLLER_DB_PROVIDER={{ autographs_deploy_controller_db_provider }}
AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER={{ autographs_deploy_controller_media_storage_provider }}
AUTOGRAPHS_CONTROLLER_LOCAL_MEDIA_ROOT={{ autographs_deploy_controller_local_media_root }}
OCI_AUTH_MODE=instance_principal
```

**Caddy admin route boundary** (`Caddyfile` lines 8-23):
```caddyfile
@operator path /api/operator /api/operator/*
respond @operator 404

handle /admin/api/* {
	reverse_proxy autographs-controller:8080
}

handle_path /admin/* {
	root * /srv/autographs/admin
	try_files {path} /index.html
	file_server
}
```

Only add deploy variables if retention/status behavior needs runtime configuration. Keep `/admin/api/*` private-controller routing and retired `/api/operator/*` block intact. Update docs to describe the single daily admin path, session/lockout behavior, pending changes, cleanup warnings, retention policy, and operator-run live smoke gates.

## Shared Patterns

### Authentication and CSRF
**Source:** `controller/src/routes.rs` lines 577-586 and `controller/tests/auth_and_health.rs` lines 84-158  
**Apply to:** All admin mutation routes
```rust
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
```

### Redacted DTOs
**Source:** `controller/src/routes.rs` lines 594-650 and `controller/tests/seed_content.rs` lines 132-140  
**Apply to:** Health, diagnostics, item, image, history, cleanup, and publish status responses
```rust
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
```

Do not add `storageNamespace`, `bucketName`, `objectKey`, original filename, Oracle details, OCI identifiers, or secret values to admin/public DTOs.

### Fail-Closed Public Publishing
**Source:** `controller/src/publisher.rs` lines 456-463 and 964-1001  
**Apply to:** Publish batching, retention, and media/history changes
```rust
let result = self
    .build_candidate(repository, media, mode, &release_id, &candidate)
    .await
    .and_then(|_| validate_candidate(&candidate))
    .and_then(|manifest| {
        promote_candidate(&self.root, &release_id)?;
        Ok(manifest)
    });
```

### Observable Cleanup
**Source:** `controller/src/routes.rs` lines 432-447 and `controller/src/media.rs` lines 61-67  
**Apply to:** Image delete/replace and cleanup retry flows
```rust
Err(error) => {
    tracing::error!(%item_id, %image_id, %object_key, ?error, "failed to attach uploaded image metadata");
    let _ = state.media.delete(&object_key).await;
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}
```

For delete/replace, extend this from best-effort rollback to durable cleanup events/warnings when Object Storage deletion fails after metadata work.

### Static Admin Privacy
**Source:** `controller/tests/static_admin.rs` lines 3-23  
**Apply to:** Every static admin HTML/CSS/JS change
```rust
for denied in [
    "AUTOGRAPHS_ADMIN_PASSWORD",
    "AUTOGRAPHS_OPERATOR_API_TOKEN",
    "storageNamespace",
    "bucketName",
    "objectKey",
    "https://objectstorage",
    "OCI_",
    "localStorage",
    "sessionStorage",
] {
    assert!(!source.contains(denied), "static admin source contains {denied}");
}
assert!(!source.replace("/admin/api/", "").contains("/api/"));
```

## No Analog Found

All planned Phase 6 file roles have usable in-repo analogs. The closest gap is field-level edit history, which has no existing implementation but should follow the repository/schema/route/static-admin patterns above plus the research-recommended append-only event table.

## Metadata

**Analog search scope:** `controller/src`, `controller/static-admin`, `controller/tests`, `controller/db`, `deploy/ansible/roles/autographs_deploy`, `docs`  
**Files scanned:** 43 listed files plus targeted `rg` matches  
**Pattern extraction date:** 2026-06-21
