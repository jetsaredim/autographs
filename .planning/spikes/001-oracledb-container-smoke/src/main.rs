use std::{env, process};

use oracledb::{Config, Connection};
use uuid::Uuid;

const READ_GATE: &str = "AUTOGRAPHS_ORACLEDB_SPIKE_READ_ONLY";
const WRITE_GATE: &str = "AUTOGRAPHS_ORACLEDB_SPIKE_WRITE_SMOKE";
const CLEANUP_ITEM_ID: &str = "AUTOGRAPHS_ORACLEDB_SPIKE_CLEANUP_ITEM_ID";
const DRIVER_VERSION: &str = "26.0.0-beta.2";

fn main() {
    if let Err(error) = run() {
        eprintln!("oracledb spike failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    if let Some(item_id) = optional(CLEANUP_ITEM_ID) {
        let connection = connect()?;
        cleanup(&connection, &item_id)?;
        println!("{{\"mode\":\"cleanup\",\"item_id\":\"{item_id}\",\"status\":\"clean\"}}");
        return Ok(());
    }

    if !enabled(READ_GATE) {
        println!("{{\"mode\":\"skipped\",\"reason\":\"{READ_GATE} is not true\"}}");
        return Ok(());
    }

    let connection = connect()?;
    read_only_preflight(&connection)?;

    if enabled(WRITE_GATE) {
        write_smoke(&connection)?;
    }

    Ok(())
}

fn connect() -> Result<Connection, String> {
    let user = required("ORACLE_DB_USER")?;
    let password = required("ORACLE_DB_PASSWORD")?;
    let connect_string = required("ORACLE_DB_CONNECT_STRING")?;
    let wallet_dir = env::var("ORACLE_DB_WALLET_DIR")
        .or_else(|_| env::var("TNS_ADMIN"))
        .map_err(|_| "ORACLE_DB_WALLET_DIR or TNS_ADMIN is required".to_owned())?;

    let mut config = Config::default()
        .set_credentials(&user, &password)
        .set_config_dir(&wallet_dir)
        .set_wallet_location(&wallet_dir)
        .set_connect_string(&connect_string)
        .map_err(|error| format!("configure Oracle connect string: {error}"))?;

    if let Some(wallet_password) = optional("ORACLE_DB_WALLET_PASSWORD") {
        config = config.set_wallet_password(&wallet_password);
    }

    oracledb::connect(config)
        .map_err(|error| format!("connect to Oracle Autonomous Database: {error}"))
}

fn read_only_preflight(connection: &Connection) -> Result<(), String> {
    let row = connection
        .query_row(
            "select count(*) from user_tables where table_name in ('AUTOGRAPH_ITEMS', 'AUTOGRAPH_IMAGES')",
            &[],
        )
        .map_err(|error| format!("read catalog schema state: {error}"))?;
    let table_count: i64 = row
        .get(0)
        .map_err(|error| format!("decode catalog schema state: {error}"))?;
    if table_count != 2 {
        return Err(format!(
            "catalog schema preflight expected 2 required tables, found {table_count}"
        ));
    }

    let row = connection
        .query_row(
            "select count(*) from autograph_items where rownum <= 1",
            &[],
        )
        .map_err(|error| format!("run representative read-only catalog query: {error}"))?;
    let sample_count: i64 = row
        .get(0)
        .map_err(|error| format!("decode representative read-only catalog query: {error}"))?;

    println!(
        "{{\"mode\":\"read-only\",\"driver\":\"oracledb\",\"driver_version\":\"{}\",\"schema\":\"ready\",\"representative_read\":{sample_count}}}",
        DRIVER_VERSION
    );
    Ok(())
}

fn write_smoke(connection: &Connection) -> Result<(), String> {
    let item_id = Uuid::new_v4().to_string();
    let image_id = Uuid::new_v4().to_string();
    let object_key = format!("originals/{item_id}/{image_id}");
    let title = format!("oracledb spike {item_id}");
    let signer = "oracledb spike";

    connection
        .execute(
            "insert into autograph_items (id, title, signer, category, publication_status) values (:1, :2, :3, :4, :5)",
            &[&item_id, &title, &signer, &"Smoke", &"draft"],
        )
        .map_err(|error| format!("insert temporary autograph item: {error}"))?;
    connection
        .execute(
            "insert into autograph_images (id, item_id, storage_namespace, bucket_name, object_key, content_type, byte_size, original_filename, is_primary) values (:1, :2, :3, :4, :5, :6, :7, :8, 'Y')",
            &[&image_id, &item_id, &"oracledb-spike", &"not-uploaded", &object_key, &"application/octet-stream", &16_i64, &"fake-image.bin"],
        )
        .map_err(|error| format!("insert temporary autograph image metadata: {error}"))?;
    connection
        .commit()
        .map_err(|error| format!("commit temporary smoke rows: {error}"))?;

    let result = (|| -> Result<(), String> {
        let item = connection
            .query_row(
                "select title from autograph_items where id = :1",
                &[&item_id],
            )
            .map_err(|error| format!("read temporary autograph item: {error}"))?;
        let persisted_title: String = item
            .get(0)
            .map_err(|error| format!("decode temporary autograph item: {error}"))?;
        if persisted_title != title {
            return Err("temporary autograph item title did not round-trip".to_owned());
        }
        let image = connection
            .query_row(
                "select object_key, original_filename from autograph_images where id = :1",
                &[&image_id],
            )
            .map_err(|error| format!("read temporary autograph image metadata: {error}"))?;
        let persisted_key: String = image
            .get(0)
            .map_err(|error| format!("decode temporary image object key: {error}"))?;
        let persisted_filename: String = image
            .get(1)
            .map_err(|error| format!("decode temporary image filename: {error}"))?;
        if persisted_key != object_key || persisted_filename != "fake-image.bin" {
            return Err("temporary autograph image metadata did not round-trip".to_owned());
        }
        Ok(())
    })();

    let cleanup_result = cleanup(connection, &item_id);
    result?;
    cleanup_result?;
    println!("{{\"mode\":\"write-smoke\",\"status\":\"created-verified-cleaned\"}}");
    Ok(())
}

fn cleanup(connection: &Connection, item_id: &str) -> Result<(), String> {
    for sql in [
        "delete from autograph_cleanup_events where item_id = :1",
        "delete from autograph_images where item_id = :1",
        "delete from autograph_item_tags where item_id = :1",
        "delete from autograph_items where id = :1",
    ] {
        connection
            .execute(sql, &[&item_id])
            .map_err(|error| format!("remove temporary smoke row: {error}"))?;
    }
    connection
        .commit()
        .map_err(|error| format!("commit temporary smoke cleanup: {error}"))?;
    let row = connection
        .query_row(
            "select count(*) from autograph_items where id = :1",
            &[&item_id],
        )
        .map_err(|error| format!("verify temporary smoke cleanup: {error}"))?;
    let remaining: i64 = row
        .get(0)
        .map_err(|error| format!("decode temporary smoke cleanup: {error}"))?;
    if remaining != 0 {
        return Err("temporary smoke item remains after cleanup".to_owned());
    }
    Ok(())
}

fn enabled(name: &str) -> bool {
    env::var(name).as_deref() == Ok("true")
}

fn required(name: &str) -> Result<String, String> {
    optional(name).ok_or_else(|| format!("{name} is required"))
}

fn optional(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}
