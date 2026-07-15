use std::{fs, path::PathBuf};

#[test]
fn static_admin_source_keeps_secrets_private_and_privileged_calls_same_origin() {
    let source = static_admin_source();
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
        assert!(
            !source.contains(denied),
            "static admin source contains {denied}"
        );
    }
    assert!(!source.replace("/admin/api/", "").contains("/api/"));
}

#[test]
fn static_admin_source_references_collection_workflow_contract() {
    let source = static_admin_source();
    for endpoint in [
        "/admin/api/login",
        "/admin/api/logout",
        "/admin/api/health",
        "/admin/api/status",
        "/admin/api/items",
        "/admin/api/publish/incremental",
        "/admin/api/publish/full",
        "/admin/api/publish/status",
    ] {
        assert!(
            source.contains(endpoint),
            "static admin source is missing {endpoint}"
        );
    }
    for workflow_copy in [
        "Admin hub",
        "Add item",
        "Items",
        "Pending changes",
        "Cleanup warnings",
        "Redacted diagnostics",
        "Filters",
        "Identity",
        "Classification",
        "Details",
        "Publication",
        "Publish changes",
        "Full rebuild",
        "No history recorded yet",
        "No saved items yet",
        "Start with the backlog: add an autograph item, upload its images, save it privately, then publish when the batch is ready.",
        "Run a full rebuild after schema or taxonomy migration changes. Continue?",
        "Remove image: Remove this image from the item and queue cleanup of the private original? This cannot be undone from the admin UI.",
    ] {
        assert!(
            source.contains(workflow_copy),
            "static admin source is missing workflow copy {workflow_copy}"
        );
    }
    for workflow_structure in [
        "class=\"tab-button\" data-view=\"add-item-view\"",
        "class=\"tab-button\" data-view=\"items-view\"",
        "<details class=\"status-section\"",
        "<details class=\"filter-panel\" id=\"item-filter-panel\">",
        "id=\"item-list-status\" class=\"status-message\" role=\"status\"",
        "id=\"item-list\" class=\"item-table\"",
        "const loadingState = (message) =>",
        "wrapper.className = \"loading-state\"",
        "icon-action",
        "const iconPaths",
        "const publicationStatusButton",
        "const formatRelativeEpoch",
        "const pendingChangesIcon",
        "status-icon",
        "status-icon-action",
        "status-icon-success",
        "status-icon-warning",
        "iconButton(\"Edit item\", \"edit\"",
        "iconButton(\"View history\", \"history\"",
        "button.setAttribute(\"aria-label\", `Publish status: ${label}`)",
        "button.addEventListener(\"click\", onClick)",
        "const taxonomyCell = (item) =>",
        "cell.className = \"taxonomy-cell\"",
        "const stateCell = (item) =>",
        "cell.className = \"state-cell\"",
        "layout.className = \"state-layout\"",
        "state-copy",
        "state-icons",
        "imageCountLabel",
        "taxonomy-primary",
        "taxonomy-secondary",
        "iconBadge(\"Pending changes\", \"pending\"",
        "iconBadge(\"No pending changes\", \"clean\"",
        "actions.className = \"actions-cell\"",
        "actionGroup.className = \"row-actions\"",
        "itemListStatus: $(\"#item-list-status\")",
        "elements.itemList.setAttribute(\"aria-busy\", \"true\")",
        "elements.itemListStatus.textContent = \"Requesting item summaries...\"",
        "elements.itemList.replaceChildren(loadingState(\"Requesting item summaries...\"))",
        "elements.itemListStatus.textContent = `Preparing ${itemCount} item${itemCount === 1 ? \"\" : \"s\"}...`",
        "elements.itemListStatus.textContent = \"\"",
        "elements.itemListStatus.textContent = \"Item list unavailable.\"",
        "elements.itemList.replaceChildren(loadingState(`Preparing ${itemCount} item${itemCount === 1 ? \"\" : \"s\"}...`))",
        "const nextFrame = () => new Promise((resolve) => requestAnimationFrame(resolve));",
        "await nextFrame();",
        "elements.itemList.removeAttribute(\"aria-busy\")",
        "submit.textContent = \"Signing in...\"",
        "elements.loginMessage.setAttribute(\"role\", \"status\")",
        "elements.loginMessage.setAttribute(\"role\", \"alert\")",
        "submit.disabled = false;",
        "submit.textContent = originalSubmitText;",
        "function compareItems",
        "function updateSort",
        "function openNewItemEditor()",
        "tab.dataset.view === \"add-item-view\"",
        "$(\"#add-another-item\").addEventListener(\"click\", openNewItemEditor)",
        "button.addEventListener(\"click\", () =>",
        "elements.itemFilters.addEventListener(\"submit\", (event) =>",
        "name=\"changes\"",
    ] {
        assert!(
            source.contains(workflow_structure),
            "static admin source is missing workflow structure {workflow_structure}"
        );
    }
    for field in [
        "title",
        "signer",
        "category",
        "tags",
        "publicationStatus",
        "eventName",
        "source",
        "inscription",
        "certificationCompany",
        "estimatedYear",
        "altText",
    ] {
        assert!(
            source.contains(field),
            "static admin source is missing {field}"
        );
    }
    assert!(source.contains("FormData"));
    assert!(!source.to_ascii_lowercase().contains("seed"));
    assert!(
        !source.contains("id=\"item-list\" class=\"item-table\" aria-live"),
        "item list table wrapper should not be the live region while aria-busy changes"
    );
    let loading_start = source
        .find("const loadingState = (message) =>")
        .expect("loadingState helper exists");
    let loading_end = source[loading_start..]
        .find("const nextFrame = ()")
        .map(|offset| loading_start + offset)
        .expect("nextFrame helper follows loadingState");
    let loading_source = &source[loading_start..loading_end];
    assert!(
        !loading_source.contains("role"),
        "visual loading placeholder inside busy item list should not be a live region"
    );
}

