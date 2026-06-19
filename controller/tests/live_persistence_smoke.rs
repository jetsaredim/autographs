#[cfg(feature = "live-persistence")]
mod live {
    use std::env;

    use autographs_controller::{
        media::PrivateMediaStore, oci_media::OciInstancePrincipalMediaStore,
        storage_keys::build_original_object_key,
    };
    use oracle::Connection;
    use uuid::Uuid;

    #[tokio::test]
    #[ignore = "requires live Oracle wallet and OCI instance-principal media access"]
    async fn live_persistence_smoke_persists_oracle_item_and_oci_original() {
        let cleanup_item_ids = optional_list("AUTOGRAPHS_LIVE_PERSISTENCE_CLEANUP_ITEM_IDS")
            .or_else(|| optional_list("AUTOGRAPHS_LIVE_PERSISTENCE_CLEANUP_ITEM_ID"))
            .unwrap_or_default();
        let cleanup_object_keys = optional_list("AUTOGRAPHS_LIVE_PERSISTENCE_CLEANUP_OBJECT_KEYS")
            .or_else(|| optional_list("AUTOGRAPHS_LIVE_PERSISTENCE_CLEANUP_OBJECT_KEY"))
            .unwrap_or_default();
        if !cleanup_item_ids.is_empty() || !cleanup_object_keys.is_empty() {
            run_cleanup(cleanup_item_ids, cleanup_object_keys).await;
            return;
        }
        if env::var("AUTOGRAPHS_LIVE_PERSISTENCE_LIST_SMOKE_ROWS").as_deref() == Ok("true") {
            list_smoke_rows();
            return;
        }

        if env::var("AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE").as_deref() != Ok("true") {
            println!(
                "skipping live persistence smoke: AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE is not true"
            );
            return;
        }

        let oracle_user = required("ORACLE_DB_USER");
        let oracle_password = required("ORACLE_DB_PASSWORD");
        let oracle_connect_string = required("ORACLE_DB_CONNECT_STRING");
        let storage_namespace = required("OCI_MEDIA_NAMESPACE");
        let bucket_name = required("OCI_MEDIA_BUCKET_NAME");

        let connection =
            Connection::connect(&oracle_user, &oracle_password, &oracle_connect_string)
                .expect("connect to Oracle Autonomous Database");
        assert_static_runtime_schema(&connection);
        let media =
            OciInstancePrincipalMediaStore::new(storage_namespace.clone(), bucket_name.clone())
                .expect("configure OCI instance-principal media store");

        let item_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();
        let object_key = build_original_object_key(item_id, image_id);
        let source_filename = "live secret source.jpg";
        assert!(!object_key.contains(source_filename));
        assert!(!object_key.contains(".jpg"));
        println!("live smoke item id: {item_id}");
        println!("live smoke object key: {object_key}");

        let item_id = item_id.to_string();
        let image_id = image_id.to_string();
        let _cleanup = LivePersistenceSmokeCleanup {
            connection: &connection,
            media: media.clone(),
            item_id: item_id.clone(),
            object_key: object_key.clone(),
        };
        connection
            .execute(
                "insert into autograph_items (id, title, signer, category, publication_status) values (:1, :2, :3, :4, :5)",
                &[&item_id, &"Live Smoke Signed Item", &"Live Smoke Signer", &"Smoke", &"draft"],
            )
            .expect("insert live smoke item");
        connection.commit().expect("commit smoke item");

        let body = vec![
            0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0xff, 0xfe, 0xfd, 0x80, 0x81,
            0x82, 0x83,
        ];
        media
            .write(&object_key, &body)
            .await
            .expect("upload private original to OCI Object Storage");
        connection
            .execute(
                "insert into autograph_images (id, item_id, storage_namespace, bucket_name, object_key, content_type, byte_size, original_filename, is_primary) values (:1, :2, :3, :4, :5, :6, :7, :8, 'Y')",
                &[&image_id, &item_id, &storage_namespace, &bucket_name, &object_key, &"application/octet-stream", &(body.len() as i64), &source_filename],
            )
            .expect("insert live smoke image metadata");
        connection.commit().expect("commit smoke image metadata");
        let downloaded = media
            .read(&object_key)
            .await
            .expect("read private original from OCI Object Storage");
        assert_eq!(downloaded, body);

        let mut rows = connection
            .query(
                "select title from autograph_items where id = :1",
                &[&item_id],
            )
            .expect("read live smoke item");
        let title: String = rows
            .next()
            .expect("read live smoke item row")
            .expect("read live smoke item row values")
            .get(0)
            .expect("read live smoke item title");
        assert_eq!(title, "Live Smoke Signed Item");
        assert!(rows.next().is_none());

        let mut image_rows = connection
            .query(
                "select object_key, original_filename from autograph_images where id = :1",
                &[&image_id],
            )
            .expect("read live smoke image metadata");
        let image_row = image_rows
            .next()
            .expect("read live smoke image metadata row")
            .expect("read live smoke image metadata row values");
        let stored_object_key: String = image_row.get(0).expect("read stored object key");
        let stored_filename: String = image_row.get(1).expect("read stored original filename");
        assert_eq!(stored_object_key, object_key);
        assert_eq!(stored_filename, "live secret source.jpg");
        assert!(image_rows.next().is_none());
    }

