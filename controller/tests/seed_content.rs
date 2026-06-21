use std::{fs, io::Cursor, path::Path, sync::Arc};

use async_trait::async_trait;
use autographs_controller::{
    catalog::{
        AutographImage, AutographItem, AutographItemInput, AutographItemUpdate, CatalogRepository,
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
                alt_text: None,
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
                    r#"{"title":"Signed Jedi Card","signer":"Mark Hamill","category":"Cards","tags":["jedi"],"eventName":"Example Convention","source":"Private collection","estimatedYear":2024}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let body = response_json(create).await;
    let item_id = Uuid::parse_str(body["id"].as_str().unwrap()).unwrap();
    assert_eq!(body["eventName"], "Example Convention");
    assert_eq!(body["estimatedYear"], 2024);

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
    assert_eq!(
        stored.images[0].alt_text.as_deref(),
        Some("Signed card front")
    );
    assert!(
        !stored.images[0]
            .object_key
            .contains("secret bucket photo.jpg")
    );
    assert_eq!(
        media.read(&stored.images[0].object_key).await.unwrap(),
        png_fixture()
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

#[tokio::test]
async fn seed_content_upload_does_not_leave_orphan_media_when_attachment_fails() {
    let root = tempdir().unwrap();
    let item = AutographItem {
        id: Uuid::new_v4(),
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
        images: Vec::new(),
        created_at_epoch_seconds: 0,
        updated_at_epoch_seconds: 0,
    };
    let repository = Arc::new(FailingAttachRepository { item: item.clone() });
    let media = Arc::new(LocalMediaStore::new(root.path()));
    let app = router_with_stores(ControllerConfig::for_test(true), repository, media);

    let uploaded = app
        .oneshot(upload_request(item.id, Some("Bearer operator-test-token")))
        .await
        .unwrap();

    assert_eq!(uploaded.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(file_count(root.path()), 0);
}

#[tokio::test]
async fn seed_content_upload_rejects_spoofed_image_bytes_before_media_write() {
    let root = tempdir().unwrap();
    let repository = Arc::new(MemoryCatalogRepository::default());
    let media = Arc::new(LocalMediaStore::new(root.path()));
    let app = router_with_stores(
        ControllerConfig::for_test(true),
        repository.clone(),
        media.clone(),
    );
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

    let uploaded = app
        .oneshot(upload_request_with_body(
            item.id,
            Some("Bearer operator-test-token"),
            "image/jpeg",
            b"not-an-image",
        ))
        .await
        .unwrap();

    assert_eq!(uploaded.status(), StatusCode::BAD_REQUEST);
    assert_eq!(file_count(root.path()), 0);
    assert!(
        repository
            .get(item.id)
            .await
            .unwrap()
            .unwrap()
            .images
            .is_empty()
    );
}

fn upload_request(item_id: Uuid, authorization: Option<&str>) -> Request<Body> {
    let fixture = png_fixture();
    upload_request_with_body(item_id, authorization, "image/png", &fixture)
}

fn upload_request_with_body(
    item_id: Uuid,
    authorization: Option<&str>,
    content_type: &str,
    image_body: &[u8],
) -> Request<Body> {
    let boundary = "autographs-test-boundary";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"altText\"\r\n\r\nSigned card front\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"secret bucket photo.jpg\"\r\nContent-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(image_body);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
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

fn png_fixture() -> Vec<u8> {
    let mut body = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(RgbImage::from_pixel(16, 16, Rgb([21, 92, 126])))
        .write_to(&mut body, ImageFormat::Png)
        .expect("encode test PNG");
    body.into_inner()
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
}

fn file_count(root: &Path) -> usize {
    let mut count = 0;
    let mut paths = vec![root.to_path_buf()];
    while let Some(path) = paths.pop() {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                paths.push(path);
            } else {
                count += 1;
            }
        }
    }
    count
}

struct FailingAttachRepository {
    item: AutographItem,
}

#[async_trait]
impl CatalogRepository for FailingAttachRepository {
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
        Err("forced attachment failure".to_owned())
    }
}
