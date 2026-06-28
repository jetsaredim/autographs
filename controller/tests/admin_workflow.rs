use async_trait::async_trait;
use autographs_controller::{
    catalog::{
        AutographImage, AutographItemInput, AutographItemUpdate, CatalogRepository, CleanupStatus,
        EditEventKind, ImageCleanupEvent, MemoryCatalogRepository, PublicationStatus,
    },
    config::ControllerConfig,
    media::{LocalMediaStore, PrivateMediaStore},
    routes::router_with_stores,
    storage_keys::build_original_object_key,
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
use serde_json::{Value, json};
use std::{
    io::Cursor,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::Notify;
use tower::ServiceExt;

#[tokio::test]
async fn history_nullable_field_clear_records_before_and_after_values() {
    let repository = MemoryCatalogRepository::default();
    let item = repository
        .create(AutographItemInput {
            title: "Signed Jedi Card".to_owned(),
            signer: "Mark Hamill".to_owned(),
            description: Some("signed at event".to_owned()),
            category: "Cards".to_owned(),
            tags: vec!["jedi".to_owned()],
            object_reference: None,
            event_name: Some("Example Convention".to_owned()),
            event_location: None,
            source: None,
            inscription: None,
            certification_company: None,
            certification_id: None,
            estimated_year: None,
            publication_status: PublicationStatus::Draft,
        })
        .await
        .unwrap();

    let update: AutographItemUpdate = serde_json::from_value(json!({
        "description": null,
        "source": "Private collection"
    }))
    .unwrap();
    let updated = repository.update(item.id, update).await.unwrap();

    assert_eq!(updated.description, None);
    assert_eq!(updated.event_name.as_deref(), Some("Example Convention"));
    assert_eq!(updated.source.as_deref(), Some("Private collection"));

    let history = repository.history(item.id).await.unwrap();
    let metadata_event = history
        .iter()
        .find(|event| event.kind == EditEventKind::MetadataUpdated)
        .expect("metadata history event");
    assert_field_diff(
        metadata_event,
        "description",
        json!("signed at event"),
        Value::Null,
    );
    assert_field_diff(
        metadata_event,
        "source",
        Value::Null,
        json!("Private collection"),
    );
}

#[tokio::test]
async fn update_blank_required_field_returns_bad_request() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let item = repository
        .create(AutographItemInput {
            title: "Signed Jedi Card".to_owned(),
            signer: "Mark Hamill".to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: vec!["jedi".to_owned()],
            object_reference: None,
            event_name: None,
            event_location: None,
            source: None,
            inscription: None,
            certification_company: None,
            certification_id: None,
            estimated_year: None,
            publication_status: PublicationStatus::Draft,
        })
        .await
        .unwrap();
    let media_root = tempfile::tempdir().unwrap();
    let app = router_with_stores(
        ControllerConfig::for_test(false),
        repository,
        Arc::new(LocalMediaStore::new(media_root.path().to_path_buf())),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/api/items/{}", item.id))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_can_list_get_update_and_read_history() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let hamill = repository
        .create(test_item_input(
            "Signed Jedi Card",
            "Mark Hamill",
            "Cards",
            vec!["jedi", "skywalker"],
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();
    let fisher = repository
        .create(test_item_input(
            "Signed Princess Photo",
            "Carrie Fisher",
            "Photos",
            vec!["rebellion", "princess"],
            PublicationStatus::Published,
        ))
        .await
        .unwrap();
    repository
        .attach_image(
            hamill.id,
            AutographImage {
                id: uuid::Uuid::new_v4(),
                object_key: "OCI_objectstorage/private/leak-check.jpg".to_owned(),
                original_filename: "private-original.jpg".to_owned(),
                content_type: "image/jpeg".to_owned(),
                byte_size: 1234,
                is_primary: true,
                sort_order: 0,
                alt_text: Some("Signed Jedi Card by Mark Hamill".to_owned()),
            },
        )
        .await
        .unwrap();
    let media_root = tempfile::tempdir().unwrap();
    let app = router_with_stores(
        ControllerConfig::for_test(false),
        repository,
        Arc::new(LocalMediaStore::new(media_root.path().to_path_buf())),
    );

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api/items?query=mark&tag=jedi&publicationStatus=draft")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = response_string(list).await;
    assert_redacted(&list_body);
    let list_json: Value = serde_json::from_str(&list_body).unwrap();
    assert_eq!(list_json.as_array().unwrap().len(), 1);
    assert_eq!(list_json[0]["id"], hamill.id.to_string());
    assert_eq!(list_json[0]["title"], "Signed Jedi Card");
    assert_eq!(list_json[0]["imageCount"], 1);
    assert_json_true(&list_json[0]["hasPendingChanges"]);

    let detail = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/admin/api/items/{}", hamill.id))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), StatusCode::OK);
    let detail_body = response_string(detail).await;
    assert_redacted(&detail_body);
    let detail_json: Value = serde_json::from_str(&detail_body).unwrap();
    assert_eq!(detail_json["id"], hamill.id.to_string());
    assert_eq!(
        detail_json["images"][0]["altText"],
        "Signed Jedi Card by Mark Hamill"
    );
    assert_json_true(&detail_json["pendingChanges"]["hasPendingChanges"]);

    let patch = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/api/items/{}", hamill.id))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "title": "Signed Jedi Card - updated",
                        "signer": "Mark Hamill"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch.status(), StatusCode::OK);
    let patch_json = response_json(patch).await;
    assert_eq!(patch_json["title"], "Signed Jedi Card - updated");
    assert_json_true(&patch_json["pendingChanges"]["hasPendingChanges"]);
    assert!(patch_json["pendingChanges"]["count"].as_u64().unwrap() >= 2);

    let history = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/admin/api/items/{}/history", hamill.id))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(history.status(), StatusCode::OK);
    let history_body = response_string(history).await;
    assert_redacted(&history_body);
    let history_json: Value = serde_json::from_str(&history_body).unwrap();
    assert_eq!(history_json["itemId"], hamill.id.to_string());
    assert!(
        history_json["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| {
                event["eventType"] == "metadataUpdated"
                    && event["fieldDiffs"].as_array().unwrap().iter().any(|diff| {
                        diff["field"] == "title" && diff["after"] == "Signed Jedi Card - updated"
                    })
            })
    );

    let fisher_list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api/items?signer=fisher&category=photos&publicationStatus=published")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let fisher_json = response_json(fisher_list).await;
    assert_eq!(fisher_json.as_array().unwrap().len(), 1);
    assert_eq!(fisher_json[0]["id"], fisher.id.to_string());
}

#[tokio::test]
async fn save_does_not_publish() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media_root = tempfile::tempdir().unwrap();
    let app = router_with_stores(
        ControllerConfig::for_test(false),
        repository,
        Arc::new(LocalMediaStore::new(media_root.path().to_path_buf())),
    );

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api/items")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "title": "Private Backlog Item",
                        "signer": "New Signer",
                        "category": "Cards",
                        "tags": ["backlog"],
                        "publicationStatus": "draft"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let create_json = response_json(create).await;
    assert_json_true(&create_json["pendingChanges"]["hasPendingChanges"]);
    let item_id = create_json["id"].as_str().unwrap();

    let status_after_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api/publish/status")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status_json = response_json(status_after_create).await;
    assert_eq!(status_json["state"], "idle");

    let patch = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/api/items/{item_id}"))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "title": "Private Backlog Item Updated",
                        "publicationStatus": "published"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch.status(), StatusCode::OK);
    let patch_json = response_json(patch).await;
    assert_json_true(&patch_json["pendingChanges"]["hasPendingChanges"]);
    assert!(patch_json["pendingChanges"]["count"].as_u64().unwrap() >= 2);

    let status_after_patch = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api/publish/status")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status_json = response_json(status_after_patch).await;
    assert_eq!(status_json["state"], "idle");
}

