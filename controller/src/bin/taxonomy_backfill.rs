use std::{env, fs, process};

use autographs_controller::taxonomy_migration::{
    BackfillMapping, LegacyExportRow, generate_backfill_report, generate_plsql_script,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mode = args
        .next()
        .ok_or_else(|| usage("missing mode: expected report or plsql"))?;
    let mut mapping_path = None;
    let mut input_path = None;
    let mut out_path = None;

    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| usage(&format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--mapping" => mapping_path = Some(value),
            "--input" => input_path = Some(value),
            "--out" => out_path = Some(value),
            _ => return Err(usage(&format!("unsupported flag {flag}"))),
        }
    }

    let mapping_path = mapping_path.ok_or_else(|| usage("missing --mapping"))?;
    let input_path = input_path.ok_or_else(|| usage("missing --input"))?;
    let out_path = out_path.ok_or_else(|| usage("missing --out"))?;
    let mapping = read_json::<BackfillMapping>(&mapping_path)?;
    let rows = read_json::<Vec<LegacyExportRow>>(&input_path)?;

    let output = match mode.as_str() {
        "report" => generate_backfill_report(&mapping, &rows).to_markdown(),
        "plsql" => generate_plsql_script(&mapping, &rows, &mapping_path),
        _ => return Err(usage("unsupported mode: expected report or plsql")),
    };
    fs::write(&out_path, output).map_err(|error| format!("write {out_path}: {error}"))
}

fn read_json<T>(path: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let content = fs::read_to_string(path).map_err(|error| format!("read {path}: {error}"))?;
    serde_json::from_str(&content).map_err(|error| format!("parse {path}: {error}"))
}

fn usage(message: &str) -> String {
    format!(
        "{message}\nusage: taxonomy_backfill <report|plsql> --mapping PATH --input PATH --out PATH"
    )
}
