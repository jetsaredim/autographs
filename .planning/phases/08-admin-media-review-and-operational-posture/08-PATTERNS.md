# Phase 08: admin-media-review-and-operational-posture - Pattern Map

**Mapped:** 2026-07-30
**Files analyzed:** 18
**Analogs found:** 18 / 18

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `.github/workflows/weekly-security-scan.yml` | config | event-driven | `.github/workflows/apply-security-updates.yml` | role-match |
| `deploy/ansible/roles/security_patching/tasks/scan.yml` | service | batch | `deploy/ansible/roles/security_patching/tasks/create_issue.yml` | role-match |
| `deploy/ansible/roles/security_patching/tasks/create_issue.yml` | service | request-response | same file | exact |
| `deploy/ansible/roles/security_patching/templates/security-report.md.j2` | utility | transform | same file | exact |
| `docs/security-patching.md` | documentation | transform | same file | exact |
| `docs/phase-08-posture-findings.md` | documentation | transform | `docs/security-review.md` | role-match |
| `.github/workflows/ci.yml` | config | event-driven | `controller/tests/caddy_static_routes.rs` | partial |
| `docs/dns-runbook.md` | documentation | transform | same file | exact |
| `docs/static-runtime-runbook.md` | documentation | transform | `docs/dns-runbook.md` | role-match |
| `deploy/ansible/roles/autographs_deploy/files/Caddyfile` | config | request-response | same file | exact |
| `controller/src/image_adjustments.rs` | utility | transform | `controller/src/derivatives.rs` | role-match |
| `controller/src/derivatives.rs` | utility | transform | same file | exact |
| `controller/src/publisher.rs` | service | batch/file-I/O | same file | exact |
| `controller/src/catalog.rs` | model | CRUD | same file | exact |
| `controller/src/oracle_catalog.rs` | service | CRUD | same file | exact |
| `controller/db/schema.sql` and `controller/db/updates/08-*.sql` | migration | CRUD | `controller/db/updates/06-03-media-cleanup.sql` | role-match |
| `controller/src/routes.rs` / `controller/src/routes/admin_items.rs` | route/controller | request-response | `controller/src/routes.rs` | exact |
| `controller/static-admin/index.html`, `admin.js`, `admin.css` | component | request-response | `controller/static-admin/admin.js` | exact |
| `controller/tests/*media*/publisher/static_contract/static_admin*.rs` | test | CRUD/request-response/batch | `controller/tests/caddy_static_routes.rs`, `controller/tests/media_cleanup.rs` | role-match |

## Pattern Assignments

### Security Patching Workflow and Ansible Scan

**Analogs:** `.github/workflows/apply-security-updates.yml`, `deploy/ansible/roles/security_patching/tasks/scan.yml`, `deploy/ansible/roles/security_patching/tasks/create_issue.yml`

**Current host inventory pattern** (`deploy/ansible/roles/security_patching/tasks/scan.yml` lines 19-31):
```yaml
- name: List available security update advisories # noqa command-instead-of-module
  ansible.builtin.command:
    argv:
      - dnf
      - -q
      - updateinfo
      - list
      - --security
      - --available
  register: security_patching_updateinfo_list
  changed_when: false
  failed_when: security_patching_updateinfo_list.rc not in [0, 100]
```

**Slow loop to replace or make non-critical** (`deploy/ansible/roles/security_patching/tasks/scan.yml` lines 76-93):
```yaml
- name: Resolve advisory IDs
  ansible.builtin.set_fact:
    security_patching_advisory_ids: >-
      {{ security_patching_entries_raw | map(attribute='advisory_id') | unique | sort | list }}

- name: Read advisory details # noqa command-instead-of-module
  ansible.builtin.command:
    argv:
      - dnf
      - -q
      - updateinfo
      - info
      - "{{ item }}"
  register: security_patching_advisory_info
  changed_when: false
  failed_when: false
  loop: "{{ security_patching_advisory_ids }}"
```