    async fn run_cleanup(item_ids: Vec<String>, object_keys: Vec<String>) {
        println!(
            "running live persistence cleanup: {} item id(s), {} object key(s)",
            item_ids.len(),
            object_keys.len()
        );
        let oracle_user = required("ORACLE_DB_USER");
        let oracle_password = required("ORACLE_DB_PASSWORD");
        let oracle_connect_string = required("ORACLE_DB_CONNECT_STRING");
        let storage_namespace = required("OCI_MEDIA_NAMESPACE");
        let bucket_name = required("OCI_MEDIA_BUCKET_NAME");

        let connection =
            Connection::connect(&oracle_user, &oracle_password, &oracle_connect_string)
                .expect("connect to Oracle Autonomous Database for cleanup");
        assert_static_runtime_schema(&connection);
        let media =
            OciInstancePrincipalMediaStore::new(storage_namespace.clone(), bucket_name.clone())
                .expect("configure OCI instance-principal media store for cleanup");

        for item_id in item_ids {
            println!("cleanup deleting Oracle rows for item id: {item_id}");
            connection
                .execute(
                    "delete from autograph_images where item_id = :1",
                    &[&item_id],
                )
                .expect("delete cleanup image rows");
            connection
                .execute(
                    "delete from autograph_item_tags where item_id = :1",
                    &[&item_id],
                )
                .expect("delete cleanup tag rows");
            connection
                .execute("delete from autograph_items where id = :1", &[&item_id])
                .expect("delete cleanup item row");
            connection.commit().expect("commit cleanup item deletes");
            assert_count_zero(
                &connection,
                "select count(*) from autograph_items where id = :1",
                &item_id,
                "cleanup item rows",
            );
            assert_count_zero(
                &connection,
                "select count(*) from autograph_images where item_id = :1",
                &item_id,
                "cleanup image rows",
            );
            assert_count_zero(
                &connection,
                "select count(*) from autograph_item_tags where item_id = :1",
                &item_id,
                "cleanup tag rows",
            );
        }

        for object_key in object_keys {
            println!("cleanup deleting OCI object key: {object_key}");
            media
                .delete(&object_key)
                .await
                .expect("delete cleanup OCI Object Storage object");
            match media.read(&object_key).await {
                Ok(_) => panic!("cleanup object still exists after delete: {object_key}"),
                Err(error) => println!("cleanup confirmed object absent: {object_key} ({error})"),
            }
        }

        println!("live persistence cleanup complete");
    }

