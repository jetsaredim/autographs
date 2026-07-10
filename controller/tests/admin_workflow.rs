use async_trait::async_trait;
use autographs_controller::{
    catalog::{
        AutographImage, AutographItemInput, AutographItemUpdate, CatalogRepository, CleanupStatus,
        EditEventKind, ImageCleanupEvent, ItemOrigin, MemoryCatalogRepository, PublicationStatus,
        SignerCreditInput, SignerProfileUpdateInput,
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
            signer_credits: Vec::new(),
            characters: Vec::new(),
            format: "Trading Card".to_owned(),
            origin: ItemOrigin::Official,
            franchises: Vec::new(),
            product_line: None,
            set_name: None,
            language: "English".to_owned(),
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
            signer_credits: Vec::new(),
            characters: Vec::new(),
            format: "Trading Card".to_owned(),
            origin: ItemOrigin::Official,
            franchises: Vec::new(),
            product_line: None,
            set_name: None,
            language: "English".to_owned(),
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
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/api/items/{}", item.id))
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_item_validation_errors_return_bad_request() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let item = repository
        .create(test_item_input(
            "Signed Jedi Card",
            "Mark Hamill",
            "Cards",
            vec!["jedi"],
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

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/api/items/{}", item.id))
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"language":"Klingon"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn signer_profile_urls_must_be_https_profile_hosts() {
    let repository = MemoryCatalogRepository::default();

    let javascript_url = repository
        .create(AutographItemInput {
            signer_credits: vec![SignerCreditInput {
                display_name: Some("Example Signer".to_owned()),
                wikipedia_url: Some("javascript:alert(1)".to_owned()),
                ..Default::default()
            }],
            ..test_item_input(
                "Signed Card",
                "Example Signer",
                "Cards",
                Vec::new(),
                PublicationStatus::Draft,
            )
        })
        .await;
    assert_eq!(
        javascript_url.unwrap_err(),
        "wikipediaUrl must be an https URL"
    );

    let wrong_host = repository
        .create(AutographItemInput {
            signer_credits: vec![SignerCreditInput {
                display_name: Some("Example Signer".to_owned()),
                imdb_url: Some("https://example.test/name/nm0000000/".to_owned()),
                ..Default::default()
            }],
            ..test_item_input(
                "Signed Card",
                "Example Signer",
                "Cards",
                Vec::new(),
                PublicationStatus::Draft,
            )
        })
        .await;
    assert_eq!(wrong_host.unwrap_err(), "imdbUrl must point to imdb.com");
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
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api/items?signer=fisher&category=photos&publicationStatus=published")
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
async fn admin_signer_and_taxonomy_routes_require_session_and_return_redacted_payloads() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let typo_item = repository
        .create(test_item_input(
            "Typo Signed Card",
            "Mark Hamel",
            "Cards",
            Vec::new(),
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();
    let target_item = repository
        .create(test_item_input(
            "Correct Signed Card",
            "Mark Hamill",
            "Cards",
            Vec::new(),
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();
    let source_signer_id = typo_item.signer_credits[0].signer.id;
    let target_signer_id = target_item.signer_credits[0].signer.id;
    let media_root = tempfile::tempdir().unwrap();
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository,
        Arc::new(LocalMediaStore::new(media_root.path().to_path_buf())),
    );

    for (method, uri, body) in [
        ("GET", "/admin/api/signers?query=mark", Body::empty()),
        ("GET", "/admin/api/taxonomy/suggestions", Body::empty()),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(header::AUTHORIZATION, "Bearer operator-test-token")
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    let bearer_patch = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/api/signers/{target_signer_id}"))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"defaultRole":"actor"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bearer_patch.status(), StatusCode::UNAUTHORIZED);

    let bearer_merge = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api/signers/merge")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "sourceSignerId": source_signer_id,
                        "targetSignerId": target_signer_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bearer_merge.status(), StatusCode::UNAUTHORIZED);

    let cookie = admin_cookie(&app).await;
    let suggestions = app
        .clone()
        .oneshot(
            Request::get("/admin/api/signers?query=Mark%20Hamel")
                .header(header::COOKIE, cookie.as_str())
                .header(header::ORIGIN, "https://autographs.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(suggestions.status(), StatusCode::OK);
    let suggestions_body = response_string(suggestions).await;
    assert_redacted(&suggestions_body);
    let suggestions_json: Value = serde_json::from_str(&suggestions_body).unwrap();
    assert_eq!(
        suggestions_json["suggestions"][0]["profile"]["displayName"],
        "Mark Hamel"
    );
    assert_json_true(&suggestions_json["suggestions"][0]["possibleDuplicate"]);

    let taxonomy = app
        .clone()
        .oneshot(
            Request::get("/admin/api/taxonomy/suggestions")
                .header(header::COOKIE, cookie.as_str())
                .header(header::ORIGIN, "https://autographs.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(taxonomy.status(), StatusCode::OK);
    let taxonomy_body = response_string(taxonomy).await;
    assert_redacted(&taxonomy_body);
    let taxonomy_json: Value = serde_json::from_str(&taxonomy_body).unwrap();
    assert!(
        taxonomy_json["formats"]
            .as_array()
            .unwrap()
            .contains(&json!("Trading Card"))
    );
    assert!(
        taxonomy_json["languages"]
            .as_array()
            .unwrap()
            .contains(&json!("English"))
    );

    let merge = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api/signers/merge")
                .header(header::COOKIE, cookie.as_str())
                .header(header::ORIGIN, "https://autographs.example.test")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "sourceSignerId": source_signer_id,
                        "targetSignerId": target_signer_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(merge.status(), StatusCode::OK);
    let merge_body = response_string(merge).await;
    assert_redacted(&merge_body);
    let merge_json: Value = serde_json::from_str(&merge_body).unwrap();
    assert_eq!(merge_json["sourceSignerId"], source_signer_id.to_string());
    assert_eq!(merge_json["targetSignerId"], target_signer_id.to_string());
    assert_eq!(merge_json["updatedItemCount"], 1);
}

#[tokio::test]
async fn admin_item_list_filters_and_summaries_use_taxonomy_fields() {
    let repository = Arc::new(MemoryCatalogRepository::default());
    let hamill = repository
        .create(AutographItemInput {
            signer_credits: vec![SignerCreditInput {
                display_name: Some("Mark Hamill".to_owned()),
                default_role: Some("actor".to_owned()),
                ..Default::default()
            }],
            characters: vec!["Luke Skywalker".to_owned()],
            franchises: vec!["Star Wars".to_owned()],
            product_line: Some("Star Wars CCG".to_owned()),
            set_name: Some("Premiere".to_owned()),
            format: "Trading Card".to_owned(),
            language: "Japanese".to_owned(),
            tags: vec!["jedi".to_owned()],
            ..test_item_input(
                "Signed Jedi Card",
                "Mark Hamill",
                "Cards",
                Vec::new(),
                PublicationStatus::Draft,
            )
        })
        .await
        .unwrap();
    repository
        .create(AutographItemInput {
            signer_credits: vec![SignerCreditInput {
                display_name: Some("Carrie Fisher".to_owned()),
                default_role: Some("actor".to_owned()),
                ..Default::default()
            }],
            franchises: vec!["Star Wars".to_owned()],
            product_line: Some("Galactic Files".to_owned()),
            format: "Comic Book".to_owned(),
            language: "English".to_owned(),
            ..test_item_input(
                "Signed Rebel Comic",
                "Carrie Fisher",
                "Comics",
                Vec::new(),
                PublicationStatus::Published,
            )
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
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api/items?signer=hamill&franchise=star&productLine=ccg&format=trading&language=japanese")
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_string(response).await;
    assert_redacted(&body);
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 1);
    let summary = &json[0];
    assert_eq!(summary["id"], hamill.id.to_string());
    assert_eq!(summary["signerText"], "Mark Hamill");
    assert_eq!(summary["signerNames"], json!(["Mark Hamill"]));
    assert_eq!(summary["franchises"], json!(["Star Wars"]));
    assert_eq!(summary["productLine"], "Star Wars CCG");
    assert_eq!(summary["format"], "Trading Card");
    assert_eq!(summary["language"], "Japanese");
    assert_eq!(summary["publicationStatus"], "draft");
    assert!(summary.get("category").is_none());
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
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api/publish/status")
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/api/items/{}/images", item.id))
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api/status")
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
    assert_eq!(
        json["cleanup"]["warnings"][0]["title"],
        "Status Pending Item"
    );
    assert_eq!(json["cleanup"]["warnings"][0]["operation"], "delete");
    assert_eq!(json["cleanup"]["warnings"][0]["status"], "deleteFailed");
    assert_eq!(
        json["cleanup"]["warnings"][0]["adminMessage"],
        "Cleanup needs attention. Review the affected item before publishing again."
    );
    assert!(json["cleanup"]["warnings"][0]["imageId"].is_string());
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
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
                    .header(header::COOKIE, admin_cookie(&app).await)
                    .header(header::ORIGIN, "https://autographs.example.test")
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
                        .header(header::COOKIE, admin_cookie(&app).await)
                        .header(header::ORIGIN, "https://autographs.example.test")
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
async fn publish_clears_same_second_saved_change_included_in_release() {
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

    let item_id = create_item(&app, "Included Save Item", "Daisy Ridley").await;
    patch_item_title(&app, item_id, "Included Save Item Updated").await;
    let before_publish = admin_status(&app).await;
    assert_eq!(before_publish["pendingChanges"]["hasPendingChanges"], true);

    let publish = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api/publish/full")
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish.status(), StatusCode::CREATED);

    let status = admin_status(&app).await;
    assert_eq!(status["pendingChanges"]["count"], 0);
    assert_eq!(status["pendingChanges"]["hasPendingChanges"], false);

    let detail = response_json(
        app.clone()
            .oneshot(
                Request::get(format!("/admin/api/items/{item_id}"))
                    .header(header::COOKIE, admin_cookie(&app).await)
                    .header(header::ORIGIN, "https://autographs.example.test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(detail["pendingChanges"]["count"], 0);
    assert_eq!(detail["pendingChanges"]["hasPendingChanges"], false);
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
    let publish_cookie = admin_cookie(&app).await;
    let publish = tokio::spawn(async move {
        publish_app
            .oneshot(
                Request::post("/admin/api/publish/full")
                    .header(header::COOKIE, publish_cookie)
                    .header(header::ORIGIN, "https://autographs.example.test")
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
        app.clone()
            .oneshot(
                Request::get(format!("/admin/api/items/{}", item.id))
                    .header(header::COOKIE, admin_cookie(&app).await)
                    .header(header::ORIGIN, "https://autographs.example.test")
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
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let status = app
        .clone()
        .oneshot(
            Request::get("/admin/api/status")
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
            signer_credits: Vec::new(),
            characters: Vec::new(),
            format: "Trading Card".to_owned(),
            origin: ItemOrigin::Official,
            franchises: Vec::new(),
            product_line: None,
            set_name: None,
            language: "English".to_owned(),
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
async fn taxonomy_updates_record_metadata_diffs() {
    let repository = MemoryCatalogRepository::default();
    let item = repository
        .create(AutographItemInput {
            title: "Signed Jedi Card".to_owned(),
            signer: "Mark Hamill".to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: vec!["jedi".to_owned()],
            signer_credits: vec![SignerCreditInput {
                signer_id: None,
                display_name: Some("Mark Hamill".to_owned()),
                default_role: Some("actor".to_owned()),
                item_role: Some("actor".to_owned()),
                item_context: Some("Luke Skywalker".to_owned()),
                wikipedia_url: None,
                imdb_url: None,
            }],
            characters: vec!["Luke Skywalker".to_owned()],
            format: "Trading Card".to_owned(),
            origin: ItemOrigin::Official,
            franchises: vec!["Star Wars".to_owned()],
            product_line: Some("Star Wars CCG".to_owned()),
            set_name: Some("Premiere".to_owned()),
            language: "English".to_owned(),
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

    let update: AutographItemUpdate = serde_json::from_value(json!({
        "signerCredits": [{
            "displayName": "Carrie Fisher",
            "defaultRole": "actor",
            "itemRole": "actor",
            "itemContext": "Princess Leia"
        }],
        "characters": ["Princess Leia"],
        "franchises": ["Star Wars", "Star Wars Legends"],
        "productLine": "Star Wars Galactic Files",
        "setName": "Custom",
        "format": "Comic Book",
        "origin": "Custom",
        "language": "Japanese",
        "tags": ["rebellion"]
    }))
    .unwrap();
    let updated = repository.update(item.id, update).await.unwrap();

    assert_eq!(
        updated.signer_credits[0].signer.display_name,
        "Carrie Fisher"
    );
    assert_eq!(updated.characters, vec!["Princess Leia"]);
    assert_eq!(updated.franchises, vec!["Star Wars", "Star Wars Legends"]);
    assert_eq!(
        updated.product_line.as_deref(),
        Some("Star Wars Galactic Files")
    );
    assert_eq!(updated.set_name.as_deref(), Some("Custom"));
    assert_eq!(updated.format, "Comic Book");
    assert_eq!(updated.origin, ItemOrigin::Custom);
    assert_eq!(updated.language, "Japanese");

    let history = repository.history(item.id).await.unwrap();
    let metadata_event = history
        .iter()
        .find(|event| {
            event.kind == EditEventKind::MetadataUpdated
                && event.field_diffs.iter().any(|diff| diff.field == "signers")
        })
        .expect("taxonomy metadata history event");

    for field in [
        "signers",
        "characters",
        "franchises",
        "productLine",
        "setName",
        "format",
        "origin",
        "language",
        "tags",
    ] {
        assert!(
            metadata_event
                .field_diffs
                .iter()
                .any(|diff| diff.field == field),
            "missing taxonomy diff for {field}"
        );
    }
    assert!(metadata_event.summary.contains("signers"));
}

#[tokio::test]
async fn signer_suggestions_rank_duplicates_without_blocking_new_names() {
    let repository = MemoryCatalogRepository::default();
    let item = repository
        .create(AutographItemInput {
            title: "Signed Jedi Card".to_owned(),
            signer: "Mark Hamill".to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: Vec::new(),
            signer_credits: vec![SignerCreditInput {
                display_name: Some("Mark Hamill".to_owned()),
                default_role: Some("actor".to_owned()),
                ..Default::default()
            }],
            characters: Vec::new(),
            format: "Trading Card".to_owned(),
            origin: ItemOrigin::Official,
            franchises: Vec::new(),
            product_line: None,
            set_name: None,
            language: "English".to_owned(),
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
        .create(AutographItemInput {
            title: "Signed Rebel Card".to_owned(),
            signer: "Carrie Fisher".to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: Vec::new(),
            signer_credits: vec![SignerCreditInput {
                display_name: Some("Carrie Fisher".to_owned()),
                default_role: Some("actor".to_owned()),
                ..Default::default()
            }],
            characters: Vec::new(),
            format: "Trading Card".to_owned(),
            origin: ItemOrigin::Official,
            franchises: Vec::new(),
            product_line: None,
            set_name: None,
            language: "English".to_owned(),
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

    let exact = repository
        .signer_suggestions("Mark Hamill".to_owned())
        .await
        .unwrap();
    assert_eq!(exact[0].profile.id, item.signer_credits[0].signer.id);
    assert!(exact[0].possible_duplicate);

    let near = repository
        .signer_suggestions("Mark Hamel".to_owned())
        .await
        .unwrap();
    assert_eq!(near[0].profile.display_name, "Mark Hamill");
    assert!(near[0].possible_duplicate);

    let deliberate_new = repository
        .signer_suggestions("Ahmed Best".to_owned())
        .await
        .unwrap();
    assert!(deliberate_new.is_empty());
}

#[tokio::test]
async fn signer_profile_edits_record_history_for_linked_items_only() {
    let repository = MemoryCatalogRepository::default();
    let first = repository
        .create(test_item_input(
            "Signed Jedi Card",
            "Mark Hamill",
            "Cards",
            Vec::new(),
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();
    let signer_id = first.signer_credits[0].signer.id;
    let second = repository
        .create(AutographItemInput {
            signer_credits: vec![SignerCreditInput {
                signer_id: Some(signer_id),
                ..Default::default()
            }],
            ..test_item_input(
                "Signed Pilot Card",
                "Mark Hamill",
                "Cards",
                Vec::new(),
                PublicationStatus::Draft,
            )
        })
        .await
        .unwrap();
    let unrelated = repository
        .create(test_item_input(
            "Signed Rebel Card",
            "Carrie Fisher",
            "Cards",
            Vec::new(),
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();

    repository
        .update_signer_profile(
            signer_id,
            SignerProfileUpdateInput {
                display_name: Some("Mark Richard Hamill".to_owned()),
                default_role: Some("voice actor".to_owned()),
                wikipedia_url: Some("https://en.wikipedia.org/wiki/Mark_Hamill".to_owned()),
                imdb_url: Some("https://www.imdb.com/name/nm0000434/".to_owned()),
            },
        )
        .await
        .unwrap();

    for item_id in [first.id, second.id] {
        let history = repository.history(item_id).await.unwrap();
        let event = history
            .iter()
            .find(|event| {
                event
                    .summary
                    .contains("Updated signer profile Mark Hamill -> Mark Richard Hamill")
            })
            .expect("linked item signer profile event");
        for field in [
            "signerProfile.displayName",
            "signerProfile.defaultRole",
            "signerProfile.wikipediaUrl",
            "signerProfile.imdbUrl",
        ] {
            assert!(
                event.field_diffs.iter().any(|diff| diff.field == field),
                "missing profile edit diff for {field}"
            );
        }
    }
    assert!(
        repository
            .history(unrelated.id)
            .await
            .unwrap()
            .iter()
            .all(|event| !event.summary.contains("Updated signer profile"))
    );
}

#[tokio::test]
async fn item_signer_credit_rejects_conflicting_profile_id_and_display_name() {
    let repository = MemoryCatalogRepository::default();
    let item = repository
        .create(test_item_input(
            "Signed Jedi Card",
            "Mark Hamill",
            "Cards",
            Vec::new(),
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();
    let signer_id = item.signer_credits[0].signer.id;

    let update: AutographItemUpdate = serde_json::from_value(json!({
        "signerCredits": [{
            "signerId": signer_id,
            "displayName": "Carrie Fisher",
            "itemRole": "actor"
        }]
    }))
    .unwrap();
    let error = repository.update(item.id, update).await.unwrap_err();
    assert!(error.contains("signerId cannot be combined with a conflicting displayName"));

    let unchanged = repository.get(item.id).await.unwrap().unwrap();
    assert_eq!(
        unchanged.signer_credits[0].signer.display_name,
        "Mark Hamill"
    );
}

#[tokio::test]
async fn stale_signer_id_after_merge_does_not_recreate_source_profile() {
    let repository = MemoryCatalogRepository::default();
    let source_item = repository
        .create(test_item_input(
            "Typo Signed Card",
            "Mark Hamel",
            "Cards",
            Vec::new(),
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();
    let target_item = repository
        .create(test_item_input(
            "Canonical Signed Card",
            "Mark Hamill",
            "Cards",
            Vec::new(),
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();
    let source_signer_id = source_item.signer_credits[0].signer.id;
    let target_signer_id = target_item.signer_credits[0].signer.id;

    repository
        .merge_signer_profiles(source_signer_id, target_signer_id)
        .await
        .unwrap();

    let stale_update: AutographItemUpdate = serde_json::from_value(json!({
        "signerCredits": [{
            "signerId": source_signer_id,
            "displayName": "Mark Hamel",
            "itemRole": "actor"
        }]
    }))
    .unwrap();
    let error = repository
        .update(source_item.id, stale_update)
        .await
        .unwrap_err();
    assert_eq!(error, "signer profile was not found");

    let suggestions = repository
        .signer_suggestions("Mark Hamel".to_owned())
        .await
        .unwrap();
    assert!(
        suggestions
            .iter()
            .all(|suggestion| suggestion.profile.id != source_signer_id)
    );
}

#[tokio::test]
async fn merge_signer_profiles_moves_credits_and_records_history() {
    let repository = MemoryCatalogRepository::default();
    let source_item = repository
        .create(test_item_input(
            "Typo Signed Card",
            "Mark Hamel",
            "Cards",
            Vec::new(),
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();
    let target_item = repository
        .create(test_item_input(
            "Correct Signed Card",
            "Mark Hamill",
            "Cards",
            Vec::new(),
            PublicationStatus::Draft,
        ))
        .await
        .unwrap();
    let source_id = source_item.signer_credits[0].signer.id;
    let target_id = target_item.signer_credits[0].signer.id;

    let result = repository
        .merge_signer_profiles(source_id, target_id)
        .await
        .unwrap();
    assert_eq!(result.updated_item_count, 1);

    let updated = repository.get(source_item.id).await.unwrap().unwrap();
    assert_eq!(updated.signer_credits[0].signer.id, target_id);
    let history = repository.history(source_item.id).await.unwrap();
    assert!(history.iter().any(|event| {
        event.summary == "Merged signer Mark Hamel into Mark Hamill"
            && event.kind == EditEventKind::MetadataUpdated
    }));
}

#[tokio::test]
async fn taxonomy_suggestions_aggregate_existing_values() {
    let repository = MemoryCatalogRepository::default();
    repository
        .create(AutographItemInput {
            signer_credits: vec![SignerCreditInput {
                display_name: Some("Mark Hamill".to_owned()),
                default_role: Some("actor".to_owned()),
                ..Default::default()
            }],
            characters: vec!["Luke Skywalker".to_owned()],
            franchises: vec!["Star Wars".to_owned()],
            product_line: Some("Star Wars CCG".to_owned()),
            set_name: Some("Premiere".to_owned()),
            language: "Japanese".to_owned(),
            tags: vec!["jedi".to_owned()],
            ..test_item_input(
                "Signed Jedi Card",
                "Mark Hamill",
                "Cards",
                Vec::new(),
                PublicationStatus::Draft,
            )
        })
        .await
        .unwrap();

    let suggestions = repository.taxonomy_suggestions().await.unwrap();
    assert_eq!(suggestions.characters, vec!["Luke Skywalker"]);
    assert_eq!(suggestions.franchises, vec!["Star Wars"]);
    assert_eq!(suggestions.product_lines, vec!["Star Wars CCG"]);
    assert_eq!(suggestions.set_names, vec!["Premiere"]);
    assert!(suggestions.formats.contains(&"Trading Card".to_owned()));
    assert!(suggestions.languages.contains(&"Japanese".to_owned()));
    assert!(suggestions.roles.contains(&"actor".to_owned()));
    assert_eq!(suggestions.tags, vec!["jedi"]);
}

#[tokio::test]
async fn direct_taxonomy_payloads_are_trimmed_and_deduplicated() {
    let repository = MemoryCatalogRepository::default();
    let item = repository
        .create(AutographItemInput {
            characters: vec![
                "Luke Skywalker".to_owned(),
                " Luke Skywalker ".to_owned(),
                " ".to_owned(),
            ],
            franchises: vec![
                "Star Wars".to_owned(),
                "Star Wars".to_owned(),
                "Star Wars Legends".to_owned(),
            ],
            tags: vec!["jedi".to_owned(), "jedi".to_owned(), " hero ".to_owned()],
            ..test_item_input(
                "Signed Jedi Card",
                "Mark Hamill",
                "Cards",
                Vec::new(),
                PublicationStatus::Draft,
            )
        })
        .await
        .unwrap();

    assert_eq!(item.characters, vec!["Luke Skywalker"]);
    assert_eq!(item.franchises, vec!["Star Wars", "Star Wars Legends"]);
    assert_eq!(item.tags, vec!["jedi", "hero"]);

    let update: AutographItemUpdate = serde_json::from_value(json!({
        "characters": ["Princess Leia", " Princess Leia ", "General Organa"],
        "franchises": ["Star Wars", "Star Wars"],
        "tags": ["rebellion", "rebellion", " hero "]
    }))
    .unwrap();
    let updated = repository.update(item.id, update).await.unwrap();

    assert_eq!(updated.characters, vec!["Princess Leia", "General Organa"]);
    assert_eq!(updated.franchises, vec!["Star Wars"]);
    assert_eq!(updated.tags, vec!["rebellion", "hero"]);
}

#[test]
fn autograph_item_input_deserializes_camel_case_taxonomy_fields() {
    let input: AutographItemInput = serde_json::from_value(json!({
        "title": "Signed Jedi Card",
        "signer": "Mark Hamill",
        "category": "Cards",
        "signerCredits": [{
            "displayName": "Mark Hamill",
            "defaultRole": "actor",
            "itemRole": "actor",
            "itemContext": "Luke Skywalker"
        }],
        "characters": ["Luke Skywalker"],
        "productLine": "Star Wars CCG",
        "setName": "Premiere",
        "format": "Trading Card",
        "origin": "Official",
        "language": "English",
        "franchises": ["Star Wars"],
        "tags": ["jedi"]
    }))
    .unwrap();

    assert_eq!(input.signer_credits.len(), 1);
    assert_eq!(input.characters, vec!["Luke Skywalker"]);
    assert_eq!(input.product_line.as_deref(), Some("Star Wars CCG"));
    assert_eq!(input.set_name.as_deref(), Some("Premiere"));
    assert_eq!(input.format, "Trading Card");
    assert_eq!(input.origin, ItemOrigin::Official);
    assert_eq!(input.language, "English");
    assert_eq!(input.franchises, vec!["Star Wars"]);
    assert_eq!(input.tags, vec!["jedi"]);
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
            signer_credits: Vec::new(),
            characters: Vec::new(),
            format: "Trading Card".to_owned(),
            origin: ItemOrigin::Official,
            franchises: Vec::new(),
            product_line: None,
            set_name: None,
            language: "English".to_owned(),
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
        signer_credits: Vec::new(),
        characters: Vec::new(),
        format: "Trading Card".to_owned(),
        origin: ItemOrigin::Official,
        franchises: Vec::new(),
        product_line: None,
        set_name: None,
        language: "English".to_owned(),
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
                .header(header::COOKIE, admin_cookie(app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
                .header(header::COOKIE, admin_cookie(app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
                .header(header::COOKIE, admin_cookie(app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response).await
}

async fn admin_cookie(app: &axum::Router) -> String {
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
        .expect("set-cookie")
        .to_str()
        .expect("set-cookie text")
        .split(';')
        .next()
        .expect("cookie pair")
        .to_owned()
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