**Issue update/create pattern** (`deploy/ansible/roles/security_patching/tasks/create_issue.yml` lines 85-117):
```yaml
- name: Update existing production security update issue
  ansible.builtin.uri:
    url: "{{ security_patching_issues_url }}/{{ security_patching_existing_issue_numbers[0] }}"
    method: PATCH
    headers: "{{ security_patching_github_headers }}"
    body_format: json
    body:
      title: "Production security update report - {{ security_patching_scan_timestamp }}"
      body: "{{ lookup('ansible.builtin.file', security_patching_report_path) }}"
      labels: "{{ security_patching_issue_labels }}"
    status_code: 200

- name: Create production security update issue
  ansible.builtin.uri:
    url: "{{ security_patching_issues_url }}"
    method: POST
    headers: "{{ security_patching_github_headers }}"
    body_format: json
```

Apply this by keeping `security_patching_update_package_specs` as the hidden approval contract and letting enrichment degrade without suppressing issue creation.

### Admin Preview and Adjustment Routes

**Analogs:** `controller/src/routes.rs`, `controller/src/routes/admin_items.rs`

**Imports and route stack** (`controller/src/routes.rs` lines 3-12):
```rust
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
```

**Route registration** (`controller/src/routes.rs` lines 238-249):
```rust
.route("/admin/api/items/{id}/images", post(upload_image))
.route(
    "/admin/api/items/{id}/images/{image_id}/primary",
    post(set_primary_image),
)
.route(
    "/admin/api/items/{id}/images/{image_id}",
    delete(delete_image).put(replace_image),
)
```

**Auth, ID validation, redacted logging** (`controller/src/routes.rs` lines 512-526):
```rust
if let Err(status) = authorize_admin_session(&state, &method, &headers) {
    tracing::warn!(status = %status, "rejected upload image request");
    return status.into_response();
}
let Ok(item_id) = Uuid::parse_str(&id) else {
    tracing::warn!("rejected upload image request with malformed item id");
    return StatusCode::BAD_REQUEST.into_response();
};
```

**Media validation and redacted failure** (`controller/src/routes.rs` lines 563-605):
```rust
if !matches!(
    content_type.as_str(),
    "image/jpeg" | "image/png" | "image/webp"
) || body.len() > MAX_IMAGE_UPLOAD_BYTES
{
    tracing::warn!(%item_id, content_type = %content_type, byte_size = body.len(), "rejected image upload by content type or size");
    return StatusCode::BAD_REQUEST.into_response();
}
if let Err(error) = state.media.write(&object_key, &body).await {
    tracing::error!(%item_id, %image_id, error_kind = classify_media_error(&error), "failed to write uploaded image to private media store");
    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
}
```

Preview routes should copy this shape, return same-origin WebP bytes, set `Cache-Control: no-store`, and avoid browser-visible storage details.

### Static Admin UI

**Analog:** `controller/static-admin/admin.js`

**Endpoint and element registry pattern** (`controller/static-admin/admin.js` lines 1-24, 65-83):
```javascript
const endpoints = {
  images: (id) => `/admin/api/items/${encodeURIComponent(id)}/images`,
  imagePrimary: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}/primary`,
  imageReplace: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}`,
};

const elements = {
  imageGrid: $("#image-grid"),
  imageFiles: $("#image-files"),
  replacementImage: $("#replacement-image"),
  imageMessage: $("#image-message"),
};
```

**Tile rendering pattern** (`controller/static-admin/admin.js` lines 1233-1268):
```javascript
function renderImages(images = [], cleanupWarnings = []) {
  elements.imageGrid.replaceChildren();
  if (!state.currentItem?.id) {
    elements.imageGrid.append(textNode("p", "Save the item before uploading images.", "empty-state"));
    return;
  }
  for (const image of [...images].sort((a, b) => Number(b.isPrimary) - Number(a.isPrimary))) {
    const tile = document.createElement("article");
    tile.className = image.isPrimary ? "image-tile primary-image" : "image-tile";
    tile.append(
      textNode("h4", image.isPrimary ? "Primary image" : "Supporting image"),
      textNode("p", image.altText || "No alt text recorded."),
      textNode("p", `${image.contentType || "image"} - ${image.byteSize || 0} bytes`, "helper-text")
    );
```

