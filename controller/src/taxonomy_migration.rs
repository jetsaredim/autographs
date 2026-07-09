use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyExportRow {
    pub id: String,
    pub title: String,
    pub signer: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackfillMapping {
    #[serde(default)]
    pub values: BTreeMap<String, BackfillTarget>,
    #[serde(default)]
    pub report_only_values: BTreeMap<String, String>,
}

impl BackfillMapping {
    pub fn default_phase7() -> Self {
        let mut values = BTreeMap::new();
        for value in ["Tr", "Tra", "Trading Card"] {
            values.insert(
                value.to_owned(),
                BackfillTarget::new("format", "Trading Card"),
            );
        }
        values.insert("custom".to_owned(), BackfillTarget::new("origin", "Custom"));
        values.insert(
            "Japanese".to_owned(),
            BackfillTarget::new("language", "Japanese"),
        );
        for value in ["Star Wars", "Star Trek", "Monty Python", "Disney"] {
            values.insert(value.to_owned(), BackfillTarget::new("franchise", value));
        }
        for value in [
            "Young Jedi",
            "Force Attax",
            "Lorcana",
            "Magic: The Gathering",
            "Star Wars CCG",
        ] {
            values.insert(value.to_owned(), BackfillTarget::new("productLine", value));
        }
        for value in ["actor", "artist", "author", "game designer", "voice actor"] {
            values.insert(value.to_owned(), BackfillTarget::new("role", value));
        }

        let mut report_only_values = BTreeMap::new();
        report_only_values.insert(
            "possible duplicate physical item".to_owned(),
            "Likely duplicate physical item; review manually before merging records.".to_owned(),
        );

        Self {
            values,
            report_only_values,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackfillTarget {
    pub target_field: String,
    pub target_value: String,
}

impl BackfillTarget {
    fn new(target_field: &str, target_value: &str) -> Self {
        Self {
            target_field: target_field.to_owned(),
            target_value: target_value.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackfillReport {
    pub rows: Vec<BackfillReportRow>,
}

impl BackfillReport {
    pub fn to_markdown(&self) -> String {
        let mut output = String::from("# Phase 7 Taxonomy Backfill Report\n\n");
        for (heading, disposition) in [
            ("Mapped", BackfillDisposition::Mapped),
            ("Needs review", BackfillDisposition::NeedsReview),
            ("Report only", BackfillDisposition::ReportOnly),
        ] {
            output.push_str("## ");
            output.push_str(heading);
            output.push_str("\n\n");
            let rows = self
                .rows
                .iter()
                .filter(|row| row.disposition == disposition)
                .collect::<Vec<_>>();
            if rows.is_empty() {
                output.push_str("- None\n\n");
                continue;
            }
            for row in rows {
                output.push_str("- `");
                output.push_str(&row.item_id);
                output.push_str("` ");
                output.push_str(&row.legacy_source);
                output.push_str(" `");
                output.push_str(&row.legacy_value);
                output.push('`');
                if let (Some(field), Some(value)) = (&row.target_field, &row.target_value) {
                    output.push_str(" -> ");
                    output.push_str(field);
                    output.push_str(" = `");
                    output.push_str(value);
                    output.push('`');
                }
                if let Some(note) = &row.note {
                    output.push_str(" — ");
                    output.push_str(note);
                }
                output.push('\n');
            }
            output.push('\n');
        }
        output
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackfillReportRow {
    pub item_id: String,
    pub title: String,
    pub legacy_source: String,
    pub legacy_value: String,
    pub disposition: BackfillDisposition,
    pub target_field: Option<String>,
    pub target_value: Option<String>,
    pub note: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BackfillDisposition {
    Mapped,
    NeedsReview,
    ReportOnly,
}

pub fn generate_backfill_report(
    mapping: &BackfillMapping,
    rows: &[LegacyExportRow],
) -> BackfillReport {
    let mut report_rows = Vec::new();
    for row in rows {
        classify_value(
            mapping,
            row,
            "legacy category",
            &row.category,
            &mut report_rows,
        );
        for tag in &row.tags {
            classify_value(mapping, row, "legacy tag", tag, &mut report_rows);
        }
    }
    BackfillReport { rows: report_rows }
}

pub fn generate_plsql_script(
    mapping: &BackfillMapping,
    rows: &[LegacyExportRow],
    mapping_source: &str,
) -> String {
    let report = generate_backfill_report(mapping, rows);
    let mut output = String::new();
    output.push_str("-- Phase 7 taxonomy backfill apply script.\n");
    output.push_str("-- Review every statement before applying to live Oracle.\n");
    output.push_str("-- Mapping source: ");
    output.push_str(mapping_source);
    output.push_str("\n\nbegin\n");
    for row in report
        .rows
        .iter()
        .filter(|row| row.disposition == BackfillDisposition::Mapped)
    {
        let Some(field) = row.target_field.as_deref() else {
            continue;
        };
        let Some(value) = row.target_value.as_deref() else {
            continue;
        };
        match field {
            "format" | "origin" | "language" | "productLine" | "setName" => {
                let column = match field {
                    "productLine" => "product_line",
                    "setName" => "set_name",
                    other => other,
                };
                output.push_str("  -- legacy value: ");
                output.push_str(&sql_comment_safe(&row.legacy_value));
                output.push('\n');
                output.push_str("  update autograph_items set ");
                output.push_str(column);
                output.push_str(" = '");
                output.push_str(&sql_literal(value));
                output.push_str("' where id = '");
                output.push_str(&sql_literal(&row.item_id));
                output.push_str("';\n");
            }
            "franchise" => {
                output.push_str("  -- legacy value: ");
                output.push_str(&sql_comment_safe(&row.legacy_value));
                output.push('\n');
                output.push_str(
                    "  insert into autograph_item_franchises (item_id, franchise, sort_order)\n",
                );
                output.push_str("    select '");
                output.push_str(&sql_literal(&row.item_id));
                output.push_str("', '");
                output.push_str(&sql_literal(value));
                output.push_str("', 0 from dual\n");
                output.push_str("    where not exists (select 1 from autograph_item_franchises where item_id = '");
                output.push_str(&sql_literal(&row.item_id));
                output.push_str("' and franchise = '");
                output.push_str(&sql_literal(value));
                output.push_str("');\n");
            }
            "role" => {
                output.push_str("  -- legacy value: ");
                output.push_str(&sql_comment_safe(&row.legacy_value));
                output.push('\n');
                output.push_str("  update autograph_item_signers set item_role = '");
                output.push_str(&sql_literal(value));
                output.push_str("' where item_id = '");
                output.push_str(&sql_literal(&row.item_id));
                output.push_str("' and item_role is null;\n");
            }
            _ => {}
        }
    }
    output.push_str("end;\n/\n");
    output
}

fn classify_value(
    mapping: &BackfillMapping,
    row: &LegacyExportRow,
    legacy_source: &str,
    value: &str,
    report_rows: &mut Vec<BackfillReportRow>,
) {
    if value.trim().is_empty() {
        return;
    }
    if let Some(note) = mapping.report_only_values.get(value) {
        report_rows.push(report_row(
            row,
            legacy_source,
            value,
            BackfillDisposition::ReportOnly,
            None,
            None,
            Some(note.clone()),
        ));
        return;
    }
    if let Some(target) = mapping.values.get(value) {
        report_rows.push(report_row(
            row,
            legacy_source,
            value,
            BackfillDisposition::Mapped,
            Some(target.target_field.clone()),
            Some(target.target_value.clone()),
            None,
        ));
        return;
    }
    report_rows.push(report_row(
        row,
        legacy_source,
        value,
        BackfillDisposition::NeedsReview,
        None,
        None,
        Some("No deterministic mapping exists.".to_owned()),
    ));
}

fn report_row(
    row: &LegacyExportRow,
    legacy_source: &str,
    value: &str,
    disposition: BackfillDisposition,
    target_field: Option<String>,
    target_value: Option<String>,
    note: Option<String>,
) -> BackfillReportRow {
    BackfillReportRow {
        item_id: row.id.clone(),
        title: row.title.clone(),
        legacy_source: legacy_source.to_owned(),
        legacy_value: value.to_owned(),
        disposition,
        target_field,
        target_value,
        note,
    }
}

fn sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

fn sql_comment_safe(value: &str) -> String {
    value.replace('\n', " ").replace('\r', " ")
}
