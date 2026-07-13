use std::{fs, path::PathBuf};

#[test]
fn controller_route_tracing_does_not_log_private_or_secret_terms() {
    for path in [
        "controller/src/publisher.rs",
        "controller/src/routes.rs",
        "controller/src/routes/admin_items.rs",
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