#[test]
fn static_admin_item_list_keeps_compact_icon_column_contract() {
    let source = static_admin_source();
    let render_start = source
        .find("async function renderItemList()")
        .expect("item list renderer exists");
    let render_end = source[render_start..]
        .find("const itemTableHead = () =>")
        .map(|offset| render_start + offset)
        .expect("item list header builder follows renderer");
    let renderer = &source[render_start..render_end];
    let header_end = source[render_end..]
        .find("function sortLabel")
        .map(|offset| render_end + offset)
        .expect("sort label follows header builder");
    let header_builder = &source[render_end..header_end];
    let state_start = source
        .find("const stateCell = (item) =>")
        .expect("state cell builder exists");
    let state_end = source[state_start..]
        .find("const taxonomyCell = (item) =>")
        .map(|offset| state_start + offset)
        .expect("taxonomy cell follows state cell");
    let state_builder = &source[state_start..state_end];

    assert!(
        header_builder.contains("{ label: \"Franchise / Product\" }"),
        "item list header should keep the stacked taxonomy column"
    );
    assert!(
        header_builder.contains("{ label: \"State\" }"),
        "item list header should group status, images, changes, and updated time"
    );
    assert!(
        !header_builder.contains("{ label: \"Format\", key: \"format\" }"),
        "item list header should not spend a column on format"
    );
    for removed_header in [
        "{ label: \"Status\" }",
        "{ label: \"Images\" }",
        "{ label: \"Changes\" }",
        "{ label: \"Updated\" }",
    ] {
        assert!(
            !header_builder.contains(removed_header),
            "item list header should not keep separate state fragment {removed_header}"
        );
    }

    let renderer_fragments = [
        "row.insertBefore(taxonomyCell(item), row.children[2]);",
        "row.append(stateCell(item));",
        "actions.className = \"actions-cell\"",
        "actionGroup.className = \"row-actions\"",
        "iconButton(\"Edit item\", \"edit\"",
        "iconButton(\"View history\", \"history\"",
        "actions.append(actionGroup);",
    ];
    let mut previous_position = 0;
    for fragment in renderer_fragments {
        let relative_position = renderer[previous_position..]
            .find(fragment)
            .unwrap_or_else(|| panic!("item list renderer is missing ordered fragment {fragment}"));
        previous_position += relative_position;
    }

    let state_icon_fragments = [
        "pendingChangesIcon(item.hasPendingChanges)",
        "publicationStatusButton(item.publicationStatus, () => setView(\"publish-view\"))",
    ];
    let mut previous_position = 0;
    for fragment in state_icon_fragments {
        let relative_position = state_builder[previous_position..]
            .find(fragment)
            .unwrap_or_else(|| panic!("state cell icon cluster is missing ordered fragment {fragment}"));
        previous_position += relative_position;
    }

    for accessibility_fragment in [
        "badge.setAttribute(\"role\", \"img\");",
        "badge.setAttribute(\"aria-label\", label);",
        "badge.title = label;",
        "button.setAttribute(\"aria-label\", `Publish status: ${label}`);",
        "button.title = `Publish status: ${label}`;",
        "return { label: \"Draft\", icon: \"draft\", tone: \"status-icon-neutral\" };",
        "return { label: \"Archived\", icon: \"archived\", tone: \"status-icon-muted\" };",
        "{ label: \"Franchise / Product\" }",
        "{ label: \"State\" }",
        "layout.append(copy, icons);",
        "cell.append(layout);",
        "copy.title = formatEpoch(item.updatedAtEpochSeconds);",
        "pendingChangesIcon(item.hasPendingChanges)",
        "publicationStatusButton(item.publicationStatus, () => setView(\"publish-view\"))",
    ] {
        assert!(
            source.contains(accessibility_fragment),
            "compact item-list icons are missing accessibility contract {accessibility_fragment}"
        );
    }
}

