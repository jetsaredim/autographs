use std::collections::HashSet;

use oracle::Connection;

const SCHEMA_SQL: &str = include_str!("../db/schema.sql");
const EXPECTED_TABLES: &[&str] = &[
    "AUTOGRAPH_ITEMS",
    "AUTOGRAPH_ITEM_TAGS",
    "AUTOGRAPH_IMAGES",
    "AUTOGRAPH_PUBLISH_JOBS",
    "AUTOGRAPH_EDIT_EVENTS",
    "AUTOGRAPH_PUBLISH_JOB_EVENTS",
    "AUTOGRAPH_CLEANUP_EVENTS",
    "AUTOGRAPH_PUBLIC_DERIVATIVES",
];
const REQUIRED_COLUMNS: &[(&str, &str)] = &[
    ("AUTOGRAPH_ITEMS", "PUBLICATION_STATUS"),
    ("AUTOGRAPH_IMAGES", "ORIGINAL_FILENAME"),
    ("AUTOGRAPH_PUBLISH_JOBS", "STATUS"),
    ("AUTOGRAPH_PUBLISH_JOBS", "SNAPSHOT_EVENT_COUNT"),
    ("AUTOGRAPH_EDIT_EVENTS", "EVENT_TYPE"),
    ("AUTOGRAPH_EDIT_EVENTS", "FIELD_DIFFS_JSON"),
    ("AUTOGRAPH_PUBLISH_JOB_EVENTS", "EDIT_EVENT_ID"),
    ("AUTOGRAPH_CLEANUP_EVENTS", "ADMIN_MESSAGE"),
    ("AUTOGRAPH_CLEANUP_EVENTS", "TARGET_OBJECT_KEY"),
    ("AUTOGRAPH_CLEANUP_EVENTS", "RESOLVED_AT"),
    ("AUTOGRAPH_PUBLIC_DERIVATIVES", "PUBLIC_PATH"),
];
const REQUIRED_CHECK_CONSTRAINTS: &[(&str, &str, &str)] = &[(
    "AUTOGRAPH_EDIT_EVENTS",
    "AUTOGRAPH_EDIT_EVENTS_TYPE_CK",
    "cleanupChanged",
)];

pub fn ensure_initialized(
    user: &str,
    credential: &str,
    connect_string: &str,
) -> Result<(), String> {
    tracing::info!(%user, %connect_string, "checking Oracle catalog schema state");

    let connection = Connection::connect(user, credential, connect_string)
        .map_err(|error| format!("connect to Oracle catalog for schema bootstrap: {error}"))?;

    ensure_initialized_on_connection(&connection)
}

fn ensure_initialized_on_connection(connection: &Connection) -> Result<(), String> {
    let existing_tables = existing_autograph_tables(connection)?;
    if existing_tables.is_empty() {
        apply_schema(connection)?;
        return Ok(());
    }

    let missing_tables: Vec<&str> = EXPECTED_TABLES
        .iter()
        .copied()
        .filter(|table| !existing_tables.contains(*table))
        .collect();
    if !missing_tables.is_empty() {
        tracing::error!(missing_tables = ?missing_tables, "Oracle catalog schema is partially initialized");

        return Err(format!(
            "Oracle catalog schema is partially initialized; missing expected table(s): {}",
            missing_tables.join(", ")
        ));
    }

    for (table, column) in REQUIRED_COLUMNS {
        let count: i64 = connection
            .query_row_as(
                "select count(*) from user_tab_columns where table_name = :1 and column_name = :2",
                &[table, column],
            )
            .map_err(|error| {
                format!("inspect Oracle catalog schema column {table}.{column}: {error}")
            })?;
        if count != 1 {
            return Err(format!(
                "Oracle catalog schema is partially initialized; missing expected column {table}.{column}"
            ));
        }
    }

    for (table, constraint, required_text) in REQUIRED_CHECK_CONSTRAINTS {
        let count: i64 = connection
            .query_row_as(
                "select count(*) from user_constraints
                  where table_name = :1
                    and constraint_name = :2
                    and constraint_type = 'C'
                    and status = 'ENABLED'
                    and search_condition_vc like '%' || :3 || '%'",
                &[table, constraint, required_text],
            )
            .map_err(|error| {
                format!("inspect Oracle catalog schema constraint {table}.{constraint}: {error}")
            })?;
        if count != 1 {
            return Err(format!(
                "Oracle catalog schema is partially initialized; constraint {table}.{constraint} is missing required value {required_text}; run controller/db/updates/06-03-media-cleanup.sql before deploying this controller"
            ));
        }
    }

    tracing::info!("Oracle catalog schema preflight passed");
    Ok(())
}

