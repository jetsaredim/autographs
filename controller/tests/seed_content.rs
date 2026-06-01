use std::sync::Arc;

use autographs_controller::{
    catalog::{
        AutographImage, AutographItemInput, CatalogRepository, MemoryCatalogRepository,
        PublicationStatus,
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
use serde_json::Value;
use tempfile::tempdir;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn seed_content_local_repository_and_media_use_filename_free_keys() {
    let root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Signed Jedi Card".to_owned(),
            signer: "Mark Hamill".to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: vec!["jedi".to_owned()],
            publication_status: PublicationStatus::Draft,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let original_filename = "secret bucket photo.jpg";
    let key = build_original_object_key(item.id, image_id);

    assert!(!key.contains(original_filename));
    assert!(!key.contains(".jpg"));
    assert!(!key.contains(".png"));
    assert!(!key.contains(' '));

    media.write(&key, b"private-original-bytes").await.unwrap();
    assert_eq!(media.read(&key).await.unwrap(), b"private-original-bytes");

    let updated = repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key: key,
                original_filename: original_filename.to_owned(),
                content_type: "image/jpeg".to_owned(),
                byte_size: 22,
                is_primary: true,
                sort_order: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.images.len(), 1);
    assert!(updated.images[0].is_primary);
}

#[tokio::test]
async fn seed_content_private_api_persists_redacted_item_and_image_response() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media = Arc::new(LocalMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
    let token = "Bearer operator-test-token";

    let create = app
        .clone()
        .oneshot(
            Request::post("/admin/api/items")
                .header(header::AUTHORIZATION, token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"title":"Signed Jedi Card","signer":"Mark Hamill","category":"Cards","tags":["jedi"]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let body = response_json(create).await;
    let item_id = Uuid::parse_str(body["id"].as_str().unwrap()).unwrap();

    let unauthenticated_upload = app
        .clone()
        .oneshot(upload_request(item_id, None))
        .await
        .unwrap();
    assert_eq!(unauthenticated_upload.status(), StatusCode::UNAUTHORIZED);

    let uploaded = app
        .clone()
        .oneshot(upload_request(item_id, Some(token)))
        .await
        .unwrap();
    assert_eq!(uploaded.status(), StatusCode::CREATED);
    let rendered = String::from_utf8(
        to_bytes(uploaded.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    for denied in [
        "storageNamespace",
        "bucketName",
        "objectKey",
        "objectstorage",
        "secret bucket photo.jpg",
    ] {
        assert!(!rendered.contains(denied), "response leaked {denied}");
    }

    let stored = repository.get(item_id).await.unwrap().unwrap();
    assert_eq!(stored.images.len(), 1);
    assert!(
        !stored.images[0]
            .object_key
            .contains("secret bucket photo.jpg")
    );
    assert_eq!(
        media.read(&stored.images[0].object_key).await.unwrap(),
        b"private-original-bytes"
    );

    let published = app
        .clone()
        .oneshot(
            Request::post(format!("/admin/api/items/{item_id}/publication"))
                .header(header::AUTHORIZATION, token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"publicationStatus":"published"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::OK);
    assert_eq!(
        response_json(published).await["publicationStatus"],
        "published"
    );
}

fn upload_request(item_id: Uuid, authorization: Option<&str>) -> Request<Body> {
    let boundary = "autographs-test-boundary";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"secret bucket photo.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nprivate-original-bytes\r\n--{boundary}--\r\n"
    );
    let mut request = Request::post(format!("/admin/api/items/{item_id}/images"))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();
    if let Some(authorization) = authorization {
        request.headers_mut().insert(
            header::AUTHORIZATION,
            authorization.parse().expect("authorization header"),
        );
    }
    request
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
}
