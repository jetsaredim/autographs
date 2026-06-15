use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

use autographs_controller::{
    catalog::{
        AutographImage, AutographItem, AutographItemInput, AutographItemUpdate, CatalogRepository,
        MemoryCatalogRepository, PublicationStatus,
    },
    config::ControllerConfig,
    contracts::{ImageVariantName, PublicCatalog, PublicItemDetail, PublishManifest},
    media::{LocalMediaStore, PrivateMediaStore},
    publisher::{
        LocalPublisher, PublishChange, PublishMode, artifact_impact_for, validate_candidate,
    },
    routes::router_with_services,
    storage_keys::build_original_object_key,
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use image::{DynamicImage, ImageFormat};
use serde_json::Value;
use tempfile::{TempDir, tempdir};
use tower::ServiceExt;
use uuid::Uuid;

struct Fixture {
    root: TempDir,
    _media_root: TempDir,
    repository: MemoryCatalogRepository,
    media: LocalMediaStore,
    published: AutographItem,
    private_image_id: Uuid,
    private_filename: String,
}

#[tokio::test]
async fn publisher_generates_candidate_release_and_derivatives() {
    let fixture = fixture().await;
    let publisher = LocalPublisher::new(fixture.root.path());

    let status = publisher
        .publish(&fixture.repository, &fixture.media, PublishMode::Full)
        .await
        .unwrap();
    let current = fixture.root.path().join("current");

    assert_eq!(status.state, "succeeded");
    assert!(status.artifact_count > 0);
    assert!(status.byte_size > 0);
    assert!(status.started_at_epoch_seconds.is_some());
    assert!(status.finished_at_epoch_seconds.is_some());
    for path in [
        "index.html",
        "favicon.ico",
        "icon.png",
        "architecture/index.html",
        "architecture/architecture-diagram.svg",
        "collection/index.html",
        "assets/browse.js",
        "assets/site.css",
        "data/collection.json",
        "data/facets.json",
        "data/items/signed-jedi-card.json",
        "items/signed-jedi-card/index.html",
        "manifest.json",
        "media/signed-jedi-card/image-1-thumbnail.webp",
        "media/signed-jedi-card/image-1-detail.webp",
    ] {
        assert!(current.join(path).is_file(), "missing {path}");
    }

    let rendered = read_tree(&current);
    assert!(!rendered.contains(&fixture.private_filename));
    assert!(!rendered.contains(&fixture.private_image_id.to_string()));
    assert!(!rendered.contains(&fixture.published.images[0].object_key));
    assert!(!rendered.contains("Draft Only"));

    let manifest: PublishManifest = read_json(&current.join("manifest.json"));
    let derivatives = manifest
        .artifacts
        .iter()
        .filter(|entry| entry.variant.is_some())
        .collect::<Vec<_>>();
    assert_eq!(derivatives.len(), 2);
    assert!(derivatives.iter().all(|entry| {
        entry.content_type.as_deref() == Some("image/webp")
            && entry.byte_size > 0
            && matches!(
                entry.variant,
                Some(ImageVariantName::Thumbnail | ImageVariantName::Detail)
            )
    }));

    let catalog: PublicCatalog = read_json(&current.join("data/collection.json"));
    let selected = catalog
        .items
        .iter()
        .filter(|item| item.category == "Cards" && item.tags.contains(&"jedi".to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    let script = fs::read_to_string(current.join("assets/browse.js")).unwrap();
    assert!(script.contains("/data/collection.json"));
    assert!(script.contains("/data/facets.json"));
    assert!(!script.contains("/api/"));

    let landing = fs::read_to_string(current.join("index.html")).unwrap();
    let collection = fs::read_to_string(current.join("collection/index.html")).unwrap();
    let detail = fs::read_to_string(current.join("items/signed-jedi-card/index.html")).unwrap();
    let architecture = fs::read_to_string(current.join("architecture/index.html")).unwrap();
    let site_css = fs::read_to_string(current.join("assets/site.css")).unwrap();
    assert!(landing.contains("landing-hero"));
    assert!(landing.contains(r#"<link rel="icon" href="/favicon.ico" sizes="any">"#));
    assert!(landing.contains(r#"<link rel="apple-touch-icon" href="/icon.png">"#));
    assert!(landing.contains("public-footer"));
    assert!(collection.contains("collection-panel"));
    assert!(collection.contains("public-footer"));
    assert!(detail.contains("image-viewer is-revealed"));
    assert!(detail.contains("public-footer"));
    assert!(architecture.contains("Autographs system overview"));
    assert!(architecture.contains(r#"<link rel="icon" href="/favicon.ico" sizes="any">"#));
    assert!(architecture.contains("public-footer"));
    assert!(architecture.contains("./architecture-diagram.svg"));
    assert!(site_css.contains(".gallery-card"));
}

#[tokio::test]
async fn publisher_validation_rejects_missing_derivatives_and_private_terms() {
    let fixture = fixture().await;
    let publisher = LocalPublisher::new(fixture.root.path());
    publisher
        .publish(&fixture.repository, &fixture.media, PublishMode::Full)
        .await
        .unwrap();
    let current = fixture.root.path().join("current");
    let derivative = current.join("media/signed-jedi-card/image-1-detail.webp");
    let generated_webp = fs::read(&derivative).unwrap();

    fs::remove_file(&derivative).unwrap();
    assert!(
        validate_candidate(&current)
            .unwrap_err()
            .contains("missing referenced derivative")
    );

    fs::write(&derivative, generated_webp).unwrap();
    fs::write(current.join("index.html"), "objectKey").unwrap();
    assert!(
        validate_candidate(&current)
            .unwrap_err()
            .contains("byte size changed")
    );

    let release = current.canonicalize().unwrap();
    fs::write(release.join("index.html"), "<p>objectKey</p>").unwrap();
    let mut value: Value = read_json(&release.join("manifest.json"));
    let index = value["artifacts"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["path"] == "index.html")
        .unwrap();
    index["byteSize"] = Value::from(fs::metadata(release.join("index.html")).unwrap().len());
    fs::write(
        release.join("manifest.json"),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
    assert!(
        validate_candidate(&release)
            .unwrap_err()
            .contains("privacy scan")
    );
}

#[tokio::test]
async fn publisher_public_browse_surfaces_do_not_execute_operator_markup() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let title = r#"<img src=x onerror=alert("title")>"#;
    let signer = r#"<script>alert("signer")</script>"#;
    let tag = r#"<svg onload=alert("tag")>"#;
    let item = repository
        .create(AutographItemInput {
            title: title.to_owned(),
            signer: signer.to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: vec![tag.to_owned()],
            object_reference: None,
            event_name: None,
            event_location: None,
            source: None,
            inscription: None,
            certification_company: None,
            certification_id: None,
            estimated_year: None,
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();

    LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap();

    let current = root.path().join("current");
    let collection_html = fs::read_to_string(current.join("collection/index.html")).unwrap();
    let detail_html = fs::read_to_string(
        current
            .join("items")
            .join(slug_for_test(&item.title))
            .join("index.html"),
    )
    .unwrap();
    let browse_js = fs::read_to_string(current.join("assets/browse.js")).unwrap();

    assert!(!collection_html.contains(title));
    assert!(!collection_html.contains(signer));
    assert!(!collection_html.contains(tag));
    assert!(!detail_html.contains(title));
    assert!(!detail_html.contains(signer));
    assert!(detail_html.contains("&lt;img src=x onerror=alert(&quot;title&quot;)&gt;"));
    assert!(detail_html.contains("&lt;script&gt;alert(&quot;signer&quot;)&lt;/script&gt;"));
    assert!(!browse_js.contains("innerHTML"));
    assert!(!browse_js.contains(title));
    assert!(!browse_js.contains(signer));
    assert!(!browse_js.contains(tag));
    assert!(browse_js.contains("textContent"));
    assert!(browse_js.contains("replaceChildren"));
}

#[tokio::test]
async fn publisher_uses_primary_image_first_for_gallery_and_derivatives() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Primary Selection Card".to_owned(),
            signer: "Example Signer".to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: vec!["primary".to_owned()],
            object_reference: None,
            event_name: None,
            event_location: None,
            source: None,
            inscription: None,
            certification_company: None,
            certification_id: None,
            estimated_year: None,
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();

    let supporting_id = Uuid::new_v4();
    let supporting_key = build_original_object_key(item.id, supporting_id);
    let supporting_bytes = png_bytes();
    media
        .write(&supporting_key, &supporting_bytes)
        .await
        .unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: supporting_id,
                object_key: supporting_key,
                original_filename: "supporting.png".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: supporting_bytes.len(),
                is_primary: false,
                sort_order: 0,
                alt_text: Some("Supporting image".to_owned()),
            },
        )
        .await
        .unwrap();

    let primary_id = Uuid::new_v4();
    let primary_key = build_original_object_key(item.id, primary_id);
    let primary_bytes = png_bytes();
    media.write(&primary_key, &primary_bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: primary_id,
                object_key: primary_key,
                original_filename: "primary.png".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: primary_bytes.len(),
                is_primary: true,
                sort_order: 1,
                alt_text: Some("Primary image".to_owned()),
            },
        )
        .await
        .unwrap();

    LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap();

    let current = root.path().join("current");
    let catalog: PublicCatalog = read_json(&current.join("data/collection.json"));
    let detail: PublicItemDetail =
        read_json(&current.join("data/items/primary-selection-card.json"));

    assert_eq!(
        catalog.items[0].primary_image.as_ref().unwrap().alt_text,
        "Primary image"
    );
    assert_eq!(detail.images[0].alt_text, "Primary image");
    assert_eq!(detail.images[1].alt_text, "Supporting image");
    let detail_html =
        fs::read_to_string(current.join("items/primary-selection-card/index.html")).unwrap();
    assert!(detail_html.contains("Primary image"));
    assert!(detail_html.contains("Supporting image"));
    assert!(detail_html.contains("image-1-detail.webp"));
    assert!(detail_html.contains("image-2-detail.webp"));
    assert!(detail_html.contains("image-thumbnails"));
    assert!(
        current
            .join("media/primary-selection-card/image-1-thumbnail.webp")
            .is_file()
    );
    assert!(
        current
            .join("media/primary-selection-card/image-2-thumbnail.webp")
            .is_file()
    );
}

#[tokio::test]
async fn publisher_incremental_removes_unpublished_and_stale_artifacts() {
    let fixture = fixture().await;
    let publisher = LocalPublisher::new(fixture.root.path());
    publisher
        .publish(&fixture.repository, &fixture.media, PublishMode::Full)
        .await
        .unwrap();
    let current = fixture.root.path().join("current");
    fs::create_dir_all(current.join("api/catalog")).unwrap();
    fs::write(current.join("api/catalog/index.html"), b"stale api").unwrap();
    fs::write(current.join("media/stale.webp"), b"stale").unwrap();

    fixture
        .repository
        .update(
            fixture.published.id,
            AutographItemUpdate {
                publication_status: Some(PublicationStatus::Draft),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    publisher
        .publish(
            &fixture.repository,
            &fixture.media,
            PublishMode::Incremental,
        )
        .await
        .unwrap();

    let current = fixture.root.path().join("current");
    assert!(!current.join("data/items/signed-jedi-card.json").exists());
    assert!(!current.join("items/signed-jedi-card/index.html").exists());
    assert!(!current.join("media/signed-jedi-card").exists());
    assert!(!current.join("media/stale.webp").exists());
    assert!(!current.join("api/catalog/index.html").exists());
    let manifest: PublishManifest = read_json(&current.join("manifest.json"));
    assert!(
        !manifest
            .artifacts
            .iter()
            .any(|entry| entry.path.contains("signed-jedi-card")
                || entry.path == "media/stale.webp"
                || entry.path.starts_with("api/"))
    );

    assert!(artifact_impact_for(PublishChange::PublicationStatus).derivatives);
    assert!(artifact_impact_for(PublishChange::TagsAndFacets).facets);
}

#[tokio::test]
async fn publisher_routes_require_auth_and_report_redacted_status() {
    let fixture = fixture().await;
    let publisher = Arc::new(LocalPublisher::new(fixture.root.path()));
    let app = router_with_services(
        ControllerConfig::for_test(false),
        Arc::new(fixture.repository),
        Arc::new(fixture.media),
        publisher,
    );

    let unauthenticated = app
        .clone()
        .oneshot(
            Request::post("/admin/api/publish/full")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let published = app
        .clone()
        .oneshot(
            Request::post("/admin/api/publish/full")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::CREATED);

    let status = app
        .oneshot(
            Request::get("/admin/api/publish/status")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(status.status(), StatusCode::OK);
    let rendered = String::from_utf8(
        to_bytes(status.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(rendered.contains(r#""state":"succeeded""#));
    for denied in ["objectKey", "bucketName", "objectstorage", "OCI_"] {
        assert!(!rendered.contains(denied));
    }
}

#[tokio::test]
async fn publisher_failed_publish_retains_only_latest_candidate() {
    let fixture = fixture().await;
    fixture
        .media
        .write(&fixture.published.images[0].object_key, b"not an image")
        .await
        .unwrap();
    let publisher = LocalPublisher::new(fixture.root.path());

    for _ in 0..2 {
        assert!(
            publisher
                .publish(&fixture.repository, &fixture.media, PublishMode::Full)
                .await
                .is_err()
        );
    }

    assert_eq!(
        fs::read_dir(fixture.root.path().join("failed"))
            .unwrap()
            .count(),
        1
    );
    let status = publisher.status();
    assert_eq!(status.state, "failed");
    assert!(status.error.is_some());
    assert!(!status.error.unwrap().contains("objectKey"));
}

async fn fixture() -> Fixture {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let published = repository
        .create(AutographItemInput {
            title: "Signed Jedi Card".to_owned(),
            signer: "Mark Hamill".to_owned(),
            description: Some("A public description.".to_owned()),
            category: "Cards".to_owned(),
            tags: vec!["jedi".to_owned(), "star-wars".to_owned()],
            object_reference: None,
            event_name: None,
            event_location: None,
            source: None,
            inscription: None,
            certification_company: None,
            certification_id: None,
            estimated_year: None,
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    repository
        .create(AutographItemInput {
            title: "Draft Only".to_owned(),
            signer: "Private Signer".to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: vec!["private".to_owned()],
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
    let private_image_id = Uuid::new_v4();
    let private_filename = "original private scan.png".to_owned();
    let object_key = build_original_object_key(published.id, private_image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    let published = repository
        .attach_image(
            published.id,
            AutographImage {
                id: private_image_id,
                object_key,
                original_filename: private_filename.clone(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();
    Fixture {
        root,
        _media_root: media_root,
        repository,
        media,
        published,
        private_image_id,
        private_filename,
    }
}

fn png_bytes() -> Vec<u8> {
    let image = DynamicImage::new_rgb8(32, 24);
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

fn read_tree(root: &Path) -> String {
    let mut rendered = String::new();
    let mut paths = vec![PathBuf::from(root)];
    while let Some(path) = paths.pop() {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                paths.push(path);
            } else {
                rendered.push_str(&String::from_utf8_lossy(&fs::read(path).unwrap()));
            }
        }
    }
    rendered
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn slug_for_test(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