fn existing_autograph_tables(connection: &Connection) -> Result<HashSet<String>, String> {
    let mut rows = connection
        .query(
            "select table_name from user_tables where table_name in (
                'AUTOGRAPH_ITEMS',
                'AUTOGRAPH_ITEM_TAGS',
                'AUTOGRAPH_IMAGES',
                'AUTOGRAPH_PUBLISH_JOBS',
                'AUTOGRAPH_EDIT_EVENTS',
                'AUTOGRAPH_PUBLISH_JOB_EVENTS',
                'AUTOGRAPH_CLEANUP_EVENTS',
                'AUTOGRAPH_PUBLIC_DERIVATIVES'
            )",
            &[],
        )
        .map_err(|error| format!("inspect Oracle catalog schema tables: {error}"))?;
    let mut tables = HashSet::new();
    for row in &mut rows {
        let table: String = row
            .map_err(|error| format!("read Oracle catalog schema table row: {error}"))?
            .get(0)
            .map_err(|error| format!("read Oracle catalog schema table name: {error}"))?;
        tables.insert(table);
    }
    Ok(tables)
}

fn apply_schema(connection: &Connection) -> Result<(), String> {
    let statements = schema_statements();
    tracing::info!(
        statement_count = statements.len(),
        "applying Oracle catalog schema"
    );

    for statement in statements {
        let label = statement.lines().next().unwrap_or("schema statement");
        tracing::debug!(%label, "applying Oracle catalog schema statement");
        connection
            .execute(&statement, &[])
            .map_err(|error| format!("apply Oracle catalog schema statement `{label}`: {error}"))?;
    }
    connection
        .commit()
        .map_err(|error| format!("commit Oracle catalog schema bootstrap: {error}"))?;

    tracing::info!("committed Oracle catalog schema bootstrap");
    Ok(())
}

fn schema_statements() -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();

    for raw_line in SCHEMA_SQL.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
        if line.ends_with(';') {
            let statement = current.trim().trim_end_matches(';').trim().to_owned();
            if !statement.is_empty() {
                statements.push(statement);
            }
            current.clear();
        }
    }

    let trailing = current.trim();
    if !trailing.is_empty() {
        statements.push(trailing.to_owned());
    }

    statements
}

#[cfg(test)]
mod tests {
    use super::schema_statements;

    #[test]
    fn schema_parser_discards_comments_and_statement_terminators() {
        let statements = schema_statements();
        assert!(statements.iter().all(|statement| !statement.ends_with(';')));
        assert!(
            statements
                .iter()
                .all(|statement| !statement.starts_with("--"))
        );
        assert!(
            statements
                .iter()
                .any(|statement| statement.starts_with("create table autograph_items"))
        );
        assert!(
            statements
                .iter()
                .any(|statement| statement.starts_with("create table autograph_edit_events"))
        );
        assert!(
            statements
                .iter()
                .any(|statement| statement.starts_with("create table autograph_publish_job_events"))
        );
        assert!(
            statements
                .iter()
                .any(|statement| statement.starts_with("create table autograph_cleanup_events"))
        );
    }

    #[test]
    fn publish_snapshot_update_script_creates_event_mapping_table() {
        let script = include_str!("../db/updates/06-04-publish-snapshot-events.sql");

        assert!(script.contains("AUTOGRAPH_PUBLISH_JOB_EVENTS"));
        assert!(script.contains("create table autograph_publish_job_events"));
        assert!(script.contains("publish_job_id varchar2(36) not null"));
        assert!(script.contains("edit_event_id varchar2(36) not null"));
        assert!(script.contains("references autograph_publish_jobs(id) on delete cascade"));
        assert!(script.contains("references autograph_edit_events(id) on delete cascade"));
        assert!(script.contains("create index autograph_publish_job_events_event_idx"));
    }
}
