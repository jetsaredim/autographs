# Phase 07: Metadata Taxonomy and Public Facets - Pattern Map

**Mapped:** 2026-07-09
**Files analyzed:** 18
**Analogs found:** 15 / 18

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `controller/db/schema.sql` | model | CRUD | `controller/db/schema.sql` | exact |
| `controller/db/updates/07-*.sql` | migration | batch | `controller/db/updates/06-04-publish-snapshot-events.sql` | role-match |
| `controller/src/catalog.rs` | model/service | CRUD | `controller/src/catalog.rs` | exact |
| `controller/src/catalog_admin.rs` | service | request-response | `controller/src/routes/admin_items.rs` | role-match |
| `controller/src/oracle_catalog.rs` | service | CRUD | `controller/src/oracle_catalog.rs` | exact |
| `controller/src/oracle_schema.rs` | config/utility | request-response | `controller/src/oracle_schema.rs` | exact |
| `controller/src/contracts.rs` | model | transform | `controller/src/contracts.rs` | exact |
| `controller/src/publisher.rs` | service | file-I/O/transform | `controller/src/publisher.rs` | exact |
| `controller/src/routes/admin_items.rs` | route | request-response | `controller/src/routes/admin_items.rs` | exact |
| `controller/static-public/assets/browse.js` | component | event-driven | `controller/static-public/assets/browse.js` | exact |
| `controller/static-public/templates/detail.html` | component | transform | `controller/src/publisher.rs` detail rendering helpers | role-match |
| `controller/static-public/assets/site.css` | component | transform | `controller/static-admin/admin.css` and existing public CSS | role-match |
| `controller/static-admin/index.html` | component | request-response | `controller/static-admin/index.html` | exact |
| `controller/static-admin/admin.js` | component | event-driven/request-response | `controller/static-admin/admin.js` | exact |
| `controller/static-admin/admin.css` | component | transform | `controller/static-admin/admin.css` | exact |
| `controller/tests/admin_workflow.rs` | test | request-response | `controller/tests/admin_workflow.rs` | exact |
| `controller/tests/static_contract.rs` | test | file-I/O/transform | `controller/tests/static_contract.rs` | exact |
| `controller/tests/static_admin.rs` | test | transform | `controller/tests/static_admin.rs` | exact |

## Pattern Assignments

### `controller/src/catalog.rs` (model/service, CRUD)

**Analog:** `controller/src/catalog.rs`

**Imports and DTO pattern** (lines 1-12, 22-41):
```rust
use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutographItemInput {
    pub title: String,
    pub signer: String,
    pub description: Option<String>,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
```

**Repository contract pattern** (lines 324-414):
```rust
#[async_trait]
pub trait CatalogRepository: Send + Sync {
    async fn create(&self, input: AutographItemInput) -> Result<AutographItem, String>;
    async fn update(&self, id: Uuid, input: AutographItemUpdate) -> Result<AutographItem, String>;
    async fn get(&self, id: Uuid) -> Result<Option<AutographItem>, String>;
    async fn list(&self) -> Result<Vec<AutographItem>, String>;

    async fn history(&self, _item_id: Uuid) -> Result<Vec<AutographEditEvent>, String> {
        Ok(Vec::new())
    }

    async fn record_event(&self, event: AutographEditEvent) -> Result<AutographEditEvent, String> {
        Ok(event)
    }
}
```

**Update/diff/history pattern** (lines 856-923, 975-1023):
```rust
pub(crate) fn apply_update(item: &mut AutographItem, input: AutographItemUpdate) -> Vec<FieldDiff> {
    let mut field_diffs = Vec::new();
    apply_required_update("title", &mut item.title, input.title, &mut field_diffs);
    apply_required_update("signer", &mut item.signer, input.signer, &mut field_diffs);
    apply_optional_update("description", &mut item.description, input.description, &mut field_diffs);
    apply_required_update("category", &mut item.category, input.category, &mut field_diffs);
    apply_required_update("tags", &mut item.tags, input.tags, &mut field_diffs);
    field_diffs
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
```

**Apply to Phase 7:** add signer profile, signer credit, character/franchise arrays, product line, set name, format, origin, and language as typed domain fields. Extend `apply_update`, required validation, `EditEventKind` if merge/backfill events need distinct history, and memory repository behavior before Oracle.

### `controller/src/oracle_catalog.rs` (service, CRUD)

**Analog:** `controller/src/oracle_catalog.rs`

