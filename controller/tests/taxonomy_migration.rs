use autographs_controller::taxonomy_migration::{
    BackfillDisposition, BackfillMapping, LegacyExportRow, generate_backfill_report,
    generate_plsql_script,
};

fn sample_rows() -> Vec<LegacyExportRow> {
    vec![
        LegacyExportRow {
            id: "11111111-1111-4111-8111-111111111111".to_owned(),
            title: "Custom Japanese Jedi".to_owned(),
            signer: "Mark Hamill".to_owned(),
            category: "Tr".to_owned(),
            tags: vec![
                "custom".to_owned(),
                "Japanese".to_owned(),
                "Star Wars".to_owned(),
                "Young Jedi".to_owned(),
                "actor".to_owned(),
            ],
        },
        LegacyExportRow {
            id: "22222222-2222-4222-8222-222222222222".to_owned(),
            title: "Likely duplicate multi-signer workaround".to_owned(),
            signer: "Mark Hamill / Carrie Fisher".to_owned(),
            category: "Trading Card".to_owned(),
            tags: vec!["possible duplicate physical item".to_owned()],
        },
        LegacyExportRow {
            id: "33333333-3333-4333-8333-333333333333".to_owned(),
            title: "Unknown legacy value".to_owned(),
            signer: "Unknown Signer".to_owned(),
            category: "Mystery".to_owned(),
            tags: vec!["unclear".to_owned()],
        },
    ]
}

#[test]
fn backfill_report_classifies_mapped_review_and_report_only_rows() {
    let mapping = BackfillMapping::default_phase7();
    let report = generate_backfill_report(&mapping, &sample_rows());
    let rendered = report.to_markdown();

    assert!(rendered.contains("## Mapped"));
    assert!(rendered.contains("## Needs review"));
    assert!(rendered.contains("## Report only"));
    assert!(report.rows.iter().any(|row| {
        row.legacy_value == "custom"
            && row.target_field.as_deref() == Some("origin")
            && row.target_value.as_deref() == Some("Custom")
            && row.disposition == BackfillDisposition::Mapped
    }));
    assert!(report.rows.iter().any(|row| {
        row.legacy_value == "possible duplicate physical item"
            && row.disposition == BackfillDisposition::ReportOnly
    }));
}

#[test]
fn generated_plsql_maps_known_values_without_private_identifiers() {
    let mapping = BackfillMapping::default_phase7();
    let script = generate_plsql_script(
        &mapping,
        &sample_rows(),
        ".planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json",
    );

    assert!(script.contains("origin = 'Custom'"));
    assert!(script.contains("format = 'Trading Card'"));
    assert!(script.contains("legacy value: Tr"));
    assert!(script.contains("legacy value: Tra") || script.contains("legacy value: Trading Card"));
    assert!(script.contains("merge into autograph_signers signer"));
    assert!(script.contains(
        "insert into autograph_item_signers (item_id, signer_id, sort_order, item_role)"
    ));
    assert!(script.contains("'Mark Hamill' display_name"));
    assert!(script.contains("'mark hamill' normalized_name"));
    assert!(script.contains("signer.normalized_name = 'mark hamill'"));
    assert!(
        script.contains("select '11111111-1111-4111-8111-111111111111', signer.id, 0, 'actor'")
    );
    assert!(!script.contains("'Mark Hamill / Carrie Fisher' display_name"));
    assert!(!script.contains("possible duplicate physical item"));

    for forbidden in [
        "objectstorage",
        "bucket",
        "object_key",
        "oracle://",
        "password",
        "secret",
        "token",
        "private_key",
    ] {
        assert!(
            !script.to_lowercase().contains(forbidden),
            "generated PL/SQL leaked forbidden token `{forbidden}`"
        );
    }
}

#[test]
fn review_sql_artifact_is_read_only_outside_comments() {
    let review_sql = include_str!("../db/updates/07-02-taxonomy-backfill-review.sql");
    for line in review_sql.lines().filter(|line| {
        let trimmed = line.trim_start();
        !trimmed.starts_with("--") && !trimmed.is_empty()
    }) {
        let lowered = line.to_lowercase();
        for forbidden in ["update ", "insert ", "merge ", "delete "] {
            assert!(
                !lowered.contains(forbidden),
                "review SQL contains mutating statement `{forbidden}` in `{line}`"
            );
        }
    }
}
