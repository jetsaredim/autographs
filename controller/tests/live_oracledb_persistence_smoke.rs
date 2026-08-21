#[cfg(feature = "oracledb-live-smoke")]
mod live {
    use std::{env, time::Instant};

    use autographs_controller::{
        media::PrivateMediaStore, oci_media::OciInstancePrincipalMediaStore,
        storage_keys::build_original_object_key,
    };
    use oracledb::{Config, Connection};
    use uuid::Uuid;

    const RUN_GATE: &str = "AUTOGRAPHS_LIVE_ORACLEDB_PERSISTENCE_SMOKE";
    const CLEANUP_ITEM_IDS: &str = "AUTOGRAPHS_LIVE_ORACLEDB_PERSISTENCE_CLEANUP_ITEM_IDS";
    const CLEANUP_OBJECT_KEYS: &str = "AUTOGRAPHS_LIVE_ORACLEDB_PERSISTENCE_CLEANUP_OBJECT_KEYS";

    #[tokio::test]
    #[ignore = "requires live Oracle wallet and OCI instance-principal media access"]
    async fn live_oracledb_persistence_smoke_persists_oracle_item_and_oci_original() {
        let cleanup_item_ids = optional_list(CLEANUP_ITEM_IDS);
        let cleanup_object_keys = optional_list(CLEANUP_OBJECT_KEYS);
        if !cleanup_item_ids.is_empty() || !cleanup_object_keys.is_empty() {
            run_cleanup(cleanup_item_ids, cleanup_object_keys).await;
            return;
        }
        if env::var(RUN_GATE).as_deref() != Ok("true") {
            println!("skipping oracledb persistence smoke: {RUN_GATE} is not true");
            return;
        }

        let started_at = Instant::now();
        let connection = connect().expect("connect to Oracle Autonomous Database with oracledb");
        let connect_elapsed = started_at.elapsed();
        assert_static_runtime_schema(&connection);
        let storage_namespace = required("OCI_MEDIA_NAMESPACE");
        let bucket_name = required("OCI_MEDIA_BUCKET_NAME");
        let media =
            OciInstancePrincipalMediaStore::new(storage_namespace.clone(), bucket_name.clone())
                .expect("configure OCI instance-principal media store");

        let item_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();
        let item_id_text = item_id.to_string();
        let image_id_text = image_id.to_string();
        let object_key = build_original_object_key(item_id, image_id);
        println!("oracledb smoke item id: {item_id}");
        println!("oracledb smoke object key: {object_key}");

        let body = vec![
            0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0xff, 0xfe, 0xfd, 0x80, 0x81,
            0x82, 0x83,
        ];
        let mut item_written = false;
        let mut object_written = false;
        let result = async {
            let item_started_at = Instant::now();
            connection
                .execute(
                    "insert into autograph_items (id, title, signer, category, publication_status) values (:1, :2, :3, :4, :5)",
                    &[&item_id_text, &"Live Oracledb Smoke Signed Item", &"Live Oracledb Smoke Signer", &"Smoke", &"draft"],
                )
                .map_err(|error| format!("insert temporary autograph item: {error}"))?;
            connection
                .commit()
                .map_err(|error| format!("commit temporary autograph item: {error}"))?;
            item_written = true;
            let item_elapsed = item_started_at.elapsed();

            let upload_started_at = Instant::now();
            media
                .write(&object_key, &body)
                .await
                .map_err(|error| format!("upload private original to OCI Object Storage: {error}"))?;
            object_written = true;
            let upload_elapsed = upload_started_at.elapsed();

            let image_started_at = Instant::now();
            connection
                .execute(
                    "insert into autograph_images (id, item_id, storage_namespace, bucket_name, object_key, content_type, byte_size, original_filename, is_primary) values (:1, :2, :3, :4, :5, :6, :7, :8, 'Y')",
                    &[&image_id_text, &item_id_text, &storage_namespace, &bucket_name, &object_key, &"application/octet-stream", &(body.len() as i64), &"live secret source.jpg"],
                )
                .map_err(|error| format!("insert temporary autograph image metadata: {error}"))?;
            connection
                .commit()
                .map_err(|error| format!("commit temporary autograph image metadata: {error}"))?;
            let image_elapsed = image_started_at.elapsed();

            let verify_started_at = Instant::now();
            let downloaded = media
                .read(&object_key)
                .await
                .map_err(|error| format!("read private original from OCI Object Storage: {error}"))?;
            if downloaded != body {
                return Err("downloaded private original differs from uploaded bytes".to_owned());
            }
            let item = connection
                .query_row("select title from autograph_items where id = :1", &[&item_id_text])
                .map_err(|error| format!("read temporary autograph item: {error}"))?;
            let title: String = item
                .get(0)
                .map_err(|error| format!("decode temporary autograph item: {error}"))?;
            if title != "Live Oracledb Smoke Signed Item" {
                return Err("temporary autograph item did not round-trip".to_owned());
            }
            let image = connection
                .query_row(
                    "select object_key, original_filename from autograph_images where id = :1",
                    &[&image_id_text],
                )
                .map_err(|error| format!("read temporary autograph image metadata: {error}"))?;
            let persisted_key: String = image
                .get(0)
                .map_err(|error| format!("decode temporary image object key: {error}"))?;
            let source_filename: String = image
                .get(1)
                .map_err(|error| format!("decode temporary image filename: {error}"))?;
            if persisted_key != object_key || source_filename != "live secret source.jpg" {
                return Err("temporary autograph image metadata did not round-trip".to_owned());
            }
            let verify_elapsed = verify_started_at.elapsed();
            println!(
                "oracledb persistence timings: connect_ms={} item_ms={} upload_ms={} image_ms={} verify_ms={}",
                connect_elapsed.as_millis(),
                item_elapsed.as_millis(),
                upload_elapsed.as_millis(),
                image_elapsed.as_millis(),
                verify_elapsed.as_millis(),
            );
            Ok::<(), String>(())
        }
        .await;

        let cleanup_started_at = Instant::now();
        let cleanup_result = cleanup(
            &connection,
            &media,
            &item_id_text,
            &object_key,
            item_written,
            object_written,
        )
        .await;
        println!(
            "oracledb persistence cleanup_ms={}",
            cleanup_started_at.elapsed().as_millis()
        );
        result.expect("complete temporary Oracle and OCI smoke lifecycle");
        cleanup_result.expect("clean up temporary Oracle and OCI smoke lifecycle");
    }