**Blocking Oracle access pattern** (lines 88-105):
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

**Transactional create/update pattern** (lines 110-159, 162-222):
```rust
async fn create(&self, input: AutographItemInput) -> Result<AutographItem, String> {
    validate_required_fields(&input.title, &input.signer, &input.category)?;
    let id = Uuid::new_v4();
    self.with_connection(move |connection| {
        connection.execute("insert into autograph_items (...)", &[/* binds */])
            .map_err(|error| format!("insert Oracle catalog item: {error}"))?;
        replace_tags(&connection, id, &input.tags)?;
        insert_edit_event(&connection, &event)?;
        connection.commit().map_err(|error| format!("commit Oracle catalog item: {error}"))?;
        load_item(&connection, id)?.ok_or_else(|| "created Oracle item was not found".to_owned())
    }).await
}
```

**Join-table replace/load pattern** (lines 724-740, 892-909):
```rust
fn load_tags(connection: &Connection, id: Uuid) -> Result<Vec<String>, String> {
    let id_text = id.to_string();
    let mut rows = connection
        .query("select tag from autograph_item_tags where item_id = :1 order by tag", &[&id_text])
        .map_err(|error| format!("read Oracle catalog tags: {error}"))?;
    let mut tags = Vec::new();
    for row in &mut rows {
        tags.push(row.map_err(|error| format!("read Oracle catalog tag row: {error}"))?
            .get(0)
            .map_err(|error| format!("read Oracle catalog tag: {error}"))?);
    }
    Ok(tags)
}

fn replace_tags(connection: &Connection, id: Uuid, tags: &[String]) -> Result<(), String> {
    connection.execute("delete from autograph_item_tags where item_id = :1", &[&id_text])
        .map_err(|error| format!("clear Oracle catalog tags: {error}"))?;
    for tag in tags {
        connection.execute("insert into autograph_item_tags (item_id, tag) values (:1, :2)", &[&id_text, tag])
            .map_err(|error| format!("insert Oracle catalog tag: {error}"))?;
    }
    Ok(())
}
```

**Apply to Phase 7:** use the same delete-and-reinsert pattern for item-owned multi-value taxonomy (`characters`, `franchises`, signer credits), but keep reusable signer profile upserts/merges separate and transactional.

### `controller/db/schema.sql` and `controller/db/updates/07-*.sql` (model/migration, CRUD/batch)

**Analog:** `controller/db/schema.sql`

**Table/constraint style** (lines 7-35, 104-114):
```sql
create table autograph_items (
  id varchar2(36) primary key,
  title varchar2(255) not null,
  signer varchar2(255) not null,
  category varchar2(100) not null,
  publication_status varchar2(24) default 'draft' not null,
  created_at timestamp default current_timestamp not null,
  updated_at timestamp default current_timestamp not null,
  constraint autograph_items_publication_ck
    check (publication_status in ('draft', 'published', 'archived'))
);

create table autograph_item_tags (
  item_id varchar2(36) not null,
  tag varchar2(80) not null,
  created_at timestamp default current_timestamp not null,
  constraint autograph_item_tags_pk primary key (item_id, tag),
  constraint autograph_item_tags_item_fk
    foreign key (item_id) references autograph_items(id) on delete cascade
);
```

**Index style** (lines 149-170):
```sql
create index autograph_items_signer_idx on autograph_items(signer);
create index autograph_items_category_idx on autograph_items(category);
create index autograph_items_publication_idx on autograph_items(publication_status);
create index autograph_images_item_order_idx on autograph_images(item_id, sort_order);
```

**Apply to Phase 7:** add normalized signer/profile and item-credit tables with explicit PK/FK/check/unique constraints, add taxonomy join tables/columns, keep legacy `signer` and `category` through Phase 7, and mirror final state in `schema.sql` plus additive/reviewable `controller/db/updates/07-*.sql` scripts.

### `controller/src/oracle_schema.rs` (config/utility, request-response)

**Analog:** `controller/src/oracle_schema.rs`

**Schema preflight pattern** (lines 5-33, 48-83):
```rust
const SCHEMA_SQL: &str = include_str!("../db/schema.sql");
const EXPECTED_TABLES: &[&str] = &[
    "AUTOGRAPH_ITEMS",
    "AUTOGRAPH_ITEM_TAGS",
    "AUTOGRAPH_IMAGES",
];
const REQUIRED_COLUMNS: &[(&str, &str)] = &[
    ("AUTOGRAPH_ITEMS", "PUBLICATION_STATUS"),
    ("AUTOGRAPH_EDIT_EVENTS", "FIELD_DIFFS_JSON"),
];

let missing_tables: Vec<&str> = EXPECTED_TABLES
    .iter()
    .copied()
    .filter(|table| !existing_tables.contains(*table))
    .collect();
if !missing_tables.is_empty() {
    return Err(format!(
        "Oracle catalog schema is partially initialized; missing expected table(s): {}",
        missing_tables.join(", ")
    ));
}
```

