use std::{fs, path::PathBuf};

#[test]
fn controller_route_tracing_does_not_log_private_or_secret_terms() {
    for path in [
        "controller/src/publisher.rs",
        "controller/src/routes.rs",
        "controller/src/routes/admin_items.rs",
        "controller/src/oracle_heartbeat.rs",
    ] {
        let source = read_repo(path);
        for block in tracing_blocks(&source) {
            for denied in [
                "object_key",
                "objectKey",
                "object key",
                "original_filename",
                "originalFilename",
                "file_name",
                "filename",
                "bucket",
                "namespace",
                "secret",
                "token",
                "password",
                "connect_string",
                "ORACLE_DB_CONNECT_STRING",
            ] {
                assert!(
                    !block.contains(denied),
                    "{path} tracing block must not log private or secret terms: {block}"
                );
            }
            if block.contains("private media") || block.contains("private image") {
                assert!(
                    !block.contains("error = %error")
                        && !block.contains("error = ?error")
                        && !block.contains("%error"),
                    "{path} private media tracing block must use safe error categories: {block}"
                );
                assert!(
                    block.contains("error_kind") || !block.contains("failed"),
                    "{path} failed private media tracing block should include an error_kind category: {block}"
                );
            }
            if block.contains("rejected ")
                && block.contains(" request")
                && block.contains("status = %status")
            {
                for denied in ["%id", "%image_id", "%signer_id"] {
                    assert!(
                        !block.contains(denied),
                        "{path} auth/rejection tracing block must not log unvalidated path params: {block}"
                    );
                }
            }
        }
    }
}

#[test]
fn image_replacement_recovery_event_is_correlated_and_fingerprint_only() {
    let source = read_repo("controller/src/routes.rs");
    let event = tracing_blocks(&source)
        .into_iter()
        .find(|block| block.contains("image_replacement_manual_recovery_required"))
        .expect("manual image recovery event");
    for required in [
        "%recovery_id",
        "%item_id",
        "%image_id",
        "transition",
        "outcome",
        "candidate_key_fingerprints",
    ] {
        assert!(
            event.contains(required),
            "recovery event missing {required}"
        );
    }
    for denied in ["object_key", "checksum", "bucket", "namespace"] {
        assert!(!event.contains(denied), "recovery event leaks {denied}");
    }
}

#[test]
fn live_cleanup_inventory_and_evidence_contract_is_discovery_first_and_opaque() {
    let source = read_repo("controller/tests/live_persistence_smoke.rs");
    for required in [
        "autographs-image-recovery-key-v1",
        "select object_key from autograph_images where item_id = :1",
        "select target_object_key from autograph_cleanup_events where item_id = :1",
        "delete_and_verify_cleanup_object",
        "verify_cleanup_rows_absent",
        "cleanup_object_message",
        "cleanup_recovery_message",
        "object_fingerprint=",
        "LIVE_PERSISTENCE_RECOVERY_REQUIRED item_id={item_id} step={step}",
    ] {
        assert!(
            source.contains(required),
            "live cleanup contract missing {required}"
        );
    }
    let cleanup_start = source
        .find("async fn run_cleanup")
        .expect("manual cleanup exists");
    let cleanup_end = source[cleanup_start..]
        .find("async fn audit_image_checksums")
        .map(|offset| cleanup_start + offset)
        .expect("checksum audit follows cleanup evidence paths");
    let guard_paths_start = source
        .find("struct LivePersistenceSmokeCleanup")
        .expect("live cleanup guard exists");
    let guard_paths_end = source[guard_paths_start..]
        .find("fn required")
        .map(|offset| guard_paths_start + offset)
        .expect("required-env helper follows cleanup guard paths");
    let cleanup_source = format!(
        "{}{}",
        &source[cleanup_start..cleanup_end],
        &source[guard_paths_start..guard_paths_end]
    );
    for denied in [
        "cleanup deleting OCI object key",
        "timed out deleting cleanup OCI object: {object_key}",
        "error={error}",
        "object_key={}",
    ] {
        assert!(
            !cleanup_source.contains(denied),
            "live cleanup evidence must not contain {denied}"
        );
    }

    let manual = &source[cleanup_start
        ..source[cleanup_start..]
            .find("fn cleanup_object_fingerprint")
            .map(|offset| cleanup_start + offset)
            .expect("fingerprint helper follows manual cleanup")];
    assert_ordered(
        manual,
        &[
            "discover_cleanup_object_keys",
            "delete_and_verify_cleanup_object",
            "delete_cleanup_item_rows",
            "verify_cleanup_rows_absent",
        ],
        "manual cleanup",
    );

    let guard_start = source
        .find("async fn cleanup_and_verify(&mut self)")
        .expect("live cleanup guard exists");
    let guard_end = source[guard_start..]
        .find("fn report_recovery_required")
        .map(|offset| guard_start + offset)
        .expect("recovery reporter follows cleanup guard");
    let guard = &source[guard_start..guard_end];
    assert_ordered(
        guard,
        &[
            "self.refresh_inventory()",
            "delete_and_verify_cleanup_object",
            "delete_cleanup_item_rows",
            "verify_cleanup_rows_absent",
        ],
        "live cleanup guard",
    );

    let drop_start = source
        .find("impl Drop for LivePersistenceSmokeCleanup")
        .expect("fallback Drop cleanup exists");
    let drop_end = source[drop_start..]
        .find("fn delete_cleanup_item_rows")
        .map(|offset| drop_start + offset)
        .expect("row helper follows Drop cleanup");
    assert_ordered(
        &source[drop_start..drop_end],
        &[
            "self.refresh_inventory()",
            "delete_and_verify_cleanup_object",
            "delete_cleanup_item_rows",
            "verify_cleanup_rows_absent",
        ],
        "fallback Drop cleanup",
    );

    let runbook = read_repo("docs/static-runtime-runbook.md");
    assert!(runbook.contains("consumes but never echoes"));
    assert!(runbook.contains("object fingerprints"));
}

fn assert_ordered(source: &str, needles: &[&str], label: &str) {
    let mut offset = 0;
    for needle in needles {
        let found = source[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("{label} missing {needle}"));
        offset += found + needle.len();
    }
}

fn tracing_blocks(source: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut collecting = false;

    for line in source.lines() {
        if !collecting && line.contains("tracing::") {
            collecting = true;
            current.clear();
        }
        if collecting {
            current.push_str(line);
            current.push('\n');
            if line.trim_end().ends_with(");") {
                blocks.push(current.clone());
                collecting = false;
            }
        }
    }

    blocks
}

fn read_repo(relative: &str) -> String {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("controller parent")
        .to_path_buf();
    fs::read_to_string(repo.join(relative)).expect("read repository artifact")
}