#[tokio::test]
async fn image_upload_response_includes_pending_changes() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let item = repository
        .create(test_item_input(
            "Upload Pending Item",
            "Ashley Eckstein",
            "Photos",
            vec!["ahsoka"],
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();
    let media_root = tempfile::tempdir().unwrap();
    let app = router_with_stores(
        ControllerConfig::for_test(false),
        repository,
        Arc::new(LocalMediaStore::new(media_root.path().to_path_buf())),
    );

    let boundary = "autographs-test-boundary";
    let png = png_fixture();
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"altText\"\r\n\r\nUploaded test image\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"upload.png\"\r\nContent-Type: image/png\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(&png);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/api/items/{}/images", item.id))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let response_json = response_json(response).await;
    assert_json_true(&response_json["pendingChanges"]["hasPendingChanges"]);
    assert!(response_json["pendingChanges"]["count"].as_u64().unwrap() >= 2);
    assert_eq!(response_json["images"][0]["altText"], "Uploaded test image");
}

#[tokio::test]
async fn admin_status_reports_pending_publish_cleanup_and_retention() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let item = repository
        .create(test_item_input(
            "Status Pending Item",
            "Billy Dee Williams",
            "Photos",
            vec!["lando"],
            PublicationStatus::Published,
        ))
        .await
        .unwrap();
    let image_id = uuid::Uuid::new_v4();
    repository
        .record_cleanup_event(ImageCleanupEvent::new(
            item.id,
            image_id,
            "originals/private/leaked-key",
            "delete",
            CleanupStatus::DeleteFailed,
            "Cleanup needs attention. Review the affected item before publishing again.",
            item.updated_at_epoch_seconds,
        ))
        .await
        .unwrap();
    let media_root = tempfile::tempdir().unwrap();
    let app = router_with_stores(
        ControllerConfig::for_test(false),
        repository,
        Arc::new(LocalMediaStore::new(media_root.path().to_path_buf())),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api/status")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_string(response).await;
    assert_redacted(&body);
    for denied in [
        "objectKey",
        "bucketName",
        "storageNamespace",
        "ORACLE_DB",
        "OCI_",
        "objectstorage",
        "originals/",
        "AUTOGRAPHS_ADMIN_PASSWORD",
    ] {
        assert!(!body.contains(denied), "status leaked {denied}: {body}");
    }
    let json: Value = serde_json::from_str(&body).unwrap();
    assert!(json.get("providers").is_some());
    assert!(json.get("publish").is_some());
    assert!(json.get("pendingChanges").is_some());
    assert!(json.get("cleanup").is_some());
    assert!(json.get("releaseRetention").is_some());
    assert_eq!(json["providers"]["database"], "local");
    assert_eq!(json["providers"]["media"], "local");
    assert_eq!(json["publish"]["state"], "idle");
    assert!(json["pendingChanges"]["count"].as_u64().unwrap() > 0);
    assert_eq!(json["cleanup"]["warningCount"], 1);
    assert_eq!(json["releaseRetention"]["promotedReleaseRetainCount"], 5);
    assert_eq!(json["releaseRetention"]["failedCandidateRetainCount"], 1);
    assert_eq!(
        json["liveSmokeGuidance"],
        "Run live smoke from docs/static-runtime-runbook.md when Oracle/Object Storage behavior changes."
    );
    assert_eq!(
        json["cleanupGuidance"],
        "Cleanup warnings must be resolved before trusting a publish batch."
    );
}

