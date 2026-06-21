use autographs_controller::{
    catalog::{
        AutographItemInput, AutographItemUpdate, CatalogRepository, EditEventKind,
        MemoryCatalogRepository, PublicationStatus,
    },
    config::ControllerConfig,
    media::LocalMediaStore,
    routes::router_with_stores,
};
use axum::{
    body::Body,
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
