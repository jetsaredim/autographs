#[cfg(feature = "live-persistence")]
mod live {
    use std::env;

    use autographs_controller::storage_keys::build_original_object_key;
    use oracle::Connection;
    use s3::{Bucket, creds::Credentials, region::Region};
    use uuid::Uuid;

    #[tokio::test]
    #[ignore = "requires live Oracle wallet and OCI S3-compatible credentials"]
    async fn live_persistence_smoke_persists_oracle_item_and_oci_original() {
        if env::var("AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE").as_deref() != Ok("true") {
            println!(
                "skipping live persistence smoke: AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE is not true"
            );
            return;
        }

        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .expect("install rustls aws-lc-rs crypto provider");

        let oracle_user = required("ORACLE_DB_USER");
        let oracle_password = required("ORACLE_DB_PASSWORD");
        let oracle_connect_string = required("ORACLE_DB_CONNECT_STRING");
        let s3_endpoint = required("OCI_S3_ENDPOINT");
        let s3_region = env::var("OCI_REGION").unwrap_or_else(|_| "us-ashburn-1".to_owned());
        let s3_access_key = required("OCI_S3_ACCESS_KEY");
        let s3_secret_key = required("OCI_S3_SECRET_KEY");
        let storage_namespace = required("OCI_MEDIA_NAMESPACE");
        let bucket_name = required("OCI_MEDIA_BUCKET_NAME");

        let connection =
            Connection::connect(&oracle_user, &oracle_password, &oracle_connect_string)
                .expect("connect to Oracle Autonomous Database");
        assert_static_runtime_schema(&connection);
        let credentials =
            Credentials::new(Some(&s3_access_key), Some(&s3_secret_key), None, None, None)
                .expect("configure OCI Customer Secret credentials");
        let bucket = Bucket::new(
            &bucket_name,
            Region::Custom {
                region: s3_region,
                endpoint: s3_endpoint,
            },
            credentials,
        )
        .expect("configure OCI S3-compatible bucket")
        .with_path_style();

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
            bucket: &bucket,
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

        bucket
            .put_object_with_content_type(&object_key, b"live-smoke-private-original", "image/jpeg")
            .await
            .expect("upload private original to OCI Object Storage");
        connection
            .execute(
                "insert into autograph_images (id, item_id, storage_namespace, bucket_name, object_key, content_type, byte_size, original_filename, is_primary) values (:1, :2, :3, :4, :5, :6, :7, :8, 'Y')",
                &[&image_id, &item_id, &storage_namespace, &bucket_name, &object_key, &"image/jpeg", &27_i64, &source_filename],
            )
            .expect("insert live smoke image metadata");
        connection.commit().expect("commit smoke image metadata");
        let downloaded = bucket
            .get_object(&object_key)
            .await
            .expect("read private original from OCI Object Storage");
        assert_eq!(downloaded.bytes().as_ref(), b"live-smoke-private-original");

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

    struct LivePersistenceSmokeCleanup<'a> {
        connection: &'a Connection,
        bucket: &'a Bucket,
        item_id: String,
        object_key: String,
    }

    impl Drop for LivePersistenceSmokeCleanup<'_> {
        fn drop(&mut self) {
            std::thread::scope(|scope| {
                let bucket = self.bucket;
                let object_key = self.object_key.clone();
                let _ = scope
                    .spawn(move || {
                        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        else {
                            return;
                        };
                        let _ = runtime.block_on(bucket.delete_object(&object_key));
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

    fn assert_static_runtime_schema(connection: &Connection) {
        let count: i64 = connection
            .query_row_as(
                "select count(*) from user_tab_columns where table_name = 'AUTOGRAPH_IMAGES' and column_name = 'ORIGINAL_FILENAME'",
                &[],
            )
            .expect("inspect static runtime schema");
        assert_eq!(
            count, 1,
            "static runtime schema is missing ORIGINAL_FILENAME; run app db:migrate before the live persistence smoke"
        );
    }
}

#[cfg(not(feature = "live-persistence"))]
#[test]
#[ignore = "compile with --features live-persistence and supply live credentials"]
fn live_persistence_smoke_requires_explicit_feature() {
    println!("skipping live persistence smoke: compile with --features live-persistence");
}
