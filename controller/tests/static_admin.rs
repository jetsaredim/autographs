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

fn static_admin_source() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static-admin");
    ["index.html", "admin.js", "admin.css"]
        .into_iter()
        .map(|name| fs::read_to_string(root.join(name)).expect("read static admin source"))
        .collect::<Vec<_>>()
        .join("\n")
}