    fn list_smoke_rows() {
        println!("listing live smoke rows from Oracle");
        let oracle_user = required("ORACLE_DB_USER");
        let oracle_password = required("ORACLE_DB_PASSWORD");
        let oracle_connect_string = required("ORACLE_DB_CONNECT_STRING");
        let connection =
            Connection::connect(&oracle_user, &oracle_password, &oracle_connect_string)
                .expect("connect to Oracle Autonomous Database for listing smoke rows");
        assert_static_runtime_schema(&connection);

        let mut rows = connection
            .query(
                "select
                   i.id,
                   i.title,
                   i.publication_status,
                   img.id,
                   img.object_key
                 from autograph_items i
                 left join autograph_images img on img.item_id = i.id
                 where i.title like 'Live Smoke%'
                    or i.title like 'Live Static Smoke%'
                 order by i.created_at, i.id, img.created_at, img.id",
                &[],
            )
            .expect("query live smoke rows");

        let mut found = false;
        while let Some(row) = rows.next() {
            found = true;
            let row = row.expect("read live smoke row");
            let item_id: String = row.get(0).expect("read smoke item id");
            let title: String = row.get(1).expect("read smoke title");
            let status: String = row.get(2).expect("read smoke status");
            let image_id: Option<String> = row.get(3).expect("read smoke image id");
            let object_key: Option<String> = row.get(4).expect("read smoke object key");
            println!(
                "item_id={item_id} status={status} title={title:?} image_id={} object_key={}",
                image_id.as_deref().unwrap_or("<none>"),
                object_key.as_deref().unwrap_or("<none>")
            );
        }
        if !found {
            println!("no live smoke rows found in Oracle");
        }
    }

    struct LivePersistenceSmokeCleanup<'a> {
        connection: &'a Connection,
        media: OciInstancePrincipalMediaStore,
        item_id: String,
        object_key: String,
    }

    impl Drop for LivePersistenceSmokeCleanup<'_> {
        fn drop(&mut self) {
            std::thread::scope(|scope| {
                let media = self.media.clone();
                let object_key = self.object_key.clone();
                let _ = scope
                    .spawn(move || {
                        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        else {
                            return;
                        };
                        let _ = runtime.block_on(media.delete(&object_key));
                    })
                    .join();
            });
            let _ = self.connection.execute(
                "delete from autograph_images where item_id = :1",
                &[&self.item_id],
            );
            let _ = self.connection.execute(
                "delete from autograph_items where id = :1",
                &[&self.item_id],
            );
            let _ = self.connection.commit();
        }
    }

    fn required(name: &str) -> String {
        env::var(name)
            .unwrap_or_else(|_| panic!("{name} is required for the live persistence smoke"))
    }

    fn optional_list(name: &str) -> Option<Vec<String>> {
        let values = env::var(name).ok()?;
        let values = values
            .split([',', '\n'])
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if values.is_empty() {
            None
        } else {
            Some(values)
        }
    }

    fn assert_count_zero(connection: &Connection, sql: &str, value: &str, label: &str) {
        let count: i64 = connection
            .query_row_as(sql, &[&value])
            .unwrap_or_else(|error| panic!("verify {label}: {error}"));
        assert_eq!(count, 0, "{label} still present for {value}");
        println!("verified {label}: 0");
    }

    fn assert_static_runtime_schema(connection: &Connection) {
        let count: i64 = connection
            .query_row_as(
                "select count(*) from user_tab_columns where table_name = 'AUTOGRAPH_IMAGES' and column_name = 'ORIGINAL_FILENAME'",
                &[],
            )
            .expect("inspect static runtime schema");
        assert_eq!(
            count, 1,
            "static runtime schema is missing ORIGINAL_FILENAME; initialize the database from controller/db/schema.sql before the live persistence smoke"
        );
    }
}

#[cfg(not(feature = "live-persistence"))]
#[test]
#[ignore = "compile with --features live-persistence and supply live credentials"]
fn live_persistence_smoke_requires_explicit_feature() {
    println!("skipping live persistence smoke: compile with --features live-persistence");
}
