use std::{
    fs,
    io::Cursor,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use autographs_controller::{
    catalog::{
        AutographImage, AutographItem, AutographItemInput, AutographItemUpdate, CatalogRepository,
        CleanupStatus, CleanupWarning, ImageCleanupEvent, ImageReplacementInput,
        MemoryCatalogRepository, PublicationStatus,
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
use serde_json::Value;
use tempfile::tempdir;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn supporting_upload_preserves_primary_and_primary_route_selects_one_image() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        Arc::new(LocalMediaStore::new(root.path())),
    );
    let item = repository.create(item_input()).await.unwrap();

    let first = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(first["images"][0]["isPrimary"], true);

    let second = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    let second_id = second["images"]
        .as_array()
        .unwrap()
        .iter()
        .find(|image| image["isPrimary"] == false)
        .and_then(|image| image["id"].as_str())
        .unwrap()
        .to_owned();
    assert_eq!(
        second["images"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|image| image["isPrimary"] == true)
            .count(),
        1
    );

    let primary = app
        .oneshot(
            Request::post(format!(
                "/admin/api/items/{}/images/{second_id}/primary",
                item.id
            ))
            .header(header::AUTHORIZATION, "Bearer operator-test-token")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary.status(), StatusCode::OK);
    let selected = response_json(primary).await;
    assert_eq!(
        selected["images"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|image| image["isPrimary"] == true)
            .count(),
        1
    );
    assert!(
        selected["images"]
            .as_array()
            .unwrap()
            .iter()
            .any(|image| image["id"] == second_id && image["isPrimary"] == true)
    );
}

#[tokio::test]
async fn first_upload_is_primary_even_when_false_is_requested() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        Arc::new(LocalMediaStore::new(root.path())),
    );
    let item = repository.create(item_input()).await.unwrap();

    let first = response_json(
        app.oneshot(upload_request(item.id, Some(false)))
            .await
            .unwrap(),
    )
    .await;

    assert_eq!(first["images"][0]["isPrimary"], true);
}

#[tokio::test]
async fn delete_image_removes_private_object_and_metadata() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media = Arc::new(LocalMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
    let item = repository.create(item_input()).await.unwrap();
    let uploaded = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    let image_id = Uuid::parse_str(uploaded["images"][0]["id"].as_str().unwrap()).unwrap();
    let object_key = build_original_object_key(item.id, image_id);
    assert_eq!(media.read(&object_key).await.unwrap(), png_fixture());

    let deleted = app
        .oneshot(
            Request::delete(format!("/admin/api/items/{}/images/{image_id}", item.id))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::OK);
    assert!(
        repository
            .get(item.id)
            .await
            .unwrap()
            .unwrap()
            .images
            .is_empty()
    );
    assert!(media.read(&object_key).await.is_err());
    assert_eq!(file_count(root.path()), 0);
}

