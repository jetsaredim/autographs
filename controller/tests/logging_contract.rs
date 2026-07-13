use std::{fs, path::PathBuf};

#[test]
fn controller_route_tracing_does_not_log_private_object_keys() {
    for path in [
        "controller/src/routes.rs",
        "controller/src/routes/admin_items.rs",
    ] {
        let source = read_repo(path);
        for block in tracing_blocks(&source) {
            for denied in ["object_key", "objectKey", "object key"] {
                assert!(
                    !block.contains(denied),
                    "{path} tracing block must not log private media object keys: {block}"
                );
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
