use autographs_controller::{
    catalog::{
        AutographImage, AutographItemInput, AutographItemUpdate, CatalogRepository, EditEventKind,
        MemoryCatalogRepository, PublicationStatus,
    },
    config::ControllerConfig,
    media::LocalMediaStore,
    routes::router_with_stores,
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use std::sync::Arc;
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
    assert_eq!(list_json[0]["hasPendingChanges"], true);

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
    assert_eq!(detail_json["pendingChanges"]["hasPendingChanges"], true);

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
    assert_eq!(patch_json["pendingChanges"]["hasPendingChanges"], true);
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
    assert_eq!(create_json["pendingChanges"]["hasPendingChanges"], true);
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
    assert_eq!(patch_json["pendingChanges"]["hasPendingChanges"], true);
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