#[tokio::test]
async fn delete_image_failure_keeps_metadata_and_returns_redacted_cleanup_warning() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media = Arc::new(FailingDeleteMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
    let item = repository.create(item_input()).await.unwrap();
    let uploaded = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    let image_id = Uuid::parse_str(uploaded["images"][0]["id"].as_str().unwrap()).unwrap();
    media.fail_deletes(true);

    let deleted = app
        .oneshot(
            Request::delete(format!("/admin/api/items/{}/images/{image_id}", item.id))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::CONFLICT);
    let rendered = String::from_utf8(
        to_bytes(deleted.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(rendered.contains("cleanupWarning"));
    for denied in [
        "originals/",
        "objectKey",
        "bucketName",
        "storageNamespace",
        "objectstorage",
        "private.jpg",
    ] {
        assert!(
            !rendered.contains(denied),
            "cleanup warning leaked {denied}"
        );
    }
    assert_eq!(
        repository.get(item.id).await.unwrap().unwrap().images.len(),
        1
    );
    assert_eq!(repository.cleanup_warnings(item.id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn delete_metadata_failure_after_media_delete_records_retryable_warning() {
    let root = tempdir().unwrap();
    let item_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item_id, image_id);
    let item = item_with_image(item_id, image_id, object_key.clone());
    let repository = Arc::new(FailingRemoveRepository {
        item: Mutex::new(item),
        cleanup_events: Mutex::new(Vec::new()),
    });
    let media = Arc::new(LocalMediaStore::new(root.path()));
    media.write(&object_key, &png_fixture()).await.unwrap();
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );

    let deleted = app
        .oneshot(
            Request::delete(format!("/admin/api/items/{item_id}/images/{image_id}"))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::CONFLICT);
    assert!(media.read(&object_key).await.is_err());
    assert_eq!(repository.cleanup_warnings(item_id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn replacement_rolls_back_new_object_when_metadata_swap_fails() {
    let root = tempdir().unwrap();
    let item_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();
    let old_key = build_original_object_key(item_id, image_id);
    let item = AutographItem {
        id: item_id,
        title: "Signed Card".into(),
        signer: "Signer".into(),
        description: None,
        category: "Cards".into(),
        tags: vec![],
        object_reference: None,
        event_name: None,
        event_location: None,
        source: None,
        inscription: None,
        certification_company: None,
        certification_id: None,
        estimated_year: None,
        publication_status: PublicationStatus::Draft,
        images: vec![AutographImage {
            id: image_id,
            object_key: old_key.clone(),
            original_filename: "old.png".into(),
            content_type: "image/png".into(),
            byte_size: 3,
            is_primary: true,
            sort_order: 0,
            alt_text: None,
        }],
        created_at_epoch_seconds: 0,
        updated_at_epoch_seconds: 0,
    };
    let repository = Arc::new(FailingReplaceRepository { item });
    let media = Arc::new(LocalMediaStore::new(root.path()));
    media.write(&old_key, &png_fixture()).await.unwrap();
    let app = router_with_stores(ControllerConfig::for_test(true), repository, media);

    let replaced = app
        .oneshot(replace_request(item_id, image_id))
        .await
        .unwrap();

    assert_eq!(replaced.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(file_count(root.path()), 1);
}

#[tokio::test]
async fn replacement_cleanup_warning_is_visible_and_retryable_by_old_image_id() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media = Arc::new(FailingDeleteMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
    let item = repository.create(item_input()).await.unwrap();
    let uploaded = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    let old_image_id = Uuid::parse_str(uploaded["images"][0]["id"].as_str().unwrap()).unwrap();
    media.fail_deletes(true);

    let replaced = app
        .clone()
        .oneshot(replace_request(item.id, old_image_id))
        .await
        .unwrap();

    assert_eq!(replaced.status(), StatusCode::OK);
    let replaced_json = response_json(replaced).await;
    assert_eq!(
        replaced_json["cleanupWarnings"][0]["imageId"],
        old_image_id.to_string()
    );
    let new_image_id = Uuid::parse_str(replaced_json["images"][0]["id"].as_str().unwrap()).unwrap();
    assert_eq!(new_image_id, old_image_id);
    let replacement_key = repository.get(item.id).await.unwrap().unwrap().images[0]
        .object_key
        .clone();
    assert_ne!(
        replacement_key,
        build_original_object_key(item.id, old_image_id)
    );

    let detail = response_json(
        app.clone()
            .oneshot(
                Request::get(format!("/admin/api/items/{}", item.id))
                    .header(header::AUTHORIZATION, "Bearer operator-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        detail["cleanupWarnings"][0]["imageId"],
        old_image_id.to_string()
    );

    media.fail_deletes(false);
    media
        .delete(&build_original_object_key(item.id, old_image_id))
        .await
        .unwrap();
    let retried = app
        .oneshot(
            Request::post(format!(
                "/admin/api/items/{}/images/{old_image_id}/cleanup/retry",
                item.id
            ))
            .header(header::AUTHORIZATION, "Bearer operator-test-token")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retried.status(), StatusCode::OK);
    assert!(
        repository
            .cleanup_warnings(item.id)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(media.read(&replacement_key).await.is_ok());
    let item_after_retry = repository.get(item.id).await.unwrap().unwrap();
    assert_eq!(item_after_retry.images.len(), 1);
    assert_eq!(item_after_retry.images[0].id, old_image_id);
    assert_eq!(item_after_retry.images[0].object_key, replacement_key);
}

#[tokio::test]
async fn replacement_upload_rejects_images_over_twenty_mebibytes() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media = Arc::new(LocalMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
    let item = repository.create(item_input()).await.unwrap();
    let uploaded = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    let image_id = Uuid::parse_str(uploaded["images"][0]["id"].as_str().unwrap()).unwrap();

    let rejected = app
        .oneshot(replace_request_with_body(
            item.id,
            image_id,
            vec![0_u8; 20 * 1024 * 1024 + 1],
        ))
        .await
        .unwrap();

    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        repository.get(item.id).await.unwrap().unwrap().images.len(),
        1
    );
}

#[tokio::test]
async fn failed_replacement_cleanup_retry_preserves_replacement_target() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media = Arc::new(FailingDeleteMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
    let item = repository.create(item_input()).await.unwrap();
    let uploaded = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    let image_id = Uuid::parse_str(uploaded["images"][0]["id"].as_str().unwrap()).unwrap();
    media.fail_deletes(true);
    let replaced = app
        .clone()
        .oneshot(replace_request(item.id, image_id))
        .await
        .unwrap();
    assert_eq!(replaced.status(), StatusCode::OK);
    let replacement_key = repository.get(item.id).await.unwrap().unwrap().images[0]
        .object_key
        .clone();

    let failed_retry = app
        .clone()
        .oneshot(
            Request::post(format!(
                "/admin/api/items/{}/images/{image_id}/cleanup/retry",
                item.id
            ))
            .header(header::AUTHORIZATION, "Bearer operator-test-token")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(failed_retry.status(), StatusCode::CONFLICT);
    assert!(
        repository
            .cleanup_warnings(item.id)
            .await
            .unwrap()
            .iter()
            .all(|warning| warning.operation == "replace")
    );

    media.fail_deletes(false);
    let succeeded_retry = app
        .oneshot(
            Request::post(format!(
                "/admin/api/items/{}/images/{image_id}/cleanup/retry",
                item.id
            ))
            .header(header::AUTHORIZATION, "Bearer operator-test-token")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(succeeded_retry.status(), StatusCode::OK);
    let item_after_retry = repository.get(item.id).await.unwrap().unwrap();
    assert_eq!(item_after_retry.images.len(), 1);
    assert_eq!(item_after_retry.images[0].id, image_id);
    assert_eq!(item_after_retry.images[0].object_key, replacement_key);
    assert!(media.read(&replacement_key).await.is_ok());
    assert!(
        repository
            .cleanup_warnings(item.id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn second_replacement_cleanup_retry_deletes_previous_replacement_object() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media = Arc::new(FailingDeleteMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
    let item = repository.create(item_input()).await.unwrap();
    let uploaded = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    let image_id = Uuid::parse_str(uploaded["images"][0]["id"].as_str().unwrap()).unwrap();

    let first_replacement = app
        .clone()
        .oneshot(replace_request(item.id, image_id))
        .await
        .unwrap();
    assert_eq!(first_replacement.status(), StatusCode::OK);
    let first_replacement_key = repository.get(item.id).await.unwrap().unwrap().images[0]
        .object_key
        .clone();
    assert!(media.read(&first_replacement_key).await.is_ok());

    media.fail_deletes(true);
    let second_replacement = app
        .clone()
        .oneshot(replace_request(item.id, image_id))
        .await
        .unwrap();
    assert_eq!(second_replacement.status(), StatusCode::OK);
    let second_replacement_key = repository.get(item.id).await.unwrap().unwrap().images[0]
        .object_key
        .clone();
    assert_ne!(first_replacement_key, second_replacement_key);

    media.fail_deletes(false);
    let retried = app
        .oneshot(
            Request::post(format!(
                "/admin/api/items/{}/images/{image_id}/cleanup/retry",
                item.id
            ))
            .header(header::AUTHORIZATION, "Bearer operator-test-token")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retried.status(), StatusCode::OK);
    assert!(media.read(&first_replacement_key).await.is_err());
    assert!(media.read(&second_replacement_key).await.is_ok());
    let item_after_retry = repository.get(item.id).await.unwrap().unwrap();
    assert_eq!(item_after_retry.images.len(), 1);
    assert_eq!(item_after_retry.images[0].id, image_id);
    assert_eq!(
        item_after_retry.images[0].object_key,
        second_replacement_key
    );
}

#[tokio::test]
async fn replacement_cleanup_warning_persistence_failure_returns_error() {
    let root = tempdir().unwrap();
    let repository = Arc::new(FailingCleanupEventRepository {
        inner: MemoryCatalogRepository::default(),
    });
    let media = Arc::new(FailingDeleteMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
    let item = repository.create(item_input()).await.unwrap();
    let uploaded = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    let old_image_id = Uuid::parse_str(uploaded["images"][0]["id"].as_str().unwrap()).unwrap();
    let old_key = repository.get(item.id).await.unwrap().unwrap().images[0]
        .object_key
        .clone();
    media.fail_deletes(true);

    let replaced = app
        .oneshot(replace_request(item.id, old_image_id))
        .await
        .unwrap();

    assert_eq!(replaced.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        repository
            .cleanup_warnings(item.id)
            .await
            .unwrap()
            .is_empty()
    );
    let rolled_back = repository.get(item.id).await.unwrap().unwrap();
    assert_eq!(rolled_back.images.len(), 1);
    assert_eq!(rolled_back.images[0].id, old_image_id);
    assert_eq!(rolled_back.images[0].object_key, old_key);
}

#[tokio::test]
async fn cleanup_retry_is_idempotent_when_object_is_already_gone() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media = Arc::new(FailingDeleteMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
    let item = repository.create(item_input()).await.unwrap();
    let uploaded = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    let image_id = Uuid::parse_str(uploaded["images"][0]["id"].as_str().unwrap()).unwrap();
    media.fail_deletes(true);
    let failed = app
        .clone()
        .oneshot(
            Request::delete(format!("/admin/api/items/{}/images/{image_id}", item.id))
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(failed.status(), StatusCode::CONFLICT);
    media.fail_deletes(false);
    media
        .delete(&build_original_object_key(item.id, image_id))
        .await
        .unwrap();

    let retried = app
        .oneshot(
            Request::post(format!(
                "/admin/api/items/{}/images/{image_id}/cleanup/retry",
                item.id
            ))
            .header(header::AUTHORIZATION, "Bearer operator-test-token")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retried.status(), StatusCode::OK);
    assert!(
        repository
            .get(item.id)
            .await
            .unwrap()
            .unwrap()
            .images
            .is_empty()
    );
    assert!(
        repository
            .cleanup_warnings(item.id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn cleanup_retry_without_warning_does_not_delete_healthy_image() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media = Arc::new(LocalMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
    let item = repository.create(item_input()).await.unwrap();
    let uploaded = response_json(
        app.clone()
            .oneshot(upload_request(item.id, None))
            .await
            .unwrap(),
    )
    .await;
    let image_id = Uuid::parse_str(uploaded["images"][0]["id"].as_str().unwrap()).unwrap();
    let object_key = build_original_object_key(item.id, image_id);

    let retried = app
        .oneshot(
            Request::post(format!(
                "/admin/api/items/{}/images/{image_id}/cleanup/retry",
                item.id
            ))
            .header(header::AUTHORIZATION, "Bearer operator-test-token")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retried.status(), StatusCode::CONFLICT);
    assert_eq!(media.read(&object_key).await.unwrap(), png_fixture());
    assert_eq!(
        repository.get(item.id).await.unwrap().unwrap().images.len(),
        1
    );
    assert!(
        repository
            .cleanup_warnings(item.id)
            .await
            .unwrap()
            .is_empty()
    );
}

fn item_input() -> AutographItemInput {
    AutographItemInput {
        title: "Signed Card".into(),
        signer: "Signer".into(),
        description: None,
        category: "Cards".into(),
        tags: vec![],
        object_reference: None,
        event_name: None,
        event_location: None,
        source: None,
        inscription: None,
        certification_company: None,
        certification_id: None,
        estimated_year: None,
        publication_status: PublicationStatus::Draft,
    }
}

fn upload_request(item_id: Uuid, primary: Option<bool>) -> Request<Body> {
    let boundary = "cleanup-boundary";
    let mut body = Vec::new();
    if let Some(primary) = primary {
        body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"isPrimary\"\r\n\r\n{primary}\r\n").as_bytes());
    }
    body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"private.jpg\"\r\nContent-Type: image/png\r\n\r\n").as_bytes());
    body.extend_from_slice(&png_fixture());
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    Request::post(format!("/admin/api/items/{item_id}/images"))
        .header(header::AUTHORIZATION, "Bearer operator-test-token")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap()
}

fn replace_request(item_id: Uuid, image_id: Uuid) -> Request<Body> {
    replace_request_with_body(item_id, image_id, png_fixture())
}

fn replace_request_with_body(item_id: Uuid, image_id: Uuid, image_body: Vec<u8>) -> Request<Body> {
    let boundary = "cleanup-replace-boundary";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"replacement.png\"\r\nContent-Type: image/png\r\n\r\n").as_bytes());
    body.extend_from_slice(&image_body);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    Request::put(format!("/admin/api/items/{item_id}/images/{image_id}"))
        .header(header::AUTHORIZATION, "Bearer operator-test-token")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap()
}

fn item_with_image(item_id: Uuid, image_id: Uuid, object_key: String) -> AutographItem {
    AutographItem {
        id: item_id,
        title: "Signed Card".into(),
        signer: "Signer".into(),
        description: None,
        category: "Cards".into(),
        tags: vec![],
        object_reference: None,
        event_name: None,
        event_location: None,
        source: None,
        inscription: None,
        certification_company: None,
        certification_id: None,
        estimated_year: None,
        publication_status: PublicationStatus::Draft,
        images: vec![AutographImage {
            id: image_id,
            object_key,
            original_filename: "old.png".into(),
            content_type: "image/png".into(),
            byte_size: 3,
            is_primary: true,
            sort_order: 0,
            alt_text: None,
        }],
        created_at_epoch_seconds: 0,
        updated_at_epoch_seconds: 0,
    }
}

fn png_fixture() -> Vec<u8> {
    let mut body = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(RgbImage::from_pixel(8, 8, Rgb([1, 2, 3])))
        .write_to(&mut body, ImageFormat::Png)
        .unwrap();
    body.into_inner()
}
async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
}

fn file_count(root: &Path) -> usize {
    fn visit(path: &Path) -> usize {
        let mut count = 0;
        for entry in fs::read_dir(path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += visit(&path);
            } else {
                count += 1;
            }
        }
        count
    }
    visit(root)
}

struct FailingDeleteMediaStore {
    inner: LocalMediaStore,
    fail_deletes: AtomicBool,
}

impl FailingDeleteMediaStore {
    fn new(root: impl AsRef<Path>) -> Self {
        Self {
            inner: LocalMediaStore::new(root.as_ref()),
            fail_deletes: AtomicBool::new(false),
        }
    }

    fn fail_deletes(&self, fail: bool) {
        self.fail_deletes.store(fail, Ordering::SeqCst);
    }
}

#[async_trait]
impl PrivateMediaStore for FailingDeleteMediaStore {
    async fn write(&self, object_key: &str, body: &[u8]) -> Result<(), String> {
        self.inner.write(object_key, body).await
    }

    async fn read(&self, object_key: &str) -> Result<Vec<u8>, String> {
        self.inner.read(object_key).await
    }

    async fn delete(&self, object_key: &str) -> Result<(), String> {
        if self.fail_deletes.load(Ordering::SeqCst) {
            Err(format!("forced delete failure for {object_key}"))
        } else {
            self.inner.delete(object_key).await
        }
    }
}

struct FailingReplaceRepository {
    item: AutographItem,
}

#[async_trait]
impl CatalogRepository for FailingReplaceRepository {
    async fn create(&self, _input: AutographItemInput) -> Result<AutographItem, String> {
        Err("not used".to_owned())
    }

    async fn update(
        &self,
        _id: Uuid,
        _input: AutographItemUpdate,
    ) -> Result<AutographItem, String> {
        Err("not used".to_owned())
    }

    async fn get(&self, id: Uuid) -> Result<Option<AutographItem>, String> {
        Ok((id == self.item.id).then(|| self.item.clone()))
    }

    async fn list(&self) -> Result<Vec<AutographItem>, String> {
        Ok(vec![self.item.clone()])
    }

    async fn attach_image(
        &self,
        _item_id: Uuid,
        _image: AutographImage,
    ) -> Result<AutographItem, String> {
        Err("not used".to_owned())
    }

    async fn set_primary_image(
        &self,
        _item_id: Uuid,
        _image_id: Uuid,
    ) -> Result<AutographItem, String> {
        Err("not used".to_owned())
    }

    async fn remove_image_metadata(
        &self,
        _item_id: Uuid,
        _image_id: Uuid,
    ) -> Result<AutographItem, String> {
        Err("not used".to_owned())
    }

    async fn replace_image_metadata(
        &self,
        _item_id: Uuid,
        _image_id: Uuid,
        _input: ImageReplacementInput,
    ) -> Result<AutographItem, String> {
        Err("forced metadata replacement failure".to_owned())
    }
}

struct FailingRemoveRepository {
    item: Mutex<AutographItem>,
    cleanup_events: Mutex<Vec<ImageCleanupEvent>>,
}

#[async_trait]
impl CatalogRepository for FailingRemoveRepository {
    async fn create(&self, _input: AutographItemInput) -> Result<AutographItem, String> {
        Err("not used".to_owned())
    }

    async fn update(
        &self,
        _id: Uuid,
        _input: AutographItemUpdate,
    ) -> Result<AutographItem, String> {
        Err("not used".to_owned())
    }

    async fn get(&self, id: Uuid) -> Result<Option<AutographItem>, String> {
        let item = self.item.lock().expect("item lock").clone();
        Ok((id == item.id).then_some(item))
    }

    async fn list(&self) -> Result<Vec<AutographItem>, String> {
        Ok(vec![self.item.lock().expect("item lock").clone()])
    }

    async fn attach_image(
        &self,
        _item_id: Uuid,
        _image: AutographImage,
    ) -> Result<AutographItem, String> {
        Err("not used".to_owned())
    }

    async fn remove_image_metadata(
        &self,
        _item_id: Uuid,
        _image_id: Uuid,
    ) -> Result<AutographItem, String> {
        Err("forced metadata removal failure".to_owned())
    }

    async fn record_cleanup_event(
        &self,
        event: ImageCleanupEvent,
    ) -> Result<ImageCleanupEvent, String> {
        self.cleanup_events
            .lock()
            .expect("cleanup lock")
            .push(event.clone());
        Ok(event)
    }

    async fn cleanup_warnings(&self, item_id: Uuid) -> Result<Vec<CleanupWarning>, String> {
        Ok(self
            .cleanup_events
            .lock()
            .expect("cleanup lock")
            .iter()
            .filter(|event| event.item_id == item_id && event.status == CleanupStatus::DeleteFailed)
            .map(|event| CleanupWarning {
                image_id: event.image_id,
                target_object_key: event.target_object_key.clone(),
                operation: event.operation.clone(),
                status: event.status,
                admin_message: event.admin_message.clone(),
            })
            .collect())
    }
}

struct FailingCleanupEventRepository {
    inner: MemoryCatalogRepository,
}

#[async_trait]
impl CatalogRepository for FailingCleanupEventRepository {
    async fn create(&self, input: AutographItemInput) -> Result<AutographItem, String> {
        self.inner.create(input).await
    }

    async fn update(&self, id: Uuid, input: AutographItemUpdate) -> Result<AutographItem, String> {
        self.inner.update(id, input).await
    }

    async fn get(&self, id: Uuid) -> Result<Option<AutographItem>, String> {
        self.inner.get(id).await
    }

    async fn list(&self) -> Result<Vec<AutographItem>, String> {
        self.inner.list().await
    }

    async fn attach_image(
        &self,
        item_id: Uuid,
        image: AutographImage,
    ) -> Result<AutographItem, String> {
        self.inner.attach_image(item_id, image).await
    }

    async fn replace_image_metadata(
        &self,
        item_id: Uuid,
        image_id: Uuid,
        input: ImageReplacementInput,
    ) -> Result<AutographItem, String> {
        self.inner
            .replace_image_metadata(item_id, image_id, input)
            .await
    }

    async fn record_cleanup_event(
        &self,
        _event: ImageCleanupEvent,
    ) -> Result<ImageCleanupEvent, String> {
        Err("forced cleanup event persistence failure".to_owned())
    }

    async fn cleanup_warnings(&self, item_id: Uuid) -> Result<Vec<CleanupWarning>, String> {
        self.inner.cleanup_warnings(item_id).await
    }
}