#[test]
fn static_admin_source_references_taxonomy_editor_contract() {
    let source = static_admin_source();
    let html = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("static-admin")
            .join("index.html"),
    )
    .expect("read static admin markup");

    let mut previous = 0;
    for heading in [
        ">Identity<",
        ">Classification<",
        ">Details<",
        ">Publication<",
        ">Images<",
        ">History<",
    ] {
        let position = html
            .find(heading)
            .unwrap_or_else(|| panic!("static admin markup is missing heading {heading}"));
        assert!(
            position >= previous,
            "static admin heading {heading} appears out of order"
        );
        previous = position;
    }

    for expected in [
        "signer-rows",
        "signer-warning-summary",
        "signer-merge-panel",
        "classification-section",
        "details-section",
        "Possible duplicate signer. Review the existing profile before saving a new signer.",
        "Type a name to create a new signer, or choose an existing signer.",
        "Wikipedia and IMDb links are optional and appear only on public item detail pages.",
        "Custom item",
        "Use loose tags only for details that do not fit signer, franchise, product line, format, origin, language, role, or set.",
        "renderSignerRows",
        "loadSignerSuggestions",
        "renderDuplicateWarnings",
        "renderTaxonomySuggestions",
        "taxonomyPayload",
        "mergeSignerProfiles",
        "signerSuggestions",
        "credentials: \"same-origin\"",
    ] {
        assert!(
            source.contains(expected),
            "static admin source is missing taxonomy editor contract {expected}"
        );
    }

    for payload_field in [
        "signerCredits",
        "characters",
        "franchises",
        "productLine",
        "setName",
        "format",
        "origin",
        "language",
        "tags",
    ] {
        assert!(
            source.contains(payload_field),
            "static admin form payload is missing {payload_field}"
        );
    }
}

#[test]
fn static_admin_signer_payload_uses_row_scoped_fields_and_item_role_only() {
    let source = static_admin_source();
    for expected in [
        "row.querySelector(`[data-signer-field=\"${field}\"]`)",
        "delete row.dataset.signerId",
        "setExistingSignerProfileControls(row)",
        "input.disabled = disabled",
        "selectedSignerName = selected.profile.displayName",
        "itemRole: value(\"role\")",
        "wikipediaUrl: value(\"wikipedia\")",
        "imdbUrl: value(\"imdb\")",
        "...new Set(",
    ] {
        assert!(
            source.contains(expected),
            "static admin signer/taxonomy payload should include {expected}"
        );
    }
    assert!(
        !source.contains("defaultRole: value(\"role\")"),
        "item-level signer role must not mutate the reusable signer default role"
    );
}

