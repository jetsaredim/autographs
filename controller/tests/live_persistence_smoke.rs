#[cfg(feature = "live-persistence")]
mod live {
    use std::{env, sync::Arc, time::Duration};

    use autographs_controller::{
        catalog::{
            AutographImage, CatalogRepository, EditEventKind, ImageAdjustmentUpdateError,
            ImageReplacementInput,
        },
        config::ControllerConfig,
        image_adjustments::ImageAdjustment,
        media::PrivateMediaStore,
        oci_media::OciInstancePrincipalMediaStore,
        oracle_catalog::OracleCatalogRepository,
        oracle_connection,
        routes::router_with_stores,
        storage_keys::build_original_object_key,
    };
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use oracledb::Connection;
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use tower::ServiceExt;
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
        let audit_requested =
            env::var("AUTOGRAPHS_LIVE_PERSISTENCE_AUDIT_IMAGE_CHECKSUMS").as_deref() == Ok("true");
        let repair_image_checksums =
            env::var("AUTOGRAPHS_LIVE_PERSISTENCE_REPAIR_IMAGE_CHECKSUMS").as_deref() == Ok("true");
        if audit_requested || repair_image_checksums {
            audit_image_checksums(repair_image_checksums).await;
            return;
        }

        if env::var("AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE").as_deref() != Ok("true") {
            println!(
                "skipping live persistence smoke: AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE is not true"
            );
            return;
        }
        let media_revision_smoke =
            env::var("AUTOGRAPHS_LIVE_MEDIA_REVISION_SMOKE").as_deref() == Ok("true");

        let oracle_user = required("ORACLE_DB_USER");
        let oracle_password = required("ORACLE_DB_PASSWORD");
        let oracle_connect_string = required("ORACLE_DB_CONNECT_STRING");
        let storage_namespace = required("OCI_MEDIA_NAMESPACE");
        let bucket_name = required("OCI_MEDIA_BUCKET_NAME");

        let connection =
            oracle_connection::connect(&oracle_user, &oracle_password, &oracle_connect_string)
                .expect("connect to Oracle Autonomous Database");
        assert_static_runtime_schema(&connection);
        assert_image_adjustment_migration_states(&connection);
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
        if !media_revision_smoke {
            println!("live smoke object key: {object_key}");
        }

        let item_id = item_id.to_string();
        let image_id = image_id.to_string();
        let smoke_title = if media_revision_smoke {
            format!("Live Media Revision Smoke {item_id}")
        } else {
            "Live Smoke Signed Item".to_owned()
        };
        let mut cleanup = LivePersistenceSmokeCleanup {
            connection: &connection,
            media: media.clone(),
            item_id: item_id.clone(),
            object_keys: vec![object_key.clone()],
            cleaned: false,
        };
        connection
            .execute(
                "insert into autograph_items (id, title, signer, category, publication_status) values (:1, :2, :3, :4, :5)",
                &[&item_id, &smoke_title, &"Live Smoke Signer", &"Smoke", &"draft"],
            )
            .expect("insert live smoke item");
        connection.commit().expect("commit smoke item");

        let body = if media_revision_smoke {
            tiny_png()
        } else {
            vec![
                0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0xff, 0xfe, 0xfd, 0x80, 0x81,
                0x82, 0x83,
            ]
        };
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
        assert_eq!(title, smoke_title);
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