**Request/update/error pattern** (`controller/static-admin/admin.js` lines 1452-1467):
```javascript
async function markPrimary(imageId) {
  if (!state.currentItem?.id) {
    return;
  }
  if (!ensureSavedBeforeImageChange()) {
    return;
  }
  try {
    const item = await request(endpoints.imagePrimary(state.currentItem.id, imageId), { method: "POST" });
    state.currentItem = item;
    renderEditor(item);
  } catch (error) {
    if (error.status !== 401) {
      elements.imageMessage.textContent = `Primary image update failed: ${error.message}`;
    }
  }
}
```

Use this for preview thumbnails, focused review state, draft-local adjustment edits, save/cancel/reset, before/after toggle, and split comparison. Keep plain HTML/CSS/JS and no browser storage.

### Adjustment Model and Persistence

**Analogs:** `controller/src/catalog.rs`, `controller/src/oracle_catalog.rs`, `controller/db/schema.sql`

**Image model extension point** (`controller/src/catalog.rs` lines 266-278):
```rust
#[derive(Clone, Debug)]
pub struct AutographImage {
    pub id: Uuid,
    pub object_key: String,
    pub original_filename: String,
    pub content_type: String,
    pub byte_size: usize,
    pub checksum: Option<String>,
    pub etag: Option<String>,
    pub is_primary: bool,
    pub sort_order: i32,
    pub alt_text: Option<String>,
}
```

**Edit history extension point** (`controller/src/catalog.rs` lines 378-389):
```rust
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
```

**Oracle column/select pattern** (`controller/src/oracle_catalog.rs` lines 35-36):
```rust
const IMAGE_SELECT_COLUMNS: &str = "id, object_key, original_filename, content_type, byte_size,
    checksum, etag, is_primary, sort_order, alt_text";
```

Prefer a typed Rust `ImageAdjustment` DTO plus one additive Oracle column such as `adjustment_json` unless planning decides SQL-visible adjustment columns are necessary.

### Derivatives and Publisher Cache

**Analogs:** `controller/src/derivatives.rs`, `controller/src/publisher.rs`

**Derivative imports and source limit** (`controller/src/derivatives.rs` lines 1-5):
```rust
use image::{ExtendedColorType, ImageReader, codecs::webp::WebPEncoder, imageops::FilterType};
use std::io::Cursor;

pub const MAX_SOURCE_BYTES: usize = 20 * 1024 * 1024;
```

**Decode, resize, encode pattern** (`controller/src/derivatives.rs` lines 37-67):
```rust
pub fn generate_derivative(
    source: &[u8],
    variant: DerivativeVariant,
) -> Result<GeneratedDerivative, String> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err("private original exceeds the derivative source limit".to_owned());
    }
    let decoded = ImageReader::new(Cursor::new(source))
        .with_guessed_format()
        .map_err(|error| format!("detect private original format: {error}"))?
        .decode()
        .map_err(|error| format!("decode private original: {error}"))?;
    let resized = decoded.resize(max_width, max_height, FilterType::Lanczos3);
```

**Cache/read/generate/write pattern** (`controller/src/publisher.rs` lines 1042-1108):
```rust
let source_key = match derivative_cache_source_key(image) {
    Some(source_key) => source_key,
    None => {
        let bytes = media.read(&image.object_key).await.inspect_err(|error| {
            tracing::error!(image_id = %image.id, error_kind = private_media_error_kind(error), "failed to read private media for derivative generation");
        })?;
        let fingerprint = derivative_source_fingerprint(&bytes);
        source = Some(bytes);
        DerivativeSourceKey::Legacy(fingerprint)
    }
};
let derivative = generate_derivative(bytes, variant).map_err(|error| {
    tracing::error!(image_id = %image.id, variant = %variant.path_segment(), error = %error, "failed to generate public derivative");
    error
})?;
derivative_cache.write(source_key.cache_key(), &derivative)
```