**Schema SQL parser test pattern** (lines 189-235):
```rust
#[test]
fn schema_parser_discards_comments_and_statement_terminators() {
    let statements = schema_statements();
    assert!(statements.iter().all(|statement| !statement.ends_with(';')));
    assert!(statements.iter().any(|statement| statement.starts_with("create table autograph_items")));
}
```

**Apply to Phase 7:** add expected signer/taxonomy tables, required columns, and required constraints/checks. Update tests to assert the new `07-*` scripts contain signer tables, FKs, indexes, and no destructive legacy-column removal.

### `controller/src/contracts.rs` (model, transform)

**Analog:** `controller/src/contracts.rs`

**Public schema and camelCase DTO pattern** (lines 1-31, 33-45, 125-155):
```rust
use serde::{Deserialize, Serialize};

pub const PUBLIC_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicGalleryItem {
    pub slug: String,
    pub title: String,
    pub signer: String,
    pub description: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub primary_image: Option<PublicImage>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FacetId {
    Signer,
    Category,
    Tag,
}
```

**Optional field serialization pattern** (lines 188-197):
```rust
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishManifestEntry {
    pub path: String,
    pub byte_size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<ImageVariantName>,
}
```

**Apply to Phase 7:** bump `PUBLIC_SCHEMA_VERSION` to `2`; add signer credit/profile-link DTOs, compact signer text, semantic taxonomy fields, new facet IDs, and optional default-hiding fields intentionally. Keep all public JSON camelCase.

### `controller/src/publisher.rs` (service, file-I/O/transform)

**Analog:** `controller/src/publisher.rs`

**Static asset include and safe publish boundary pattern** (lines 14-45):
```rust
use crate::{
    catalog::{AutographImage, AutographItem, CatalogRepository, PublicationStatus},
    contracts::{
        FacetId, ImageVariantName, PUBLIC_SCHEMA_VERSION, PublicCatalog, PublicDetailField,
        PublicDetailGroup, PublicFacetGroup, PublicFacetOption, PublicFacets, PublicGalleryItem,
        PublicImage, PublicImageVariant, PublicImageVariantParams, PublicItemDetail,
        PublishManifest, PublishManifestEntry,
    },
    derivatives::{DerivativeVariant, generate_derivative},
    media::PrivateMediaStore,
};

const BROWSE_JS: &str = include_str!("../static-public/assets/browse.js");
const DETAIL_TEMPLATE: &str = include_str!("../static-public/templates/detail.html");
const SAFE_PUBLISH_ERROR: &str = "Static publish failed. Check controller logs for details.";
```

**Facet derivation pattern** (lines 312-352):
```rust
fn derive_facets(catalog: &FixtureCatalog) -> Vec<PublicFacetGroup> {
    vec![
        facet_group(FacetId::Signer, "Signer", catalog.items.iter().map(|item| item.signer.clone())),
        facet_group(FacetId::Category, "Category", catalog.items.iter().map(|item| item.category.clone())),
        facet_group(FacetId::Tag, "IP / Genre", catalog.items.iter().flat_map(|item| item.tags.clone())),
    ]
}

fn facet_group(id: FacetId, label: &str, values: impl IntoIterator<Item = String>) -> PublicFacetGroup {
    let options = values.into_iter().collect::<BTreeSet<_>>().into_iter()
        .map(|value| PublicFacetOption { label: value.clone(), value })
        .collect();
    PublicFacetGroup { id, label: label.to_owned(), options }
}
```

**Privacy/fail-closed validation pattern** (lines 1255-1283, 1285-1317):
```rust
fn scan_privacy(root: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    collect_paths(root, &mut files)?;
    for path in files {
        let rendered = /* relative path plus text content */;
        for denied in ["storageNamespace", "bucketName", "objectKey", "objectstorage", "OCI_"] {
            if rendered.contains(denied) {
                return Err(format!("candidate privacy scan rejected denied term: {denied}"));
            }
        }
    }
    Ok(())
}
```

