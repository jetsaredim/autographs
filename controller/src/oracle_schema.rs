use std::collections::HashSet;

use oracledb::{Connection, ToDbValue};

use crate::oracle_connection;

const SCHEMA_SQL: &str = include_str!("../db/schema.sql");
const EXPECTED_TABLES: &[&str] = &[
    "AUTOGRAPH_ITEMS",
    "AUTOGRAPH_ITEM_TAGS",
    "AUTOGRAPH_SIGNERS",
    "AUTOGRAPH_ITEM_SIGNERS",
    "AUTOGRAPH_ITEM_CHARACTERS",
    "AUTOGRAPH_ITEM_FRANCHISES",
    "AUTOGRAPH_IMAGES",
    "AUTOGRAPH_PUBLISH_JOBS",
    "AUTOGRAPH_EDIT_EVENTS",
    "AUTOGRAPH_PUBLISH_JOB_EVENTS",
    "AUTOGRAPH_CLEANUP_EVENTS",
    "AUTOGRAPH_PUBLIC_DERIVATIVES",
];
const REQUIRED_COLUMNS: &[(&str, &str)] = &[
    ("AUTOGRAPH_ITEMS", "PUBLICATION_STATUS"),
    ("AUTOGRAPH_ITEMS", "FORMAT"),
    ("AUTOGRAPH_ITEMS", "ORIGIN"),
    ("AUTOGRAPH_ITEMS", "LANGUAGE"),
    ("AUTOGRAPH_ITEMS", "PRODUCT_LINE"),
    ("AUTOGRAPH_ITEMS", "SET_NAME"),
    ("AUTOGRAPH_SIGNERS", "NORMALIZED_NAME"),
    ("AUTOGRAPH_SIGNERS", "WIKIPEDIA_URL"),
    ("AUTOGRAPH_SIGNERS", "IMDB_URL"),
    ("AUTOGRAPH_ITEM_SIGNERS", "ITEM_ROLE"),
    ("AUTOGRAPH_ITEM_SIGNERS", "ITEM_CONTEXT"),
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
const REQUIRED_CHECK_CONSTRAINTS: &[(&str, &str, &[&str], &str)] = &[
    (
        "AUTOGRAPH_EDIT_EVENTS",
        "AUTOGRAPH_EDIT_EVENTS_TYPE_CK",
        &["cleanupChanged"],
        "controller/db/updates/06-03-media-cleanup.sql",
    ),
    (
        "AUTOGRAPH_ITEMS",
        "AUTOGRAPH_ITEMS_FORMAT_CK",
        &["trim(format) is not null"],
        "controller/db/updates/07-01-taxonomy-schema.sql",
    ),
    (
        "AUTOGRAPH_ITEMS",
        "AUTOGRAPH_ITEMS_ORIGIN_CK",
        &["Official", "Custom"],
        "controller/db/updates/07-01-taxonomy-schema.sql",
    ),
    (
        "AUTOGRAPH_ITEMS",
        "AUTOGRAPH_ITEMS_LANGUAGE_CK",
        &["English", "Japanese", "Chinese"],
        "controller/db/updates/07-01-taxonomy-schema.sql",
    ),
    (
        "AUTOGRAPH_SIGNERS",
        "AUTOGRAPH_SIGNERS_NORMALIZED_NAME_CK",
        &["trim(normalized_name) is not null"],
        "controller/db/updates/07-01-taxonomy-schema.sql",
    ),
];
const REQUIRED_UNIQUE_CONSTRAINTS: &[(&str, &str, &[&str], &str)] = &[(
    "AUTOGRAPH_SIGNERS",
    "AUTOGRAPH_SIGNERS_NORMALIZED_NAME_UQ",
    &["NORMALIZED_NAME"],
    "controller/db/updates/07-01-taxonomy-schema.sql",
)];

pub fn ensure_initialized(
    settings: &oracle_connection::OracleConnectionSettings,
) -> Result<(), String> {
    tracing::info!(
        user = settings.user(),
        connect_string = settings.connect_string(),
        "checking Oracle catalog schema state"
    );

    let connection = oracle_connection::connect_with_settings(settings)
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
        let count = query_count(
            connection,
            "select count(*) from user_tab_columns where table_name = :1 and column_name = :2",
            &[table, column],
            &format!("schema column {table}.{column}"),
        )?;
        if count != 1 {
            return Err(format!(
                "Oracle catalog schema is partially initialized; missing expected column {table}.{column}"
            ));
        }
    }

    for (table, constraint, required_texts, update_script) in REQUIRED_CHECK_CONSTRAINTS {
        for required_text in *required_texts {
            let count = query_count(
                connection,
                "select count(*) from user_constraints
                      where table_name = :1
                        and constraint_name = :2
                        and constraint_type = 'C'
                        and status = 'ENABLED'
                        and search_condition_vc like '%' || :3 || '%'",
                &[table, constraint, required_text],
                &format!("schema constraint {table}.{constraint}"),
            )?;
            if count != 1 {
                return Err(format!(
                    "Oracle catalog schema is partially initialized; constraint {table}.{constraint} is missing required value {required_text}; run {update_script} before deploying this controller"
                ));
            }
        }
    }

    for (table, constraint, columns, update_script) in REQUIRED_UNIQUE_CONSTRAINTS {
        let expected_columns = columns.join(",");
        let count = query_count(
            connection,
            "select count(*)
                   from (
                     select c.constraint_name
                       from user_constraints c
                       join user_cons_columns col
                         on col.table_name = c.table_name
                        and col.constraint_name = c.constraint_name
                      where c.table_name = :1
                        and c.constraint_name = :2
                        and c.constraint_type = 'U'
                        and c.status = 'ENABLED'
                      group by c.constraint_name
                     having listagg(col.column_name, ',') within group (order by col.position) = :3
                   )",
            &[table, constraint, &expected_columns],
            &format!("schema unique constraint {table}.{constraint}"),
        )?;
        if count != 1 {
            return Err(format!(
                "Oracle catalog schema is partially initialized; unique constraint {table}.{constraint} is missing expected column set {expected_columns}; run {update_script} before deploying this controller"
            ));
        }
    }

    tracing::info!("Oracle catalog schema preflight passed");
    Ok(())
}