    fn connect() -> Result<Connection, String> {
        let user = required("ORACLE_DB_USER");
        let password = required("ORACLE_DB_PASSWORD");
        let connect_string = required("ORACLE_DB_CONNECT_STRING");
        let wallet_dir = env::var("ORACLE_DB_WALLET_DIR")
            .or_else(|_| env::var("TNS_ADMIN"))
            .map_err(|_| "ORACLE_DB_WALLET_DIR or TNS_ADMIN is required".to_owned())?;
        let mut config = Config::default()
            .set_credentials(&user, &password)
            .set_config_dir(&wallet_dir)
            .set_wallet_location(&wallet_dir)
            .set_connect_string(&connect_string)
            .map_err(|error| format!("configure Oracle connect string: {error}"))?;
        if let Ok(wallet_password) = env::var("ORACLE_DB_WALLET_PASSWORD")
            && !wallet_password.trim().is_empty()
        {
            config = config.set_wallet_password(&wallet_password);
        }
        oracledb::connect(config).map_err(|error| format!("connect to Oracle catalog: {error}"))
    }

    fn assert_static_runtime_schema(connection: &Connection) {
        let row = connection
            .query_row(
                "select count(*) from user_tables where table_name in ('AUTOGRAPH_ITEMS', 'AUTOGRAPH_IMAGES')",
                &[],
            )
            .expect("read catalog schema state");
        let count: i64 = row.get(0).expect("decode catalog schema state");
        assert_eq!(count, 2, "required catalog tables are present");
    }

    async fn run_cleanup(item_ids: Vec<String>, object_keys: Vec<String>) {
        let connection = connect().expect("connect to Oracle for smoke cleanup");
        assert_static_runtime_schema(&connection);
        let media = OciInstancePrincipalMediaStore::new(
            required("OCI_MEDIA_NAMESPACE"),
            required("OCI_MEDIA_BUCKET_NAME"),
        )
        .expect("configure OCI media for smoke cleanup");
        for item_id in item_ids {
            cleanup_database(&connection, &item_id).expect("clean up temporary Oracle rows");
        }
        for object_key in object_keys {
            media
                .delete(&object_key)
                .await
                .expect("delete temporary OCI object");
            assert!(
                media.read(&object_key).await.is_err(),
                "temporary OCI object is absent"
            );
        }
    }

    async fn cleanup(
        connection: &Connection,
        media: &OciInstancePrincipalMediaStore,
        item_id: &str,
        object_key: &str,
        item_written: bool,
        object_written: bool,
    ) -> Result<(), String> {
        let database_result = if item_written {
            cleanup_database(connection, item_id)
        } else {
            Ok(())
        };
        let object_result = if object_written {
            media
                .delete(object_key)
                .await
                .map_err(|error| format!("delete temporary OCI object: {error}"))?;
            if media.read(object_key).await.is_ok() {
                Err("temporary OCI object remains after cleanup".to_owned())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
        database_result?;
        object_result
    }

    fn cleanup_database(connection: &Connection, item_id: &str) -> Result<(), String> {
        for sql in [
            "delete from autograph_cleanup_events where item_id = :1",
            "delete from autograph_images where item_id = :1",
            "delete from autograph_item_tags where item_id = :1",
            "delete from autograph_items where id = :1",
        ] {
            connection
                .execute(sql, &[&item_id])
                .map_err(|error| format!("delete temporary Oracle row: {error}"))?;
        }
        connection
            .commit()
            .map_err(|error| format!("commit temporary Oracle cleanup: {error}"))?;
        let row = connection
            .query_row(
                "select count(*) from autograph_items where id = :1",
                &[&item_id],
            )
            .map_err(|error| format!("verify temporary Oracle cleanup: {error}"))?;
        let remaining: i64 = row
            .get(0)
            .map_err(|error| format!("decode temporary Oracle cleanup: {error}"))?;
        if remaining != 0 {
            return Err("temporary Oracle item remains after cleanup".to_owned());
        }
        Ok(())
    }

    fn optional_list(name: &str) -> Vec<String> {
        env::var(name)
            .unwrap_or_default()
            .split([',', '\n'])
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    fn required(name: &str) -> String {
        env::var(name).unwrap_or_else(|_| panic!("{name} is required"))
    }
}

#[cfg(not(feature = "oracledb-live-smoke"))]
#[test]
fn live_oracledb_persistence_smoke_requires_explicit_feature() {
    println!("skipping oracledb persistence smoke: compile with --features oracledb-live-smoke");
}