**Detail HTML escaping pattern** (lines 1526-1540, 1640-1670):
```rust
fn detail_html(item: &PublicItemDetail) -> String {
    render_template(
        DETAIL_TEMPLATE,
        &[
            ("item_title", escape_html(&item.title)),
            ("item_signer", escape_html(&item.signer)),
            ("detail_groups", detail_groups(item)),
        ],
    )
}

fn detail_groups(item: &PublicItemDetail) -> String {
    item.detail_groups.iter().map(|group| {
        format!("<section class=\"metadata-group\"><h2>{}</h2><dl>{}</dl></section>",
            escape_html(&group.label), fields)
    }).collect::<String>()
}
```

**Apply to Phase 7:** derive primary and secondary facet groups from schema v2 DTO fields; match multi-value fields by flattening; render signer profile icon links only in escaped/server-rendered detail HTML; extend privacy scans for new migration/source identifiers.

### `controller/src/routes/admin_items.rs` and `controller/src/catalog_admin.rs` (route/service, request-response)

**Analog:** `controller/src/routes/admin_items.rs`

**Auth/error pattern** (lines 18-47, 50-69):
```rust
pub(super) async fn list_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(filter): Query<AdminItemFilter>,
) -> Response {
    if let Err(status) = authorize_admin_session(&state, &Method::GET, &headers) {
        return status.into_response();
    }

    match state.repository.as_ref().list_admin_items(filter).await {
        Ok(items) => Json(summaries).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "failed to list admin catalog items");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

**UUID and not-found pattern** (lines 58-68):
```rust
let Ok(id) = Uuid::parse_str(&id) else {
    return StatusCode::BAD_REQUEST.into_response();
};