#[tokio::test]
async fn publish_batches_saved_changes() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media_root = tempfile::tempdir().unwrap();
    let static_root = tempfile::tempdir().unwrap();
    let mut config = ControllerConfig::for_test(false);
    config.static_release_root = static_root.path().to_path_buf();
    let app = router_with_stores(
        config,
        repository,
        Arc::new(LocalMediaStore::new(media_root.path().to_path_buf())),
    );

    let first = create_item(&app, "Batch Item One", "Carrie Fisher").await;
    let second = create_item(&app, "Batch Item Two", "Mark Hamill").await;

    patch_item_title(&app, first, "Batch Item One Updated").await;
    patch_item_title(&app, second, "Batch Item Two Updated").await;

    let before_publish = admin_status(&app).await;
    assert!(before_publish["pendingChanges"]["count"].as_u64().unwrap() >= 4);

    let publish = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api/publish/incremental")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish.status(), StatusCode::CREATED);
    let publish_json = response_json(publish).await;
    assert_eq!(publish_json["state"], "succeeded");

    let after_publish = admin_status(&app).await;
    assert_eq!(after_publish["pendingChanges"]["count"], 0);
    assert_eq!(after_publish["pendingChanges"]["hasPendingChanges"], false);
    assert_eq!(after_publish["publish"]["state"], "succeeded");

    let list = response_json(
        app.clone()
            .oneshot(
                Request::get("/admin/api/items")
                    .header(header::AUTHORIZATION, "Bearer operator-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
    )
    .await;
    for item_id in [first, second] {
        let item = list
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == item_id.to_string())
            .expect("published item in list");
        assert_eq!(item["hasPendingChanges"], false);

        let detail = response_json(
            app.clone()
                .oneshot(
                    Request::get(format!("/admin/api/items/{item_id}"))
                        .header(header::AUTHORIZATION, "Bearer operator-test-token")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(detail["pendingChanges"]["hasPendingChanges"], false);
        assert_eq!(detail["pendingChanges"]["count"], 0);
    }
}

#[tokio::test]
async fn publish_keeps_in_flight_same_second_edit_pending() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media_root = tempfile::tempdir().unwrap();
    let static_root = tempfile::tempdir().unwrap();
    let media = Arc::new(BlockingReadMediaStore::new(media_root.path()));
    let item = repository
        .create(test_item_input(
            "In Flight Item",
            "Rosario Dawson",
            "Photos",
            vec!["ahsoka"],
            PublicationStatus::Published,
        ))
        .await
        .unwrap();
    let image_id = uuid::Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    media.write(&object_key, &png_fixture()).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "private-flight.png".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: 128,
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();
    let mut config = ControllerConfig::for_test(false);
    config.static_release_root = static_root.path().to_path_buf();
    let app = router_with_stores(config, repository, media.clone());

    let publish_app = app.clone();
    let publish = tokio::spawn(async move {
        publish_app
            .oneshot(
                Request::post("/admin/api/publish/full")
                    .header(header::AUTHORIZATION, "Bearer operator-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    });
    media.wait_for_blocked_read().await;

    patch_item_title(&app, item.id, "In Flight Item Updated").await;
    media.release_read();

    let publish = publish.await.unwrap();
    assert_eq!(publish.status(), StatusCode::CREATED);
    let status = admin_status(&app).await;
    assert!(status["pendingChanges"]["count"].as_u64().unwrap() > 0);
    assert_eq!(status["pendingChanges"]["hasPendingChanges"], true);

    let detail = response_json(
        app.oneshot(
            Request::get(format!("/admin/api/items/{}", item.id))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap(),
    )
    .await;
    assert_eq!(detail["title"], "In Flight Item Updated");
    assert_eq!(detail["pendingChanges"]["hasPendingChanges"], true);
}

#[tokio::test]
async fn admin_status_reports_safe_publish_error_without_private_media_details() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let static_root = tempfile::tempdir().unwrap();
    let item = repository
        .create(test_item_input(
            "Leaky Media Item",
            "Temuera Morrison",
            "Photos",
            vec!["bounty"],
            PublicationStatus::Published,
        ))
        .await
        .unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: uuid::Uuid::new_v4(),
                object_key: "originals/private/leaky-object-key.png".to_owned(),
                original_filename: "private-leak.png".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: 42,
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();
    let mut config = ControllerConfig::for_test(false);
    config.static_release_root = static_root.path().to_path_buf();
    let app = router_with_stores(config, repository, Arc::new(LeakyFailingReadMediaStore));

    let publish = app
        .clone()
        .oneshot(
            Request::post("/admin/api/publish/full")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let status = app
        .oneshot(
            Request::get("/admin/api/status")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(status.status(), StatusCode::OK);
    let body = response_string(status).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        json["publish"]["error"],
        "Static publish failed. Check controller logs for details."
    );
    for denied in [
        "https://objectstorage.us-ashburn-1.oraclecloud.com",
        "objectstorage",
        "private-namespace",
        "private-bucket",
        "originals/",
        "leaky-object-key",
        "OCI_",
        "ORACLE_DB",
    ] {
        assert!(!body.contains(denied), "status leaked {denied}: {body}");
    }
}

#[tokio::test]
async fn history_metadata_and_publication_updates_record_field_level_diffs() {
    let repository = MemoryCatalogRepository::default();
    let item = repository
        .create(AutographItemInput {
            title: "Signed Jedi Card".to_owned(),
            signer: "Mark Hamill".to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: vec!["jedi".to_owned()],
            object_reference: None,
            event_name: None,
            event_location: None,
            source: None,
            inscription: None,
            certification_company: None,
            certification_id: None,
            estimated_year: None,
            publication_status: PublicationStatus::Draft,
        })
        .await
        .unwrap();

    repository
        .update(
            item.id,
            AutographItemUpdate {
                signer: Some("Carrie Fisher".to_owned()),
                category: Some("Photos".to_owned()),
                tags: Some(vec!["princess".to_owned(), "rebellion".to_owned()]),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    repository
        .update(
            item.id,
            AutographItemUpdate {
                title: Some("Published Jedi Card".to_owned()),
                publication_status: Some(PublicationStatus::Published),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let history = repository.history(item.id).await.unwrap();
    let metadata_event = history
        .iter()
        .find(|event| event.kind == EditEventKind::MetadataUpdated)
        .expect("metadata history event");
    assert_field_diff(
        metadata_event,
        "signer",
        json!("Mark Hamill"),
        json!("Carrie Fisher"),
    );
    assert_field_diff(metadata_event, "category", json!("Cards"), json!("Photos"));
    assert_field_diff(
        metadata_event,
        "tags",
        json!(["jedi"]),
        json!(["princess", "rebellion"]),
    );

    let publication_event = history
        .iter()
        .find(|event| event.kind == EditEventKind::PublicationChanged)
        .expect("publication history event");
    assert_field_diff(
        publication_event,
        "publicationStatus",
        json!("draft"),
        json!("published"),
    );
    assert_field_diff(
        publication_event,
        "title",
        json!("Signed Jedi Card"),
        json!("Published Jedi Card"),
    );
}

#[tokio::test]
async fn history_pending_changes_reports_count_and_oldest_change_timestamp() {
    let repository = MemoryCatalogRepository::default();
    let item = repository
        .create(AutographItemInput {
            title: "Signed Jedi Card".to_owned(),
            signer: "Mark Hamill".to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: Vec::new(),
            object_reference: None,
            event_name: None,
            event_location: None,
            source: None,
            inscription: None,
            certification_company: None,
            certification_id: None,
            estimated_year: None,
            publication_status: PublicationStatus::Draft,
        })
        .await
        .unwrap();
    repository
        .update(
            item.id,
            AutographItemUpdate {
                title: Some("Signed Jedi Card - private edit".to_owned()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let pending = repository.pending_changes().await.unwrap();
    assert!(pending.count > 0);
    assert!(pending.oldest_changed_at_epoch_seconds.is_some());
    assert!(pending.oldest_changed_at_epoch_seconds.unwrap() <= item.created_at_epoch_seconds);
}

fn test_item_input(
    title: &str,
    signer: &str,
    category: &str,
    tags: Vec<&str>,
    publication_status: PublicationStatus,
) -> AutographItemInput {
    AutographItemInput {
        title: title.to_owned(),
        signer: signer.to_owned(),
        description: None,
        category: category.to_owned(),
        tags: tags.into_iter().map(str::to_owned).collect(),
        object_reference: None,
        event_name: None,
        event_location: None,
        source: None,
        inscription: None,
        certification_company: None,
        certification_id: None,
        estimated_year: None,
        publication_status,
    }
}

async fn create_item(app: &axum::Router, title: &str, signer: &str) -> uuid::Uuid {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api/items")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "title": title,
                        "signer": signer,
                        "category": "Cards",
                        "tags": ["batch"],
                        "publicationStatus": "published"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response_json(response).await;
    uuid::Uuid::parse_str(body["id"].as_str().unwrap()).unwrap()
}

async fn patch_item_title(app: &axum::Router, item_id: uuid::Uuid, title: &str) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/api/items/{item_id}"))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({ "title": title }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

async fn admin_status(app: &axum::Router) -> Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api/status")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response).await
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
}

async fn response_string(response: axum::response::Response) -> String {
    String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}

fn assert_json_true(value: &Value) {
    assert!(value.as_bool().is_some_and(|value| value));
}

fn png_fixture() -> Vec<u8> {
    let mut body = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(RgbImage::from_pixel(8, 8, Rgb([1, 2, 3])))
        .write_to(&mut body, ImageFormat::Png)
        .unwrap();
    body.into_inner()
}

fn assert_redacted(body: &str) {
    for denied in [
        "objectKey",
        "bucketName",
        "storageNamespace",
        "originalFilename",
        "OCI_",
        "objectstorage",
    ] {
        assert!(
            !body.contains(denied),
            "admin response leaked {denied}: {body}"
        );
    }
}

fn assert_field_diff(
    event: &autographs_controller::catalog::AutographEditEvent,
    field: &str,
    before: Value,
    after: Value,
) {
    let diff = event
        .field_diffs
        .iter()
        .find(|diff| diff.field == field)
        .unwrap_or_else(|| panic!("missing diff for {field}"));
    assert_eq!(diff.before, before);
    assert_eq!(diff.after, after);
}

struct BlockingReadMediaStore {
    inner: LocalMediaStore,
    should_block: AtomicBool,
    blocked: Notify,
    release: Notify,
}

impl BlockingReadMediaStore {
    fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            inner: LocalMediaStore::new(root),
            should_block: AtomicBool::new(true),
            blocked: Notify::new(),
            release: Notify::new(),
        }
    }

    async fn wait_for_blocked_read(&self) {
        self.blocked.notified().await;
    }

    fn release_read(&self) {
        self.release.notify_waiters();
    }
}

#[async_trait]
impl PrivateMediaStore for BlockingReadMediaStore {
    async fn write(&self, object_key: &str, body: &[u8]) -> Result<(), String> {
        self.inner.write(object_key, body).await
    }

    async fn read(&self, object_key: &str) -> Result<Vec<u8>, String> {
        if self.should_block.swap(false, Ordering::SeqCst) {
            self.blocked.notify_one();
            self.release.notified().await;
        }
        self.inner.read(object_key).await
    }

    async fn delete(&self, object_key: &str) -> Result<(), String> {
        self.inner.delete(object_key).await
    }
}

struct LeakyFailingReadMediaStore;

#[async_trait]
impl PrivateMediaStore for LeakyFailingReadMediaStore {
    async fn write(&self, _object_key: &str, _body: &[u8]) -> Result<(), String> {
        Ok(())
    }

    async fn read(&self, object_key: &str) -> Result<Vec<u8>, String> {
        Err(format!(
            "GET https://objectstorage.us-ashburn-1.oraclecloud.com/n/private-namespace/b/private-bucket/o/{object_key} failed with OCI_MEDIA_BUCKET_NAME and ORACLE_DB_CONNECT_STRING"
        ))
    }

    async fn delete(&self, _object_key: &str) -> Result<(), String> {
        Ok(())
    }
}
