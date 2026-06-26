use std::{io::Cursor, sync::Arc};

use autographs_controller::{
    catalog::{AutographItemInput, CatalogRepository, MemoryCatalogRepository, PublicationStatus},
    config::ControllerConfig,
    media::LocalMediaStore,
    routes::router_with_stores,
};
use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
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

    let first = response_json(app.clone().oneshot(upload_request(item.id, None)).await.unwrap()).await;
    assert_eq!(first["images"][0]["isPrimary"], true);

    let second = response_json(app.clone().oneshot(upload_request(item.id, None)).await.unwrap()).await;
    let second_id = second["images"]
        .as_array().unwrap().iter().find(|image| image["isPrimary"] == false)
        .and_then(|image| image["id"].as_str()).unwrap().to_owned();
    assert_eq!(second["images"].as_array().unwrap().iter().filter(|image| image["isPrimary"] == true).count(), 1);

    let primary = app.oneshot(
        Request::post(format!("/admin/api/items/{}/images/{second_id}/primary", item.id))
            .header(header::AUTHORIZATION, "Bearer operator-test-token")
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    assert_eq!(primary.status(), StatusCode::OK);
    let selected = response_json(primary).await;
    assert_eq!(selected["images"].as_array().unwrap().iter().filter(|image| image["isPrimary"] == true).count(), 1);
    assert!(selected["images"].as_array().unwrap().iter().any(|image| image["id"] == second_id && image["isPrimary"] == true));
}

fn item_input() -> AutographItemInput {
    AutographItemInput { title: "Signed Card".into(), signer: "Signer".into(), description: None, category: "Cards".into(), tags: vec![], object_reference: None, event_name: None, event_location: None, source: None, inscription: None, certification_company: None, certification_id: None, estimated_year: None, publication_status: PublicationStatus::Draft }
}

fn upload_request(item_id: Uuid, primary: Option<bool>) -> Request<Body> {
    let boundary = "cleanup-boundary";
    let mut body = Vec::new();
    if let Some(primary) = primary { body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"isPrimary\"\r\n\r\n{primary}\r\n").as_bytes()); }
    body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"private.jpg\"\r\nContent-Type: image/png\r\n\r\n").as_bytes());
    body.extend_from_slice(&png_fixture());
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    Request::post(format!("/admin/api/items/{item_id}/images"))
        .header(header::AUTHORIZATION, "Bearer operator-test-token")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(body)).unwrap()
}

fn png_fixture() -> Vec<u8> { let mut body = Cursor::new(Vec::new()); DynamicImage::ImageRgb8(RgbImage::from_pixel(8, 8, Rgb([1, 2, 3]))).write_to(&mut body, ImageFormat::Png).unwrap(); body.into_inner() }
async fn response_json(response: axum::response::Response) -> Value { serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap() }
