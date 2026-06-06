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
fn static_admin_source_references_seed_and_publish_contract() {
    let source = static_admin_source();
    for endpoint in [
        "/admin/api/login",
        "/admin/api/logout",
        "/admin/api/health",
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
    assert!(source.contains("Phase 5"));
    assert!(source.contains("Phase 6"));
}

fn static_admin_source() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static-admin");
    ["index.html", "admin.js", "admin.css"]
        .into_iter()
        .map(|name| fs::read_to_string(root.join(name)).expect("read static admin source"))
        .collect::<Vec<_>>()
        .join("\n")
}