        let repository = OracleCatalogRepository::new(
            oracle_user,
            oracle_password,
            oracle_connect_string,
            storage_namespace,
            bucket_name,
        );
        if media_revision_smoke {
            assert_media_revision_contract(
                Arc::new(repository),
                Arc::new(media.clone()),
                &item_id,
                &image_id,
                &object_key,
                &mut cleanup,
            )
            .await;
        } else {
            assert_oracle_adjustment_contract(&connection, repository, &item_id, &image_id).await;
        }
        cleanup
            .cleanup_and_verify()
            .await
            .expect("clean and verify live persistence smoke fixtures");
    }

    async fn assert_media_revision_contract(
        repository: Arc<OracleCatalogRepository>,
        media: Arc<OciInstancePrincipalMediaStore>,
        item_id: &str,
        image_id: &str,
        original_key: &str,
        cleanup: &mut LivePersistenceSmokeCleanup<'_>,
    ) {
        let item_id = Uuid::parse_str(item_id).expect("parse media revision smoke item id");
        let image_id = Uuid::parse_str(image_id).expect("parse media revision smoke image id");
        let mut image_b = tiny_png();
        image_b.extend_from_slice(b"revision-b");
        let checksum_a = Sha256::digest(
            media
                .read(original_key)
                .await
                .expect("read media revision image A"),
        );
        let checksum_b = Sha256::digest(&image_b);
        assert_ne!(checksum_a.as_slice(), checksum_b.as_slice());
        let app = router_with_stores(
            ControllerConfig::for_test(true),
            repository.clone(),
            media.clone(),
        );
        let cookie = live_admin_cookie(&app).await;
        let review_a = live_json(
            app.clone()
                .oneshot(
                    Request::get(format!(
                        "/admin/api/items/{item_id}/images/{image_id}/review"
                    ))
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap(),
                )
                .await
                .expect("request revision A review"),
        )
        .await;
        let revision_a = review_a["mediaRevision"]
            .as_str()
            .expect("revision A token")
            .to_owned();
        assert!(!review_a.to_string().contains(original_key));
        let replaced_response = app
            .clone()
            .oneshot(live_replace_request(item_id, image_id, &cookie, &image_b))
            .await
            .expect("replace revision A with image B");
        assert_eq!(replaced_response.status(), StatusCode::OK);
        let replaced = repository.get(item_id).await.unwrap().unwrap();
        let image_b_key = replaced
            .images
            .iter()
            .find(|image| image.id == image_id)
            .unwrap()
            .object_key
            .clone();
        cleanup.object_keys.push(image_b_key.clone());

        for (method, suffix, body) in [
            (
                "POST",
                "preview/draft",
                json!({"mediaRevision": revision_a, "adjustment": ImageAdjustment::identity()}),
            ),
            (
                "POST",
                "adjustment/assist",
                json!({"mediaRevision": revision_a}),
            ),
            (
                "PATCH",
                "adjustment",
                json!({"mediaRevision": revision_a, "adjustment": ImageAdjustment::identity()}),
            ),
            ("DELETE", "adjustment", json!({"mediaRevision": revision_a})),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(format!(
                            "/admin/api/items/{item_id}/images/{image_id}/{suffix}"
                        ))
                        .header(header::COOKIE, &cookie)
                        .header(header::ORIGIN, "https://autographs.example.test")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_redacted_live_conflict(response, original_key, &format!("{checksum_a:x}")).await;
        }
        let source = app.clone().oneshot(
            Request::get(format!("/admin/api/items/{item_id}/images/{image_id}/preview/source?mediaRevision={revision_a}"))
                .header(header::COOKIE, &cookie).body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_redacted_live_conflict(source, original_key, &format!("{checksum_a:x}")).await;

        let review_b = live_json(
            app.clone()
                .oneshot(
                    Request::get(format!(
                        "/admin/api/items/{item_id}/images/{image_id}/review"
                    ))
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap(),
                )
                .await
                .unwrap(),
        )
        .await;
        assert_ne!(review_b["mediaRevision"], revision_a);
        assert!(!review_b.to_string().contains(&image_b_key));
        let revision_b = review_b["mediaRevision"].as_str().unwrap();
        let current_reset = app
            .clone()
            .oneshot(
                Request::delete(format!(
                    "/admin/api/items/{item_id}/images/{image_id}/adjustment"
                ))
                .header(header::COOKIE, &cookie)
                .header(header::ORIGIN, "https://autographs.example.test")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"mediaRevision": revision_b}).to_string()))
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(current_reset.status(), StatusCode::OK);
        assert_eq!(
            media
                .read(&image_b_key)
                .await
                .expect("read media revision image B"),
            image_b
        );
        println!(
            "media revision smoke complete: item_id={item_id} stale_save=409 current_reset=200 cleanup=pending"
        );
    }

    async fn assert_redacted_live_conflict(
        response: axum::response::Response,
        key: &str,
        checksum: &str,
    ) {
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let value = live_json(response).await;
        assert_eq!(value["code"], "mediaRevisionConflict");
        let encoded = value.to_string();
        assert!(!encoded.contains(key));
        assert!(!encoded.contains(checksum));
    }

    async fn live_json(response: axum::response::Response) -> Value {
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
    }

    async fn live_admin_cookie(app: &axum::Router) -> String {
        let response = app
            .clone()
            .oneshot(
                Request::post("/admin/api/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"password":"local-test-password"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        response
            .headers()
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned()
    }

    fn live_replace_request(
        item_id: Uuid,
        image_id: Uuid,
        cookie: &str,
        image: &[u8],
    ) -> Request<Body> {
        let boundary = "live-media-revision-boundary";
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"revision-b.png\"\r\nContent-Type: image/png\r\n\r\n").as_bytes());
        body.extend_from_slice(image);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        Request::put(format!("/admin/api/items/{item_id}/images/{image_id}"))
            .header(header::COOKIE, cookie)
            .header(header::ORIGIN, "https://autographs.example.test")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap()
    }

    fn tiny_png() -> Vec<u8> {
        vec![
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
            8, 2, 0, 0, 0, 144, 119, 83, 222, 0, 0, 0, 12, 73, 68, 65, 84, 8, 215, 99, 248, 207,
            192, 0, 0, 3, 1, 1, 0, 24, 221, 141, 176, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
        ]
    }

    #[derive(Clone, Copy)]
    enum MigrationConstraintState {
        OldOnly,
        Both,
        TemporaryOnly,
        CanonicalNew,
        WidenedCanonical,
        WidenedTemporary,
        WrongCaseCanonical,
        WrongCaseTemporary,
    }

    #[derive(Clone, Copy)]
    enum MigrationConstraintCondition {
        Legacy,
        Exact,
        Widened,
        WrongCase,
    }

    fn assert_image_adjustment_migration_states(connection: &Connection) {
        for state in [
            MigrationConstraintState::OldOnly,
            MigrationConstraintState::Both,
            MigrationConstraintState::TemporaryOnly,
            MigrationConstraintState::CanonicalNew,
            MigrationConstraintState::WidenedCanonical,
            MigrationConstraintState::WidenedTemporary,
            MigrationConstraintState::WrongCaseCanonical,
            MigrationConstraintState::WrongCaseTemporary,
        ] {
            let suffix = Uuid::new_v4().simple().to_string()[..10].to_ascii_uppercase();
            let table = format!("AE_MIG_{suffix}");
            let canonical_constraint = format!("AE_CK_{suffix}");
            let temporary_constraint = format!("AE_V8_{suffix}");
            let mut cleanup = ScratchTableCleanup::create(connection, table.clone());

            match state {
                MigrationConstraintState::OldOnly => add_migration_check_constraint(
                    connection,
                    &table,
                    &canonical_constraint,
                    MigrationConstraintCondition::Legacy,
                ),
                MigrationConstraintState::Both => {
                    add_migration_check_constraint(
                        connection,
                        &table,
                        &canonical_constraint,
                        MigrationConstraintCondition::Legacy,
                    );
                    add_migration_check_constraint(
                        connection,
                        &table,
                        &temporary_constraint,
                        MigrationConstraintCondition::Exact,
                    );
                }
                MigrationConstraintState::TemporaryOnly => add_migration_check_constraint(
                    connection,
                    &table,
                    &temporary_constraint,
                    MigrationConstraintCondition::Exact,
                ),
                MigrationConstraintState::CanonicalNew => add_migration_check_constraint(
                    connection,
                    &table,
                    &canonical_constraint,
                    MigrationConstraintCondition::Exact,
                ),
                MigrationConstraintState::WidenedCanonical => add_migration_check_constraint(
                    connection,
                    &table,
                    &canonical_constraint,
                    MigrationConstraintCondition::Widened,
                ),
                MigrationConstraintState::WidenedTemporary => add_migration_check_constraint(
                    connection,
                    &table,
                    &temporary_constraint,
                    MigrationConstraintCondition::Widened,
                ),
                MigrationConstraintState::WrongCaseCanonical => add_migration_check_constraint(
                    connection,
                    &table,
                    &canonical_constraint,
                    MigrationConstraintCondition::WrongCase,
                ),
                MigrationConstraintState::WrongCaseTemporary => add_migration_check_constraint(
                    connection,
                    &table,
                    &temporary_constraint,
                    MigrationConstraintCondition::WrongCase,
                ),
            }

            let migration = migration_constraint_block_for_scratch(
                &table,
                &canonical_constraint,
                &temporary_constraint,
            );
            connection
                .execute(&migration, &[])
                .expect("apply image adjustment migration state transition");
            if matches!(state, MigrationConstraintState::CanonicalNew) {
                connection
                    .execute(&migration, &[])
                    .expect("rerun image adjustment migration state transition");
            }

            let canonical_count: i64 = connection
                .query_row(
                    "select count(*) from user_constraints
                      where table_name = :1
                        and constraint_name = :2
                        and constraint_type = 'C'
                        and status = 'ENABLED'",
                    &[&table, &canonical_constraint],
                )
                .expect("inspect migrated canonical constraint")
                .get(0)
                .expect("decode migrated canonical constraint count");
            assert_eq!(canonical_count, 1);
            let temporary_count: i64 = connection
                .query_row(
                    "select count(*) from user_constraints
                      where table_name = :1 and constraint_name = :2",
                    &[&table, &temporary_constraint],
                )
                .expect("inspect migrated temporary constraint")
                .get(0)
                .expect("decode migrated temporary constraint count");
            assert_eq!(temporary_count, 0);

            for kind in EditEventKind::ALL {
                connection
                    .execute(
                        &format!("insert into {table} (event_type) values (:1)"),
                        &[&kind.as_str()],
                    )
                    .unwrap_or_else(|error| {
                        panic!(
                            "migrated constraint rejected supported event {}: {error}",
                            kind.as_str()
                        )
                    });
            }
            assert!(
                connection
                    .execute(
                        &format!("insert into {table} (event_type) values ('unexpectedEvent')"),
                        &[],
                    )
                    .is_err(),
                "migrated constraint admitted an unsupported event"
            );
            connection
                .rollback()
                .expect("rollback migration state fixture rows");
            cleanup.cleanup();
        }
    }

    fn add_migration_check_constraint(
        connection: &Connection,
        table: &str,
        constraint: &str,
        condition: MigrationConstraintCondition,
    ) {
        let mut values = EditEventKind::ALL
            .into_iter()
            .filter(|kind| {
                !matches!(condition, MigrationConstraintCondition::Legacy)
                    || *kind != EditEventKind::ImageAdjustmentChanged
            })
            .map(|kind| format!("'{}'", kind.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
        if matches!(condition, MigrationConstraintCondition::WrongCase) {
            values = values.replace("'metadataUpdated'", "'metadataupdated'");
        }
        let widening = if matches!(condition, MigrationConstraintCondition::Widened) {
            " or 1 = 1"
        } else {
            ""
        };
        connection
            .execute(
                &format!(
                    "alter table {table} add constraint {constraint} check (event_type in ({values}){widening})"
                ),
                &[],
            )
            .expect("seed image adjustment migration constraint state");
    }

    fn migration_constraint_block_for_scratch(
        table: &str,
        canonical_constraint: &str,
        temporary_constraint: &str,
    ) -> String {
        let script = include_str!("../db/updates/08-01-image-adjustments.sql");
        let start = script
            .find("declare\n  canonical_constraint_count number;")
            .expect("find migration constraint block");
        let remainder = &script[start..];
        let end = remainder
            .find("\n/\n\ncommit;")
            .expect("find migration constraint block terminator");
        remainder[..end]
            .replace("AUTOGRAPH_EDIT_EVENTS_TYPE_V08", temporary_constraint)
            .replace(
                "autograph_edit_events_type_v08",
                &temporary_constraint.to_ascii_lowercase(),
            )
            .replace("AUTOGRAPH_EDIT_EVENTS_TYPE_CK", canonical_constraint)
            .replace(
                "autograph_edit_events_type_ck",
                &canonical_constraint.to_ascii_lowercase(),
            )
            .replace("AUTOGRAPH_EDIT_EVENTS", table)
            .replace("autograph_edit_events", &table.to_ascii_lowercase())
    }

    struct ScratchTableCleanup<'a> {
        connection: &'a Connection,
        table: String,
        active: bool,
    }

    impl<'a> ScratchTableCleanup<'a> {
        fn create(connection: &'a Connection, table: String) -> Self {
            connection
                .execute(
                    &format!("create table {table} (event_type varchar2(48) not null)"),
                    &[],
                )
                .expect("create image adjustment migration scratch table");
            Self {
                connection,
                table,
                active: true,
            }
        }

        fn cleanup(&mut self) {
            self.connection
                .execute(&format!("drop table {} purge", self.table), &[])
                .expect("drop image adjustment migration scratch table");
            self.active = false;
        }
    }

    impl Drop for ScratchTableCleanup<'_> {
        fn drop(&mut self) {
            if self.active
                && let Err(error) = self
                    .connection
                    .execute(&format!("drop table {} purge", self.table), &[])
            {
                eprintln!(
                    "LIVE_PERSISTENCE_RECOVERY_REQUIRED scratch_table={} error={error}",
                    self.table
                );
            }
        }
    }

    async fn assert_oracle_adjustment_contract(
        connection: &Connection,
        repository: OracleCatalogRepository,
        item_id: &str,
        target_image_id: &str,
    ) {
        let item_uuid = Uuid::parse_str(item_id).expect("parse smoke item id");
        let target_image_uuid =
            Uuid::parse_str(target_image_id).expect("parse smoke target image id");
        connection
            .execute(
                "update autograph_items set updated_at = timestamp '2000-01-01 00:00:00' where id = :1",
                &[&item_id],
            )
            .expect("set stable item timestamp before adjustment contract checks");
        connection
            .commit()
            .expect("commit stable item timestamp before adjustment contract checks");
        let before_updated_at = item_updated_at(connection, item_id);
        let before_adjustment_events =
            edit_event_count(connection, item_id, "imageAdjustmentChanged");
        let expected_object_key = repository
            .get(item_uuid)
            .await
            .expect("load live Oracle item for adjustment authority")
            .expect("live Oracle item exists")
            .images
            .into_iter()
            .find(|image| image.id == target_image_uuid)
            .expect("live Oracle target image exists")
            .object_key;

        let adjustment = ImageAdjustment::identity();
        let saved = repository
            .update_image_adjustment(
                item_uuid,
                target_image_uuid,
                &expected_object_key,
                Some(adjustment.clone()),
            )
            .await
            .expect("save live Oracle image adjustment");
        assert_eq!(
            saved
                .images
                .iter()
                .find(|image| image.id == target_image_uuid)
                .and_then(|image| image.adjustment.as_ref()),
            Some(&adjustment)
        );
        let stored_json = image_adjustment_json(connection, item_id, target_image_id)
            .expect("saved adjustment JSON is present");
        assert_eq!(
            ImageAdjustment::from_json(&stored_json).expect("decode saved adjustment JSON"),
            adjustment
        );
        assert_ne!(item_updated_at(connection, item_id), before_updated_at);
        assert_eq!(
            edit_event_count(connection, item_id, "imageAdjustmentChanged"),
            before_adjustment_events + 1
        );

        let reset = repository
            .update_image_adjustment(item_uuid, target_image_uuid, &expected_object_key, None)
            .await
            .expect("reset live Oracle image adjustment");
        assert_eq!(
            reset
                .images
                .iter()
                .find(|image| image.id == target_image_uuid)
                .and_then(|image| image.adjustment.as_ref()),
            None
        );
        assert_eq!(
            image_adjustment_json(connection, item_id, target_image_id),
            None
        );
        assert_eq!(
            edit_event_count(connection, item_id, "imageAdjustmentChanged"),
            before_adjustment_events + 2
        );

        for (wrong_item_id, wrong_image_id) in [
            (Uuid::new_v4(), target_image_uuid),
            (item_uuid, Uuid::new_v4()),
        ] {
            assert_eq!(
                repository
                    .update_image_adjustment(
                        wrong_item_id,
                        wrong_image_id,
                        &expected_object_key,
                        Some(ImageAdjustment::identity()),
                    )
                    .await
                    .expect_err("wrong adjustment identifiers must fail"),
                ImageAdjustmentUpdateError::NotFound
            );
        }
        assert_eq!(
            edit_event_count(connection, item_id, "imageAdjustmentChanged"),
            before_adjustment_events + 2
        );

        repository
            .update_image_adjustment(
                item_uuid,
                target_image_uuid,
                &expected_object_key,
                Some(ImageAdjustment::identity()),
            )
            .await
            .expect("save adjustment before replacement");
        assert_eq!(
            edit_event_count(connection, item_id, "imageAdjustmentChanged"),
            before_adjustment_events + 3
        );
        let replacement_object_key = build_original_object_key(item_uuid, Uuid::new_v4());
        let before_replacement_events = edit_event_count(connection, item_id, "imageReplaced");
        let replaced = repository
            .replace_image_metadata(
                item_uuid,
                target_image_uuid,
                &expected_object_key,
                ImageReplacementInput {
                    image: AutographImage {
                        id: Uuid::new_v4(),
                        object_key: replacement_object_key.clone(),
                        original_filename: "replacement-smoke.png".to_owned(),
                        content_type: "image/png".to_owned(),
                        byte_size: 16,
                        checksum: Some("replacement-checksum".to_owned()),
                        etag: Some("replacement-etag".to_owned()),
                        is_primary: false,
                        sort_order: 99,
                        alt_text: Some("Replacement smoke image".to_owned()),
                        adjustment: Some(ImageAdjustment::identity()),
                    },
                },
            )
            .await
            .expect("replace live Oracle image metadata");
        let replaced_image = replaced
            .images
            .iter()
            .find(|image| image.id == target_image_uuid)
            .expect("replaced image remains attached");
        assert_eq!(replaced_image.object_key, replacement_object_key);
        assert_eq!(replaced_image.adjustment, None);
        assert_eq!(
            image_adjustment_json(connection, item_id, target_image_id),
            None
        );
        assert_eq!(
            edit_event_count(connection, item_id, "imageReplaced"),
            before_replacement_events + 1
        );

        assert_failed_adjustment_reload_rolls_back(
            connection,
            &repository,
            item_id,
            target_image_id,
        )
        .await;
    }

    async fn assert_failed_adjustment_reload_rolls_back(
        connection: &Connection,
        repository: &OracleCatalogRepository,
        item_id: &str,
        target_image_id: &str,
    ) {
        let malformed_image_id = Uuid::new_v4().to_string();
        let malformed_object_key = build_original_object_key(
            Uuid::parse_str(item_id).expect("parse smoke item id"),
            Uuid::parse_str(&malformed_image_id).expect("parse malformed smoke image id"),
        );
        connection
            .execute(
                "insert into autograph_images (
                    id, item_id, storage_namespace, bucket_name, object_key,
                    content_type, byte_size, is_primary, adjustment_json
                ) values (:1, :2, :3, :4, :5, :6, :7, 'N', :8)",
                &[
                    &malformed_image_id,
                    &item_id,
                    &"live-smoke-namespace",
                    &"live-smoke-bucket",
                    &malformed_object_key,
                    &"application/octet-stream",
                    &1_i64,
                    &"not-json",
                ],
            )
            .expect("insert malformed sibling adjustment metadata");
        connection
            .commit()
            .expect("commit malformed sibling adjustment metadata");

        let before_updated_at: String = connection
            .query_row(
                "select to_char(updated_at, 'YYYY-MM-DD HH24:MI:SS.FF9') from autograph_items where id = :1",
                &[&item_id],
            )
            .expect("read item timestamp before rollback check")
            .get(0)
            .expect("decode item timestamp before rollback check");
        let before_history_count: i64 = connection
            .query_row(
                "select count(*) from autograph_edit_events where item_id = :1",
                &[&item_id],
            )
            .expect("read history count before rollback check")
            .get(0)
            .expect("decode history count before rollback check");

        let error = repository
            .update_image_adjustment(
                Uuid::parse_str(item_id).expect("parse smoke item id"),
                Uuid::parse_str(target_image_id).expect("parse smoke target image id"),
                &image_object_key(connection, item_id, target_image_id),
                Some(ImageAdjustment::identity()),
            )
            .await
            .expect_err("malformed sibling adjustment must reject the mutation");
        assert_eq!(
            error,
            ImageAdjustmentUpdateError::Repository(
                "read Oracle catalog image adjustment metadata: invalid adjustment JSON".to_owned()
            )
        );

        let target_adjustment: Option<String> = connection
            .query_row(
                "select adjustment_json from autograph_images where id = :1 and item_id = :2",
                &[&target_image_id, &item_id],
            )
            .expect("read target adjustment after rollback check")
            .get(0)
            .expect("decode target adjustment after rollback check");
        assert_eq!(target_adjustment, None);
        let after_updated_at: String = connection
            .query_row(
                "select to_char(updated_at, 'YYYY-MM-DD HH24:MI:SS.FF9') from autograph_items where id = :1",
                &[&item_id],
            )
            .expect("read item timestamp after rollback check")
            .get(0)
            .expect("decode item timestamp after rollback check");
        assert_eq!(after_updated_at, before_updated_at);
        let after_history_count: i64 = connection
            .query_row(
                "select count(*) from autograph_edit_events where item_id = :1",
                &[&item_id],
            )
            .expect("read history count after rollback check")
            .get(0)
            .expect("decode history count after rollback check");
        assert_eq!(after_history_count, before_history_count);
    }

    fn image_adjustment_json(
        connection: &Connection,
        item_id: &str,
        image_id: &str,
    ) -> Option<String> {
        connection
            .query_row(
                "select adjustment_json from autograph_images where id = :1 and item_id = :2",
                &[&image_id, &item_id],
            )
            .expect("read live image adjustment JSON")
            .get(0)
            .expect("decode live image adjustment JSON")
    }

    fn image_object_key(connection: &Connection, item_id: &str, image_id: &str) -> String {
        connection
            .query_row(
                "select object_key from autograph_images where item_id = :1 and id = :2",
                &[&item_id, &image_id],
            )
            .expect("read image object key")
            .get(0)
            .expect("decode image object key")
    }

    fn item_updated_at(connection: &Connection, item_id: &str) -> String {
        connection
            .query_row(
                "select to_char(updated_at, 'YYYY-MM-DD HH24:MI:SS.FF9') from autograph_items where id = :1",
                &[&item_id],
            )
            .expect("read live item timestamp")
            .get(0)
            .expect("decode live item timestamp")
    }

    fn edit_event_count(connection: &Connection, item_id: &str, event_type: &str) -> i64 {
        connection
            .query_row(
                "select count(*) from autograph_edit_events where item_id = :1 and event_type = :2",
                &[&item_id, &event_type],
            )
            .expect("read live edit event count")
            .get(0)
            .expect("decode live edit event count")
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
            oracle_connection::connect(&oracle_user, &oracle_password, &oracle_connect_string)
                .expect("connect to Oracle Autonomous Database for cleanup");
        assert_static_runtime_schema(&connection);
        let media =
            OciInstancePrincipalMediaStore::new(storage_namespace.clone(), bucket_name.clone())
                .expect("configure OCI instance-principal media store for cleanup");

        for item_id in item_ids {
            println!("cleanup deleting Oracle rows for item id: {item_id}");
            connection
                .execute(
                    "delete from autograph_cleanup_events where item_id = :1",
                    &[&item_id],
                )
                .expect("delete cleanup event rows");
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
            assert_count_zero(
                &connection,
                "select count(*) from autograph_cleanup_events where item_id = :1",
                &item_id,
                "cleanup event rows",
            );
        }

        for object_key in object_keys {
            println!("cleanup deleting OCI object key: {object_key}");
            tokio::time::timeout(Duration::from_secs(75), media.delete(&object_key))
                .await
                .unwrap_or_else(|_| panic!("timed out deleting cleanup OCI object: {object_key}"))
                .expect("delete cleanup OCI Object Storage object");
            verify_oci_object_absent(&media, &object_key)
                .await
                .unwrap_or_else(|error| panic!("{error}"));
        }

        println!("live persistence cleanup complete");
    }

    async fn verify_oci_object_absent(
        media: &OciInstancePrincipalMediaStore,
        object_key: &str,
    ) -> Result<(), String> {
        match tokio::time::timeout(Duration::from_secs(75), media.read(object_key)).await {
            Err(_) => Err("timed out confirming cleanup OCI object absence".to_owned()),
            Ok(Err(error)) if error.contains("returned status 404") => {
                println!("cleanup confirmed private object absent");
                Ok(())
            }
            Ok(Err(error)) => Err(format!(
                "could not confirm cleanup OCI object absence: {error}"
            )),
            Ok(Ok(_)) => Err("cleanup OCI object still exists after delete".to_owned()),
        }
    }

    fn list_smoke_rows() {
        println!("listing live smoke rows from Oracle");
        let oracle_user = required("ORACLE_DB_USER");
        let oracle_password = required("ORACLE_DB_PASSWORD");
        let oracle_connect_string = required("ORACLE_DB_CONNECT_STRING");
        let connection =
            oracle_connection::connect(&oracle_user, &oracle_password, &oracle_connect_string)
                .expect("connect to Oracle Autonomous Database for listing smoke rows");
        assert_static_runtime_schema(&connection);

        let rows = connection
            .query(
                "select
                   i.id,
                   i.title,
                   i.publication_status,
                   img.id,
                   img.object_key,
                   (select count(*)
                      from autograph_cleanup_events ce
                     where ce.item_id = i.id
                       and ce.status = 'deleteFailed')
                 from autograph_items i
                 left join autograph_images img on img.item_id = i.id
                 where i.title like 'Live Smoke%'
                    or i.title like 'Live Static Smoke%'
                 order by i.created_at, i.id, img.created_at, img.id",
                &[],
            )
            .expect("query live smoke rows");

        let mut found = false;
        for row in rows {
            found = true;
            let row = row.expect("read live smoke row");
            let item_id: String = row.get(0).expect("read smoke item id");
            let title: String = row.get(1).expect("read smoke title");
            let status: String = row.get(2).expect("read smoke status");
            let image_id: Option<String> = row.get(3).expect("read smoke image id");
            let object_key: Option<String> = row.get(4).expect("read smoke object key");
            let cleanup_warnings: i64 = row.get(5).expect("read smoke cleanup warning count");
            println!(
                "item_id={item_id} status={status} title={title:?} image_id={} object_key={} cleanup_warnings={cleanup_warnings}",
                image_id.as_deref().unwrap_or("<none>"),
                object_key.as_deref().unwrap_or("<none>")
            );
        }
        if !found {
            println!("no live smoke rows found in Oracle");
        }
    }

    async fn audit_image_checksums(repair: bool) {
        if repair {
            println!("repairing live image checksums from OCI Object Storage");
        } else {
            println!("auditing live image checksums against OCI Object Storage");
        }
        let oracle_user = required("ORACLE_DB_USER");
        let oracle_password = required("ORACLE_DB_PASSWORD");
        let oracle_connect_string = required("ORACLE_DB_CONNECT_STRING");
        let storage_namespace = required("OCI_MEDIA_NAMESPACE");
        let bucket_name = required("OCI_MEDIA_BUCKET_NAME");

        let connection =
            oracle_connection::connect(&oracle_user, &oracle_password, &oracle_connect_string)
                .expect("connect to Oracle Autonomous Database for checksum audit");
        assert_static_runtime_schema(&connection);
        let media =
            OciInstancePrincipalMediaStore::new(storage_namespace.clone(), bucket_name.clone())
                .expect("configure OCI instance-principal media store for checksum audit");

        let include_all =
            env::var("AUTOGRAPHS_LIVE_PERSISTENCE_AUDIT_ALL_IMAGES").as_deref() == Ok("true");
        let status_filter = if include_all {
            ""
        } else {
            " where i.publication_status = 'published'"
        };
        let sql = format!(
            "select
                i.id,
                i.title,
                i.publication_status,
                img.id,
                img.object_key,
                img.checksum
             from autograph_items i
             join autograph_images img on img.item_id = i.id
             {status_filter}
             order by i.title, i.id, img.sort_order, img.id"
        );
        let rows = connection
            .query(&sql, &[])
            .expect("query live image checksum audit rows");

        let mut checked = 0usize;
        let mut mismatched = 0usize;
        let mut missing_checksum = 0usize;
        let mut repaired = 0usize;
        let mut unreadable = 0usize;
        for row in rows {
            let row = row.expect("read live checksum audit row");
            let item_id: String = row.get(0).expect("read audit item id");
            let title: String = row.get(1).expect("read audit title");
            let status: String = row.get(2).expect("read audit publication status");
            let image_id: String = row.get(3).expect("read audit image id");
            let object_key: String = row.get(4).expect("read audit object key");
            let checksum: Option<String> = row.get(5).expect("read audit checksum");
            checked += 1;

            let Some(expected) = checksum
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                missing_checksum += 1;
                println!(
                    "missing checksum: item_id={item_id} image_id={image_id} status={status} title={title:?}"
                );
                if repair {
                    let bytes = match tokio::time::timeout(
                        Duration::from_secs(75),
                        media.read(&object_key),
                    )
                    .await
                    {
                        Err(_) => {
                            unreadable += 1;
                            println!(
                                "unreadable object timeout: item_id={item_id} image_id={image_id} status={status} title={title:?}"
                            );
                            continue;
                        }
                        Ok(Err(error)) => {
                            unreadable += 1;
                            println!(
                                "unreadable object: item_id={item_id} image_id={image_id} status={status} title={title:?} error={error}"
                            );
                            continue;
                        }
                        Ok(Ok(bytes)) => bytes,
                    };
                    let actual = image_checksum(&bytes);
                    update_image_checksum(&connection, &image_id, &actual);
                    repaired += 1;
                    println!(
                        "repaired missing checksum: item_id={item_id} image_id={image_id} status={status} title={title:?} actual={actual}"
                    );
                }
                continue;
            };

            let bytes = match tokio::time::timeout(Duration::from_secs(75), media.read(&object_key))
                .await
            {
                Err(_) => {
                    unreadable += 1;
                    println!(
                        "unreadable object timeout: item_id={item_id} image_id={image_id} status={status} title={title:?}"
                    );
                    continue;
                }
                Ok(Err(error)) => {
                    unreadable += 1;
                    println!(
                        "unreadable object: item_id={item_id} image_id={image_id} status={status} title={title:?} error={error}"
                    );
                    continue;
                }
                Ok(Ok(bytes)) => bytes,
            };
            let actual = image_checksum(&bytes);
            if actual != expected {
                mismatched += 1;
                println!(
                    "checksum mismatch: item_id={item_id} image_id={image_id} status={status} title={title:?} expected={expected} actual={actual}"
                );
                if repair {
                    update_image_checksum(&connection, &image_id, &actual);
                    repaired += 1;
                    println!(
                        "repaired checksum mismatch: item_id={item_id} image_id={image_id} status={status} title={title:?} actual={actual}"
                    );
                }
            }
        }
        if repair {
            connection
                .commit()
                .expect("commit live image checksum repairs");
        }

        println!(
            "checksum audit complete: checked={checked} mismatched={mismatched} missing_checksum={missing_checksum} repaired={repaired} unreadable={unreadable}"
        );
        assert_eq!(unreadable, 0, "checksum audit found unreadable images");
        if !repair {
            assert_eq!(mismatched, 0, "checksum audit found mismatched images");
            assert_eq!(
                missing_checksum, 0,
                "checksum audit found images missing checksum metadata"
            );
        }
    }

    struct LivePersistenceSmokeCleanup<'a> {
        connection: &'a Connection,
        media: OciInstancePrincipalMediaStore,
        item_id: String,
        object_keys: Vec<String>,
        cleaned: bool,
    }

    impl LivePersistenceSmokeCleanup<'_> {
        async fn cleanup_and_verify(&mut self) -> Result<(), String> {
            for object_key in &self.object_keys {
                let delete_result =
                    tokio::time::timeout(Duration::from_secs(75), self.media.delete(object_key))
                        .await
                        .map_err(|_| "timed out deleting live smoke OCI object".to_owned())?;
                if let Err(error) = delete_result
                    && !is_oci_not_found(&error)
                {
                    return Err(format!("delete live smoke OCI object: {error}"));
                }
                verify_oci_object_absent(&self.media, object_key).await?;
            }

            let statement = self
                .connection
                .execute(
                    "delete from autograph_items where id = :1",
                    &[&self.item_id],
                )
                .map_err(|error| format!("delete live smoke Oracle item: {error}"))?;
            if statement.rows_affected() != 1 {
                return Err(format!(
                    "delete live smoke Oracle item affected {} rows instead of 1",
                    statement.rows_affected()
                ));
            }
            self.connection
                .commit()
                .map_err(|error| format!("commit live smoke Oracle cleanup: {error}"))?;

            for (sql, label) in [
                (
                    "select count(*) from autograph_items where id = :1",
                    "item rows",
                ),
                (
                    "select count(*) from autograph_images where item_id = :1",
                    "image rows",
                ),
                (
                    "select count(*) from autograph_edit_events where item_id = :1",
                    "edit-event rows",
                ),
            ] {
                let count: i64 = self
                    .connection
                    .query_row(sql, &[&self.item_id])
                    .map_err(|error| format!("verify live smoke cleanup {label}: {error}"))?
                    .get(0)
                    .map_err(|error| format!("decode live smoke cleanup {label}: {error}"))?;
                if count != 0 {
                    return Err(format!(
                        "live smoke cleanup left {count} {label} for item {}",
                        self.item_id
                    ));
                }
            }

            self.cleaned = true;
            Ok(())
        }

        fn report_recovery_required(&self, step: &str, error: impl std::fmt::Display) {
            eprintln!(
                "LIVE_PERSISTENCE_RECOVERY_REQUIRED item_id={} step={} error={error}",
                self.item_id, step
            );
        }
    }

    impl Drop for LivePersistenceSmokeCleanup<'_> {
        fn drop(&mut self) {
            if self.cleaned {
                return;
            }

            let media_result = std::thread::scope(|scope| {
                let media = self.media.clone();
                let object_keys = self.object_keys.clone();
                scope
                    .spawn(move || -> Result<(), String> {
                        let runtime = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .map_err(|error| format!("build fallback cleanup runtime: {error}"))?;
                        runtime.block_on(async {
                            for object_key in object_keys {
                                let delete_result = tokio::time::timeout(
                                    Duration::from_secs(75),
                                    media.delete(&object_key),
                                )
                                .await
                                .map_err(|_| "timed out deleting fallback OCI object".to_owned())?;
                                if let Err(error) = delete_result
                                    && !is_oci_not_found(&error)
                                {
                                    return Err(format!("delete fallback OCI object: {error}"));
                                }
                            }
                            Ok(())
                        })
                    })
                    .join()
            });
            match media_result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => self.report_recovery_required("oci-delete", error),
                Err(_) => self.report_recovery_required("oci-delete", "fallback thread panicked"),
            }

            if let Err(error) = self.connection.execute(
                "delete from autograph_items where id = :1",
                &[&self.item_id],
            ) {
                self.report_recovery_required("oracle-delete", error);
                return;
            }
            if let Err(error) = self.connection.commit() {
                self.report_recovery_required("oracle-commit", error);
            }
        }
    }

    fn is_oci_not_found(error: &str) -> bool {
        error.contains("returned status 404")
    }

    #[test]
    fn live_cleanup_accepts_only_provider_not_found_errors() {
        assert!(is_oci_not_found(
            "delete OCI private media object returned status 404 Not Found:"
        ));
        assert!(!is_oci_not_found(
            "delete OCI private media object returned status 403 Forbidden:"
        ));
        assert!(!is_oci_not_found(
            "delete OCI private media object: timeout"
        ));
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
        let row = connection
            .query_row(sql, &[&value])
            .unwrap_or_else(|error| panic!("verify {label}: {error}"));
        let count: i64 = row
            .get(0)
            .unwrap_or_else(|error| panic!("decode {label} count: {error}"));
        assert_eq!(count, 0, "{label} still present for {value}");
        println!("verified {label}: 0");
    }

    fn assert_static_runtime_schema(connection: &Connection) {
        let row = connection
            .query_row(
                "select count(*) from user_tab_columns where table_name = 'AUTOGRAPH_IMAGES' and column_name = 'ORIGINAL_FILENAME'",
                &[],
            )
            .expect("inspect static runtime schema");
        let count: i64 = row.get(0).expect("decode static runtime schema count");
        assert_eq!(
            count, 1,
            "static runtime schema is missing ORIGINAL_FILENAME; initialize the database from controller/db/schema.sql before the live persistence smoke"
        );
        let row = connection
            .query_row(
                "select count(*) from user_tab_columns where table_name = 'AUTOGRAPH_CLEANUP_EVENTS' and column_name in ('ADMIN_MESSAGE', 'TARGET_OBJECT_KEY')",
                &[],
            )
            .expect("inspect cleanup event schema");
        let cleanup_count: i64 = row.get(0).expect("decode cleanup event schema count");
        assert_eq!(
            cleanup_count, 2,
            "static runtime schema is missing AUTOGRAPH_CLEANUP_EVENTS cleanup columns; initialize or update the database from controller/db/schema.sql and controller/db/updates/06-03-media-cleanup.sql before the live persistence smoke"
        );
        let row = connection
            .query_row(
                "select count(*) from user_tab_columns where table_name = 'AUTOGRAPH_IMAGES' and column_name = 'ADJUSTMENT_JSON'",
                &[],
            )
            .expect("inspect adjustment metadata schema");
        let adjustment_column_count: i64 =
            row.get(0).expect("decode adjustment metadata schema count");
        assert_eq!(
            adjustment_column_count, 1,
            "static runtime schema is missing AUTOGRAPH_IMAGES.ADJUSTMENT_JSON; run controller/db/updates/08-01-image-adjustments.sql before the live persistence smoke"
        );
        let row = connection
            .query_row(
                "select search_condition_vc from user_constraints
                  where table_name = 'AUTOGRAPH_EDIT_EVENTS'
                    and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_CK'
                    and constraint_type = 'C'
                    and status = 'ENABLED'",
                &[],
            )
            .expect("inspect adjustment event constraint");
        let edit_event_condition: String = row
            .get(0)
            .expect("decode adjustment event constraint condition");
        for kind in EditEventKind::ALL {
            assert!(
                edit_event_condition.contains(&format!("'{}'", kind.as_str())),
                "static runtime schema does not admit {}; run controller/db/updates/08-01-image-adjustments.sql before the live persistence smoke",
                kind.as_str()
            );
        }
    }

    fn update_image_checksum(connection: &Connection, image_id: &str, checksum: &str) {
        let statement = connection
            .execute(
                "update autograph_images set checksum = :1, updated_at = current_timestamp where id = :2",
                &[&checksum, &image_id],
            )
            .expect("update live image checksum");
        let rows_updated = statement.rows_affected();
        assert_eq!(rows_updated, 1, "expected one image checksum row update");
    }

    fn image_checksum(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

#[cfg(not(feature = "live-persistence"))]
#[test]
#[ignore = "compile with --features live-persistence and supply live credentials"]
fn live_persistence_smoke_requires_explicit_feature() {
    println!("skipping live persistence smoke: compile with --features live-persistence");
}
