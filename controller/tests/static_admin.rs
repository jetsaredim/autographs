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
        "Add new item",
        "Find or modify existing items",
        "Identity",
        "Story",
        "Provenance",
        "Publication",
        "Publish changes",
        "Full rebuild",
        "No history recorded yet",
        "No saved items yet",
        "Start with the backlog: add an autograph item, upload its images, save it privately, then publish when the batch is ready.",
        "Run a full rebuild only for repair or structural changes. Continue?",
        "Remove image: Remove this image from the item and queue cleanup of the private original? This cannot be undone from the admin UI.",
    ] {
        assert!(
            source.contains(workflow_copy),
            "static admin source is missing workflow copy {workflow_copy}"
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

fn static_admin_source() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static-admin");
    ["index.html", "admin.js", "admin.css"]
        .into_iter()
        .map(|name| fs::read_to_string(root.join(name)).expect("read static admin source"))
        .collect::<Vec<_>>()
        .join("\n")
}
