#[cfg(feature = "live-persistence")]
mod live {
    use std::env;

    use autographs_controller::storage_keys::build_original_object_key;
    use oracle_rs::{Config, Connection, Value};
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

        let oracle_host = required("AUTOGRAPHS_ORACLE_HOST");
        let oracle_port = required("AUTOGRAPHS_ORACLE_PORT").parse().unwrap();
        let oracle_service = required("AUTOGRAPHS_ORACLE_SERVICE_NAME");
        let oracle_user = required("ORACLE_DB_USER");
        let oracle_password = required("ORACLE_DB_PASSWORD");
        let wallet_dir = required("ORACLE_DB_WALLET_DIR");
        let wallet_password = env::var("ORACLE_DB_WALLET_PASSWORD").ok();
        let s3_endpoint = required("OCI_S3_ENDPOINT");
        let s3_region = env::var("OCI_REGION").unwrap_or_else(|_| "us-ashburn-1".to_owned());
        let s3_access_key = required("OCI_S3_ACCESS_KEY");
        let s3_secret_key = required("OCI_S3_SECRET_KEY");
        let storage_namespace = required("OCI_MEDIA_NAMESPACE");
        let bucket_name = required("OCI_MEDIA_BUCKET_NAME");

        let config = Config::new(
            oracle_host,
            oracle_port,
            oracle_service,
            oracle_user,
            oracle_password,
        )
        .with_wallet(wallet_dir, wallet_password.as_deref())
        .expect("configure Oracle wallet");
        let connection = Connection::connect_with_config(config)
            .await
            .expect("connect to Oracle Autonomous Database");
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

        let insert_params: Vec<Value> = vec![
            item_id.to_string().into(),
            "Live Smoke Signed Item".into(),
            "Live Smoke Signer".into(),
            "Smoke".into(),
            "draft".into(),
        ];
        connection
            .execute(
                "insert into autograph_items (id, title, signer, category, publication_status) values (:1, :2, :3, :4, :5)",
                &insert_params,
            )
            .await
            .expect("insert live smoke item");
        connection.commit().await.expect("commit smoke item");

        bucket
            .put_object_with_content_type(&object_key, b"live-smoke-private-original", "image/jpeg")
            .await
            .expect("upload private original to OCI Object Storage");
        let image_params: Vec<Value> = vec![
            image_id.to_string().into(),
            item_id.to_string().into(),
            storage_namespace.into(),
            bucket_name.clone().into(),
            object_key.clone().into(),
            "image/jpeg".into(),
            27_i64.into(),
            source_filename.into(),
        ];
        connection
            .execute(
                "insert into autograph_images (id, item_id, storage_namespace, bucket_name, object_key, content_type, byte_size, original_filename, is_primary) values (:1, :2, :3, :4, :5, :6, :7, :8, 'Y')",
                &image_params,
            )
            .await
            .expect("insert live smoke image metadata");
        connection
            .commit()
            .await
            .expect("commit smoke image metadata");
        let downloaded = bucket
            .get_object(&object_key)
            .await
            .expect("read private original from OCI Object Storage");
        assert_eq!(downloaded.bytes().as_ref(), b"live-smoke-private-original");

        let query_params: Vec<Value> = vec![item_id.to_string().into()];
        let result = connection
            .query(
                "select title from autograph_items where id = :1",
                &query_params,
            )
            .await
            .expect("read live smoke item");
        assert_eq!(result.rows.len(), 1);

        let image_query_params: Vec<Value> = vec![image_id.to_string().into()];
        let image_result = connection
            .query(
                "select object_key, original_filename from autograph_images where id = :1",
                &image_query_params,
            )
            .await
            .expect("read live smoke image metadata");
        assert_eq!(image_result.rows.len(), 1);
        assert_eq!(
            image_result.rows[0].get_string(0),
            Some(object_key.as_str())
        );
        assert_eq!(
            image_result.rows[0].get_string(1),
            Some("live secret source.jpg")
        );

        bucket
            .delete_object(&object_key)
            .await
            .expect("delete live smoke original");
        connection
            .execute("delete from autograph_items where id = :1", &query_params)
            .await
            .expect("delete live smoke item");
        connection.commit().await.expect("commit smoke cleanup");
    }

    fn required(name: &str) -> String {
        env::var(name)
            .unwrap_or_else(|_| panic!("{name} is required for the live persistence smoke"))
    }
}

#[cfg(not(feature = "live-persistence"))]
#[test]
#[ignore = "compile with --features live-persistence and supply live credentials"]
fn live_persistence_smoke_requires_explicit_feature() {
    println!("skipping live persistence smoke: compile with --features live-persistence");
}