#[test]
fn static_admin_markup_labels_every_form_control() {
    let html = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("static-admin")
            .join("index.html"),
    )
    .expect("read static admin markup");

    for tag in ["input", "select", "textarea"] {
        for control in html.match_indices(&format!("<{tag}")) {
            let element = &html[control.0..];
            let end = element.find('>').expect("control has closing bracket");
            let element = &element[..end];
            let Some(id_start) = element.find("id=\"") else {
                panic!("static admin {tag} missing id: {element}");
            };
            let id_value = &element[id_start + 4..];
            let id_end = id_value.find('"').expect("id has closing quote");
            let id = &id_value[..id_end];
            assert!(
                html.contains(&format!("<label for=\"{id}\"")),
                "static admin {tag} #{id} is missing a visible matching label"
            );
        }
    }
}

#[test]
fn static_admin_save_captures_image_selection_before_editor_reset() {
    let source = static_admin_source();
    for expected in [
        "const selectedFiles = Array.from(elements.imageFiles.files);",
        "const selectedAltText = elements.itemForm.elements.altText.value.trim();",
        "state.dirty = false;",
        "await uploadImages(item.id, selectedFiles, selectedAltText, { allowDirty: true });",
        "async function uploadImages(",
        "files = Array.from(elements.imageFiles.files)",
        "options = {}",
        "if (!options.allowDirty && !ensureSavedBeforeImageChange())",
        "Image upload failed:",
        "upload.append(\"altText\", altText);",
    ] {
        assert!(
            source.contains(expected),
            "static admin source should preserve selected image upload state with {expected}"
        );
    }
}

#[test]
fn static_admin_publish_actions_require_saved_changes_in_shared_path() {
    let source = static_admin_source();
    for expected in [
        "function ensureSavedBeforePublish()",
        "if (!state.dirty)",
        "setView(\"add-item-view\");",
        "function publishFromEditor()",
        "async function publishChanges(mode = \"incremental\")",
        "if (!ensureSavedBeforePublish())",
        "Save item before publishing these changes.",
        "elements.globalMessage.focus();",
        "$(\"#publish-from-editor\").addEventListener(\"click\", publishFromEditor);",
        "$(\"#publish-incremental\").addEventListener(\"click\", () => publishChanges(\"incremental\"));",
        "$(\"#publish-full\").addEventListener(\"click\", () => publishChanges(\"full\"));",
        "elements.publishFromEditor.setAttribute(\"aria-disabled\", \"true\");",
    ] {
        assert!(
            source.contains(expected),
            "static admin source should block stale publishes through the shared path with {expected}"
        );
    }

    let publish_start = source
        .find("async function publishChanges(mode = \"incremental\")")
        .expect("publishChanges exists");
    let publish_source = &source[publish_start..];
    let guard_position = publish_source
        .find("if (!ensureSavedBeforePublish())")
        .expect("publishChanges checks dirty editor state");
    let full_confirm_position = publish_source
        .find("if (mode === \"full\"")
        .expect("publishChanges retains full rebuild confirmation");
    assert!(
        guard_position < full_confirm_position,
        "publishChanges should block dirty editor state before prompting for a full rebuild"
    );

    let editor_start = source
        .find("function publishFromEditor()")
        .expect("publishFromEditor exists");
    let editor_source = &source[editor_start
        ..source[editor_start..]
            .find("\n}\n\nasync function bootstrapSession")
            .map(|end| editor_start + end)
            .expect("publishFromEditor body ends before bootstrapSession")];
    assert!(
        !editor_source.contains("state.dirty"),
        "publishFromEditor should delegate dirty-state protection to publishChanges"
    );
}