**Current cache key and public URL fingerprint** (`controller/src/publisher.rs` lines 1232-1241, 1373-1379):
```rust
fn derivative_cache_source_key(image: &AutographImage) -> Option<DerivativeSourceKey> {
    image.checksum.as_deref().map(str::trim).filter(|value| !value.is_empty()).map(|checksum| {
        DerivativeSourceKey::Checksum { checksum: checksum.to_owned(), cache_key: format!("checksum:{checksum}") }
    })
}

fn public_derivative_fingerprint(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest[..8].iter().map(|byte| format!("{byte:02x}")).collect()
}
```

Phase 8 must extend the cache key with canonical adjustment metadata and a transform version, then keep public `/media/*` URLs fingerprinted from final derivative bytes.

### CDN and Cache Contract

**Analogs:** `deploy/ansible/roles/autographs_deploy/files/Caddyfile`, `controller/tests/caddy_static_routes.rs`, `docs/dns-runbook.md`

**Origin headers** (`deploy/ansible/roles/autographs_deploy/files/Caddyfile` lines 11-33):
```caddyfile
handle /admin/api/* {
    header Cache-Control "no-store"
    reverse_proxy autographs-controller:8080
}

@staticMedia path /media/*
header @staticMedia Cache-Control "public, max-age=86400"
@staticDocuments path / /index.html /404.html /collection/* /items/* /architecture/* /data/* /manifest.json
header @staticDocuments Cache-Control "public, max-age=60, must-revalidate"
```

**Contract test style** (`controller/tests/caddy_static_routes.rs` lines 11-27):
```rust
assert!(caddyfile.contains("handle /admin/api/*"));
assert!(caddyfile.contains("reverse_proxy autographs-controller:8080"));
assert!(caddyfile.matches("Cache-Control \"no-store\"").count() >= 3);
assert!(caddyfile.contains("@staticMedia path /media/*"));
assert!(caddyfile.contains("Cache-Control \"public, max-age=86400\""));
assert!(caddyfile.contains("@staticDocuments path / /index.html /404.html"));
assert!(caddyfile.contains("Cache-Control \"public, max-age=60, must-revalidate\""));
```

Apply this to explicit CDN docs/verification: admin and admin API bypass CDN caching, HTML/JSON remain rollback-friendly, assets/media are cacheable, and adjusted media use new fingerprinted paths.

## Shared Patterns

### Authentication
**Source:** `controller/src/routes.rs` and `controller/src/routes/admin_items.rs`
**Apply to:** All admin preview, adjustment, auto-assist, save, reset, and comparison routes.

```rust
if let Err(status) = authorize_admin_session(&state, &method, &headers) {
    tracing::warn!(status = %status, "rejected admin request");
    return status.into_response();
}
```

### Redacted Errors
**Source:** `controller/src/routes.rs` media handlers
**Apply to:** Private preview route, media adjustment preview, upload/replace paths, publisher logging.

```rust
tracing::error!(%item_id, %image_id, error_kind = classify_media_error(&error), "failed to write uploaded image to private media store");
return StatusCode::INTERNAL_SERVER_ERROR.into_response();
```

### Public Privacy
**Source:** `controller/src/publisher.rs`, `controller/tests/static_contract.rs`
**Apply to:** Public JSON/HTML/media paths, adjustment serialization, preview APIs.

Do not expose object keys, bucket names, namespaces, original filenames, private image UUIDs, or unpublished records in public artifacts. Admin preview URLs should be same-origin and opaque enough for browser rendering.

### Verification
**Source:** `AGENTS.md`, `08-RESEARCH.md`
**Apply to:** All Phase 8 plans.

Use `cargo fmt --manifest-path controller/Cargo.toml --check`, `cargo test --manifest-path controller/Cargo.toml`, `cargo check --manifest-path controller/Cargo.toml --features production-persistence`, `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`, `node --check controller/static-admin/admin.js`, and Ansible syntax/lint checks for security patching changes.

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| None | n/a | n/a | Every inferred Phase 8 file has at least a role-match analog in the current Rust/static/Ansible codebase. |

## Metadata

**Analog search scope:** `controller/src`, `controller/static-admin`, `controller/tests`, `controller/db`, `deploy/ansible`, `.github/workflows`, `docs`, `infra/terraform`
**Files scanned:** 80+ source/planning-relevant files, stopping after strong matches per file family
**Pattern extraction date:** 2026-07-30