fn query_count(
    connection: &Connection,
    sql: &str,
    params: &[&dyn ToDbValue],
    label: &str,
) -> Result<i64, String> {
    let row = connection
        .query_row(sql, params)
        .map_err(|error| format!("inspect Oracle catalog {label}: {error}"))?;
    row.get(0)
        .map_err(|error| format!("decode Oracle catalog {label}: {error}"))
}

fn existing_autograph_tables(connection: &Connection) -> Result<HashSet<String>, String> {
    let mut rows = connection
        .query(
            "select table_name from user_tables where table_name in (
                'AUTOGRAPH_ITEMS',
                'AUTOGRAPH_ITEM_TAGS',
                'AUTOGRAPH_SIGNERS',
                'AUTOGRAPH_ITEM_SIGNERS',
                'AUTOGRAPH_ITEM_CHARACTERS',
                'AUTOGRAPH_ITEM_FRANCHISES',
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
    use super::{
        EXPECTED_TABLES, REQUIRED_CHECK_CONSTRAINTS, REQUIRED_COLUMNS, REQUIRED_UNIQUE_CONSTRAINTS,
        schema_statements,
    };

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

    #[test]
    fn phase7_schema_includes_signer_and_taxonomy_tables() {
        let statements = schema_statements();

        for table in [
            "create table autograph_signers",
            "create table autograph_item_signers",
            "create table autograph_item_characters",
            "create table autograph_item_franchises",
        ] {
            assert!(
                statements
                    .iter()
                    .any(|statement| statement.starts_with(table)),
                "missing schema statement for {table}"
            );
        }

        let items_statement = statements
            .iter()
            .find(|statement| statement.starts_with("create table autograph_items"))
            .expect("autograph_items statement is present");
        for column in [
            "format varchar2(80)",
            "origin varchar2(24)",
            "language varchar2(40)",
            "product_line varchar2(160)",
            "set_name varchar2(160)",
            "signer varchar2",
            "category varchar2",
        ] {
            assert!(items_statement.contains(column), "missing {column}");
        }
    }

    #[test]
    fn phase7_taxonomy_update_script_is_additive() {
        let script = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/db/updates/07-01-taxonomy-schema.sql"
        ))
        .expect("phase 7 taxonomy update script is readable");
        let lower_script = script.to_ascii_lowercase();

        assert!(script.contains("autograph_items_origin_ck"));
        assert!(script.contains("autograph_items_language_ck"));
        assert!(script.contains("autograph_signers_normalized_name_ck"));
        assert!(script.contains("autograph_signers_normalized_name_uq"));
        assert!(script.contains("duplicate normalized_name values exist"));
        assert!(script.contains("autograph_item_signers"));
        assert!(
            script
                .find("create table autograph_signers")
                .expect("signer table create statement is present")
                < script
                    .find("alter table autograph_signers add constraint autograph_signers_normalized_name_ck")
                    .expect("signer constraint repair statement is present")
        );
        assert!(!lower_script.contains("drop column signer"));
        assert!(!lower_script.contains("drop column category"));
        assert!(!lower_script.contains("drop table autograph_item_tags"));
    }

    #[test]
    fn phase7_preflight_expects_taxonomy_tables_and_columns() {
        for table in [
            "AUTOGRAPH_SIGNERS",
            "AUTOGRAPH_ITEM_SIGNERS",
            "AUTOGRAPH_ITEM_CHARACTERS",
            "AUTOGRAPH_ITEM_FRANCHISES",
        ] {
            assert!(
                EXPECTED_TABLES.contains(&table),
                "missing expected table {table}"
            );
        }

        for required_column in [
            ("AUTOGRAPH_ITEMS", "FORMAT"),
            ("AUTOGRAPH_ITEMS", "ORIGIN"),
            ("AUTOGRAPH_ITEMS", "LANGUAGE"),
            ("AUTOGRAPH_ITEMS", "PRODUCT_LINE"),
            ("AUTOGRAPH_ITEMS", "SET_NAME"),
            ("AUTOGRAPH_SIGNERS", "NORMALIZED_NAME"),
            ("AUTOGRAPH_SIGNERS", "WIKIPEDIA_URL"),
            ("AUTOGRAPH_SIGNERS", "IMDB_URL"),
            ("AUTOGRAPH_ITEM_SIGNERS", "ITEM_ROLE"),
            ("AUTOGRAPH_ITEM_SIGNERS", "ITEM_CONTEXT"),
        ] {
            assert!(
                REQUIRED_COLUMNS.contains(&required_column),
                "missing required column {}.{}",
                required_column.0,
                required_column.1
            );
        }

        for required_constraint in [
            (
                "AUTOGRAPH_ITEMS",
                "AUTOGRAPH_ITEMS_FORMAT_CK",
                &["trim(format) is not null"][..],
                "controller/db/updates/07-01-taxonomy-schema.sql",
            ),
            (
                "AUTOGRAPH_ITEMS",
                "AUTOGRAPH_ITEMS_ORIGIN_CK",
                &["Official", "Custom"][..],
                "controller/db/updates/07-01-taxonomy-schema.sql",
            ),
            (
                "AUTOGRAPH_ITEMS",
                "AUTOGRAPH_ITEMS_LANGUAGE_CK",
                &["English", "Japanese", "Chinese"][..],
                "controller/db/updates/07-01-taxonomy-schema.sql",
            ),
            (
                "AUTOGRAPH_SIGNERS",
                "AUTOGRAPH_SIGNERS_NORMALIZED_NAME_CK",
                &["trim(normalized_name) is not null"][..],
                "controller/db/updates/07-01-taxonomy-schema.sql",
            ),
        ] {
            assert!(
                REQUIRED_CHECK_CONSTRAINTS.contains(&required_constraint),
                "missing required check constraint {}.{}",
                required_constraint.0,
                required_constraint.1
            );
        }

        assert!(
            REQUIRED_UNIQUE_CONSTRAINTS.contains(&(
                "AUTOGRAPH_SIGNERS",
                "AUTOGRAPH_SIGNERS_NORMALIZED_NAME_UQ",
                &["NORMALIZED_NAME"][..],
                "controller/db/updates/07-01-taxonomy-schema.sql",
            )),
            "missing signer normalized-name unique constraint preflight"
        );
    }
}