#[test]
fn static_admin_image_actions_require_saved_changes_in_shared_path() {
    let source = static_admin_source();
    for expected in [
        "const uploadOnlyFieldNames = new Set([\"images\", \"replacementImage\", \"altText\"]);",
        "const markDirty = (event) =>",
        "if (uploadOnlyFieldNames.has(event?.target?.name))",
        "function ensureSavedBeforeImageChange()",
        "Save item before changing images.",
        "async function uploadImages(",
        "if (!options.allowDirty && !ensureSavedBeforeImageChange())",
        "async function markPrimary(imageId)",
        "async function removeImage(imageId)",
        "async function replaceImage(imageId)",
        "async function retryCleanup(imageId)",
    ] {
        assert!(
            source.contains(expected),
            "static admin source should guard image actions with {expected}"
        );
    }
}

#[test]
fn static_admin_bootstraps_existing_sessions_without_expired_copy() {
    let source = static_admin_source();
    for expected in [
        "const { allowAnonymous = false, ...fetchOptions } = options;",
        "if (!allowAnonymous && !elements.workflowView.hidden)",
        "const adminLoginPath = \"/admin/login\";",
        "const adminRootPath = \"/admin/\";",
        "const publicHomePath = \"/\";",
        "url.searchParams.set(\"next\", next);",
        "new URLSearchParams(window.location.search).get(\"next\")",
        "async function bootstrapSession()",
        "await renderHub({ allowAnonymous: true });",
        "window.location.replace(loginRedirectUrl());",
        "window.location.replace(nextDestination());",
        "window.location.replace(publicHomePath);",
        "elements.loginMessage.textContent = error.status === 429 ? copy.lockout : \"Login failed.\";",
        "const form = event.currentTarget;",
        "showWorkflow();",
        "showLogin();",
        "bootstrapSession();",
    ] {
        assert!(
            source.contains(expected),
            "static admin source should keep initial anonymous bootstrap distinct with {expected}"
        );
    }
}

#[test]
fn static_admin_css_keeps_hidden_sections_hidden() {
    let source = static_admin_source();
    assert!(
        source.contains("[hidden] {\n  display: none !important;\n}"),
        "static admin CSS should explicitly hide hidden sections"
    );
}

#[test]
fn static_admin_taxonomy_styles_and_accessibility_states_are_present() {
    let source = static_admin_source();
    for selector in [
        ".signer-row",
        ".signer-row-grid",
        ".warning-summary",
        ".merge-panel",
        ".token-editor",
        ".taxonomy-suggestions",
        ".loose-tags-field",
        "#classification-section",
        "#details-section",
    ] {
        assert!(
            source.contains(selector),
            "static admin source is missing taxonomy style selector {selector}"
        );
    }

    for expected in [
        "Merge signer: Merge these signer profiles and update linked items? Review the target profile first; this cannot be undone from the admin UI.",
        "role=\"status\"",
        "role=\"alert\"",
        "aria-expanded",
        "aria-label",
        "focus-visible",
        "outline: 2px solid #25636a;",
        ".loading-state",
        ".status-success {\n  color: #25636a;\n}",
        ".status-icon-success {\n  color: #25636a;\n}",
        ".status-icon-action {\n  width: 36px;",
        ".icon-action {\n  width: 36px;",
        "#9a6700",
        "#b42318",
    ] {
        assert!(
            source.contains(expected),
            "static admin source is missing accessibility/style contract {expected}"
        );
    }

    for denied in [
        "linear-gradient",
        "radial-gradient",
        "localStorage",
        "sessionStorage",
    ] {
        assert!(
            !source.contains(denied),
            "static admin source should not contain {denied}"
        );
    }
}

#[test]
fn static_admin_login_keeps_expired_sessions_in_place_when_root_redirects_back_home() {
    let source = static_admin_source();
    for expected in [
        "const next = nextDestination();",
        "if (window.location.pathname === adminRootPath && next === adminRootPath)",
        "next.includes(\"\\\\\")",
        "new URL(next, window.location.origin)",
        "showWorkflow();",
        "window.location.replace(next);",
    ] {
        assert!(
            source.contains(expected),
            "static admin source should restore an expired session in place with {expected}"
        );
    }
}

fn static_admin_source() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static-admin");
    ["index.html", "admin.js", "admin.css"]
        .into_iter()
        .map(|name| fs::read_to_string(root.join(name)).expect("read static admin source"))
        .collect::<Vec<_>>()
        .join("\n")
}