match state.repository.get(id).await {
    Ok(Some(item)) => Json(item_response_with_state(&state, item).await).into_response(),
    Ok(None) => StatusCode::NOT_FOUND.into_response(),
    Err(error) => { tracing::error!(item_id = %id, error = %error, "failed to get admin catalog item"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
}
```

**Response DTO pattern** (lines 146-174, 176-225):
```rust
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
```

**Apply to Phase 7:** preserve same-origin session authorization for signer suggestion/search, duplicate warning, profile edit, and merge endpoints. Return safe camelCase admin DTOs; never expose Object Storage/Oracle internals or raw migration data.

### `controller/static-public/assets/browse.js` (component, event-driven)

**Analog:** `controller/static-public/assets/browse.js`

**Fetch/query-state pattern** (lines 13-23, 83-98):
```javascript
const [catalog, facets] = await Promise.all([
  fetch("/data/collection.json").then((response) => response.json()),
  fetch("/data/facets.json").then((response) => response.json()),
]);
const params = new URLSearchParams(window.location.search);
const state = {
  signer: normalizedFilter(params.get("signer")),
  category: normalizedFilter(params.get("category")),
  tag: normalizedFilter(params.get("tag")),
};

const syncUrl = () => {
  const next = new URLSearchParams();
  Object.entries(state).forEach(([key, value]) => {
    if (value) next.set(key, value);
  });
  window.history.pushState({ ...state }, "", `/collection/${query ? `?${query}` : ""}`);
};
```

**Collapsible filter pattern** (lines 99-117, 119-132):
```javascript
const setOpen = (open, persist = false) => {
  panel.classList.toggle("is-collapsed", !open);
  panel.setAttribute("aria-hidden", String(!open));
  panel.inert = !open;
  toggle.setAttribute("aria-expanded", String(open));
  toggle.setAttribute("aria-label", open ? "Close filters" : "Open filters");
  setToggleIcon(open);
  syncToggleHint();
};
```

**Safe DOM rendering pattern** (lines 64-82, 134-149, 168-195, 201-231):
```javascript
const option = (value, label) => {
  const node = document.createElement("option");
  node.value = value;
  node.textContent = label;
  return node;
};

const filtered = catalog.items.filter(
  (item) =>
    (!state.signer || item.signer === state.signer) &&
    (!state.category || item.category === state.category) &&
    (!state.tag || item.tags.includes(state.tag)),
);
```

**Apply to Phase 7:** expand `state` and query params to `signer`, `franchise`, `productLine`, `format`, `language`, `origin`, `role`, and `tag`; keep single-select within groups and AND across groups; match arrays with `includes`; render language labels accessibly while storing semantic values.

### `controller/static-admin/index.html`, `admin.js`, and `admin.css` (components, event-driven/request-response)

**Analogs:** `controller/static-admin/index.html`, `controller/static-admin/admin.js`, `controller/static-admin/admin.css`

**Sectioned editor markup pattern** (`index.html` lines 153-186, 228-290):
```html
<form id="item-form" class="editor-form">
  <section class="form-section" aria-labelledby="identity-title">
    <h3 id="identity-title">Identity</h3>
    <div class="form-grid">
      <div class="field">
        <label for="title">Title</label>
        <input id="title" name="title" required>
      </div>
    </div>
  </section>

  <div id="sticky-save-bar" class="sticky-save-bar">
    <p id="dirty-state">No unsaved client-side edits.</p>
    <div class="inline-actions">
      <button type="submit" class="primary-action">Save item</button>
      <button id="publish-from-editor" type="button" class="secondary-action">Publish changes</button>
    </div>
  </div>
</form>
```

**Same-origin request and JSON error pattern** (`admin.js` lines 160-190):
```javascript
const request = async (path, options = {}) => {
  const { allowAnonymous = false, ...fetchOptions } = options;
  const response = await fetch(path, {
    credentials: "same-origin",
    ...fetchOptions,
  });
  if (response.status === 401) {
    if (!allowAnonymous && !elements.workflowView.hidden) handleAuthFailure();
    const error = new Error(copy.sessionExpired);
    error.status = response.status;
    throw error;
  }
  const contentType = response.headers.get("content-type") || "";
  const body = contentType.includes("application/json") ? await response.json() : await response.text();
  if (!response.ok) { const error = new Error(typeof body === "string" && body ? body : response.statusText); error.status = response.status; error.body = body; throw error; }
  return response.status === 204 ? null : body;
};
```

**Form hydration/payload pattern** (`admin.js` lines 512-550, 647-670):
```javascript
function renderEditor(item = null) {
  state.currentItem = item;
  state.dirty = false;
  elements.itemForm.reset();
  const values = item || { publicationStatus: "draft", tags: [], images: [] };
  for (const [name, value] of Object.entries({
    itemId: values.id || "",
    title: values.title || "",
    signer: values.signer || "",
    category: values.category || "",
    tags: Array.isArray(values.tags) ? values.tags.join(", ") : "",
  })) {
    if (elements.itemForm.elements[name]) elements.itemForm.elements[name].value = value;
  }
}

const formPayload = () => ({
  title: form.elements.title.value.trim(),
  signer: form.elements.signer.value.trim(),
  category: form.elements.category.value.trim(),
  tags: form.elements.tags.value.split(",").map((tag) => tag.trim()).filter(Boolean),
});
```

**CSS system pattern** (`admin.css` lines 1-9, 31-66, 356-380, 486-526):
```css
:root {
  font-family: "IBM Plex Sans", "Segoe UI", system-ui, sans-serif;
  font-size: 16px;
  line-height: 1.5;
  letter-spacing: 0;
  background: #f7f8f6;
  color: #242826;
}

button,
input,
select,
textarea {
  min-height: 44px;
  border: 1px solid #d7ddd5;
  border-radius: 8px;
  padding: 8px 12px;
}

.editor-form,
.filter-toolbar {
  display: grid;
  gap: 16px;
}
```

**Apply to Phase 7:** preserve tabs, editor shell, sticky save bar, no framework/build step, visible labels for all controls, `textContent`/DOM creation for dynamic values, and 44px touch targets. Replace old Identity/Story/Provenance field grouping with Identity, Classification, Details, Publication, Images, History.

### Tests (request-response, file-I/O/transform)

**Analogs:** `controller/tests/admin_workflow.rs`, `controller/tests/static_contract.rs`, `controller/tests/static_admin.rs`

**Admin route verification pattern** (`admin_workflow.rs` lines 128-294):
```rust
#[tokio::test]
async fn admin_can_list_get_update_and_read_history() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let app = router_with_stores(
        ControllerConfig::for_test(false),
        repository,
        Arc::new(LocalMediaStore::new(media_root.path().to_path_buf())),
    );

    let list = app.clone().oneshot(
        Request::builder()
            .method("GET")
            .uri("/admin/api/items?query=mark&tag=jedi&publicationStatus=draft")
            .header(header::COOKIE, admin_cookie(&app).await)
            .header(header::ORIGIN, "https://autographs.example.test")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    assert_redacted(&list_body);
}
```

**Public privacy/static contract pattern** (`static_contract.rs` lines 21-63):
```rust
let generated = generate_split_artifacts(&catalog, "fixture-release");
assert!(generated.artifacts.contains_key("collection.json"));
assert!(generated.artifacts.contains_key("facets.json"));

let rendered = generated.artifacts.iter()
    .map(|(path, bytes)| format!("{path}\n{}", String::from_utf8_lossy(bytes)))
    .collect::<Vec<_>>()
    .join("\n");
for denied in ["storageNamespace", "bucketName", "objectKey", "objectstorage", "OCI_"] {
    assert!(!rendered.contains(denied), "generated public artifacts contain denied value: {denied}");
}
```

**Static admin source contract pattern** (`static_admin.rs` lines 3-23, 25-112, 114-140):
```rust
#[test]
fn static_admin_source_keeps_secrets_private_and_privileged_calls_same_origin() {
    let source = static_admin_source();
    for denied in ["AUTOGRAPHS_ADMIN_PASSWORD", "storageNamespace", "bucketName", "objectKey", "localStorage"] {
        assert!(!source.contains(denied), "static admin source contains {denied}");
    }
    assert!(!source.replace("/admin/api/", "").contains("/api/"));
}

#[test]
fn static_admin_markup_labels_every_form_control() {
    for tag in ["input", "select", "textarea"] {
        /* each control with id has matching <label for="..."> */
    }
}
```

**Apply to Phase 7:** add tests for schema v2 public JSON/facets, signer compact text, no default English/Official detail rows, multi-value facet matching, signer duplicate warnings/merge, migration report safety, and admin label/source privacy.

## Shared Patterns

### Authentication and Same-Origin Admin Access
**Source:** `controller/src/routes/admin_items.rs` lines 18-26 and `controller/static-admin/admin.js` lines 160-190
**Apply to:** all new admin routes and admin JS calls

Use `authorize_admin_session(&state, &Method::..., &headers)` at route entry and `fetch(..., { credentials: "same-origin" })` in static admin JS. Return `401` into the existing login/session-expired path.

### Error Handling
**Source:** `controller/src/routes/admin_items.rs` lines 43-46, 65-68; `controller/src/oracle_catalog.rs` lines 116-155
**Apply to:** routes, Oracle adapter, migration/report tooling

Log server-side details with `tracing::error!`, return generic status codes to admin/public clients, and format Oracle errors with operation context internally. Public publish failures must use the safe copy from `SAFE_PUBLISH_ERROR`.

### Public Privacy Validation
**Source:** `controller/src/publisher.rs` lines 1255-1317; `controller/tests/static_contract.rs` lines 36-59
**Apply to:** publisher, detail templates, public JSON, migration/report artifact exposure

Generated artifacts must fail closed if they contain private storage terms, object keys, image UUIDs, original filenames, OCI/Oracle identifiers, credentials, or unpublished data.

### Dynamic Text Rendering
**Source:** `controller/static-public/assets/browse.js` lines 64-82, 201-205; `controller/static-admin/admin.js` lines 83-105, 358-365
**Apply to:** public filters/cards, admin signer/taxonomy controls, migration report UI

Create DOM nodes and assign `textContent`. Avoid raw `innerHTML` for operator-controlled taxonomy/signer values. Existing inline SVG icon buttons are the exception and use known static path strings.

### Edit History and Pending Publish
**Source:** `controller/src/catalog.rs` lines 233-315, 477-505, 847-966; `controller/tests/admin_workflow.rs` lines 849-931
**Apply to:** taxonomy field updates, signer profile changes, signer merges, migration/backfill record events

Every meaningful private metadata change should create field diffs and preserve save/publish separation. Publishing marks included events through a boundary; do not auto-publish saves.

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `controller/src/taxonomy_migration.rs` or equivalent | utility | batch | No existing operator-reviewed migration report/PLSQL generator exists; copy Oracle error style and static privacy tests instead. |
| `.planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.*` | config/artifact | batch | Temporary committed mapping artifact has no runtime analog; planner should keep it phase-scoped and archive/summarize after verification. |
| `docs/*phase-7-taxonomy-rollout*` or runbook update | docs | batch | Existing docs were not read for pattern extraction; use project documentation habits and rollout decisions from `07-CONTEXT.md`. |

## Metadata

**Analog search scope:** `controller/src/`, `controller/db/`, `controller/db/updates/`, `controller/static-public/`, `controller/static-admin/`, `controller/tests/`
**Files scanned:** 24
**Pattern extraction date:** 2026-07-09
