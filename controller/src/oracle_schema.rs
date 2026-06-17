use std::collections::HashSet;

use oracle::Connection;

const SCHEMA_SQL: &str = include_str!("../db/schema.sql");
const EXPECTED_TABLES: &[&str] = &[
    "AUTOGRAPH_ITEMS",
    "AUTOGRAPH_ITEM_TAGS",
    "AUTOGRAPH_IMAGES",
    "AUTOGRAPH_PUBLISH_JOBS",
    "AUTOGRAPH_PUBLIC_DERIVATIVES",
];
const REQUIRED_COLUMNS: &[(&str, &str)] = &[
    ("AUTOGRAPH_ITEMS", "PUBLICATION_STATUS"),
    ("AUTOGRAPH_IMAGES", "ORIGINAL_FILENAME"),
    ("AUTOGRAPH_PUBLISH_JOBS", "STATUS"),
    ("AUTOGRAPH_PUBLIC_DERIVATIVES", "PUBLIC_PATH"),
];

pub fn ensure_initialized(user: &str, credential: &str, connect_string: &str) -> Result<(), String> {
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
            .map_err(|error| format!("inspect Oracle catalog schema column {table}.{column}: {error}"))?;
        if count != 1 {
            return Err(format!(
                "Oracle catalog schema is partially initialized; missing expected column {table}.{column}"
            ));
        }
    }

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
    for statement in schema_statements() {
        let label = statement.lines().next().unwrap_or("schema statement");
        connection
            .execute(&statement, &[])
            .map_err(|error| format!("apply Oracle catalog schema statement `{label}`: {error}"))?;
    }
    connection
        .commit()
        .map_err(|error| format!("commit Oracle catalog schema bootstrap: {error}"))
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
        assert!(statements.iter().all(|statement| !statement.starts_with("--")));
        assert!(
            statements
                .iter()
                .any(|statement| statement.starts_with("create table autograph_items"))
        );
    }
}
