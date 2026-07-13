use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use async_trait::async_trait;
use autographs_controller::{
    catalog::{
        AutographImage, AutographItem, AutographItemInput, AutographItemUpdate, CatalogRepository,
        FieldPatch, ImageReplacementInput, ItemOrigin, MemoryCatalogRepository, PublicationStatus,
        SignerCreditInput,
    },
    config::ControllerConfig,
    contracts::{ImageVariantName, PublicCatalog, PublicItemDetail, PublishManifest},
    derivatives::{DerivativeVariant, generate_derivative},
    media::{LocalMediaStore, PrivateMediaStore},
    publisher::{
        LocalPublisher, PublishChange, PublishMode, ReleaseRetentionPolicy, artifact_impact_for,
        validate_candidate,
    },
    routes::router_with_services,
    storage_keys::build_original_object_key,
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
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

#[derive(Clone)]
struct CountingMediaStore {
    inner: LocalMediaStore,
    reads: Arc<AtomicUsize>,
}

impl CountingMediaStore {
    fn new(inner: LocalMediaStore) -> Self {
        Self {
            inner,
            reads: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn read_count(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl PrivateMediaStore for CountingMediaStore {
    async fn write(&self, object_key: &str, body: &[u8]) -> Result<(), String> {
        self.inner.write(object_key, body).await
    }

    async fn read(&self, object_key: &str) -> Result<Vec<u8>, String> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.read(object_key).await
    }

    async fn delete(&self, object_key: &str) -> Result<(), String> {
        self.inner.delete(object_key).await
    }
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
    assert_eq!(status.stage.as_deref(), Some("succeeded"));
    assert!(status.artifact_count > 0);
    assert!(status.byte_size > 0);
    assert_eq!(status.item_count, 1);
    assert_eq!(status.image_count, 1);
    assert_eq!(status.derivative_count, 2);
    assert!(status.started_at_epoch_seconds.is_some());
    assert!(status.finished_at_epoch_seconds.is_some());
    for path in [
        "index.html",
        "404.html",
        "favicon.ico",
        "icon.png",
        "architecture/index.html",
        "architecture/architecture-diagram.svg",
        "admin/index.html",
        "admin/admin.js",
        "admin/admin.css",
        "collection/index.html",
        "assets/browse.js",
        "assets/detail.js",
        "assets/footer.js",
        "assets/landing.js",
        "assets/not-found.js",
        "assets/site.css",
        "data/collection.json",
        "data/facets.json",
        "data/not-found-quotes.json",
        "data/items/signed-jedi-card.json",
        "items/signed-jedi-card/index.html",
        "manifest.json",
    ] {
        assert!(current.join(path).is_file(), "missing {path}");
    }

    let rendered = read_tree(&current);
    assert!(!rendered.contains(&fixture.private_filename));
    assert!(!rendered.contains(&fixture.private_image_id.to_string()));
    assert!(!rendered.contains(&fixture.published.images[0].object_key));
    assert!(!rendered.contains("Draft Only"));

    let manifest: PublishManifest = read_json(&current.join("manifest.json"));
    for path in ["admin/index.html", "admin/admin.js", "admin/admin.css"] {
        assert!(
            manifest.artifacts.iter().any(|entry| entry.path == path),
            "manifest missing {path}"
        );
    }
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
    for entry in derivatives {
        let derivative = image::open(current.join(&entry.path)).unwrap();
        match entry.variant {
            Some(ImageVariantName::Thumbnail) => {
                assert!(derivative.width() <= 480);
                assert!(derivative.height() <= 640);
            }
            Some(ImageVariantName::Detail) => {
                assert!(derivative.width() <= 960);
                assert!(derivative.height() <= 1280);
            }
            None => unreachable!("derivative entries always have a variant"),
        }
    }

    let catalog: PublicCatalog = read_json(&current.join("data/collection.json"));
    let detail_json: PublicItemDetail =
        read_json(&current.join("data/items/signed-jedi-card.json"));
    assert_versioned_media_file(
        &current,
        &catalog.items[0]
            .primary_image
            .as_ref()
            .unwrap()
            .variants
            .iter()
            .find(|variant| variant.name == ImageVariantName::Thumbnail)
            .unwrap()
            .path,
        "signed-jedi-card",
        "image-1",
        ImageVariantName::Thumbnail,
    );
    assert_versioned_media_file(
        &current,
        &detail_json.images[0]
            .variants
            .iter()
            .find(|variant| variant.name == ImageVariantName::Detail)
            .unwrap()
            .path,
        "signed-jedi-card",
        "image-1",
        ImageVariantName::Detail,
    );
    let selected = catalog
        .items
        .iter()
        .filter(|item| item.format == "Trading Card" && item.tags.contains(&"jedi".to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    let script = fs::read_to_string(current.join("assets/browse.js")).unwrap();
    assert!(script.contains("/data/collection.json"));
    assert!(script.contains("/data/facets.json"));
    assert!(script.contains("/items/${encodeURIComponent(item.slug)}/"));
    assert!(!script.contains("/api/"));
    let detail_script = fs::read_to_string(current.join("assets/detail.js")).unwrap();
    assert!(detail_script.contains(".focused-image-button"));
    assert!(detail_script.contains("aria-pressed"));
    let footer_script = fs::read_to_string(current.join("assets/footer.js")).unwrap();
    assert!(footer_script.contains(r#"unlockSequence = "gallery""#));
    assert!(footer_script.contains(r#"link.href = "/admin/""#));
    assert!(footer_script.contains("admin-unlock"));
    let landing_script = fs::read_to_string(current.join("assets/landing.js")).unwrap();
    assert!(landing_script.contains("[data-surprise-link]"));
    assert!(landing_script.contains("/data/collection.json"));

    let landing = fs::read_to_string(current.join("index.html")).unwrap();
    let collection = fs::read_to_string(current.join("collection/index.html")).unwrap();
    let detail = fs::read_to_string(current.join("items/signed-jedi-card/index.html")).unwrap();
    let architecture = fs::read_to_string(current.join("architecture/index.html")).unwrap();
    let site_css = fs::read_to_string(current.join("assets/site.css")).unwrap();
    assert!(landing.contains("landing-hero"));
    assert!(landing.contains("data-surprise-link"));
    assert!(landing.contains(r#"<script src="/assets/footer.js"></script>"#));
    assert!(landing.contains(r#"<script src="/assets/landing.js"></script>"#));
    assert!(landing.contains(r#"<link rel="icon" href="/favicon.ico" sizes="any">"#));
    assert!(landing.contains(r#"<link rel="apple-touch-icon" href="/icon.png">"#));
    assert!(landing.contains("public-footer"));
    assert!(collection.contains("collection-panel"));
    assert!(collection.contains(r#"<script src="/assets/footer.js"></script>"#));
    assert!(collection.contains("public-footer"));
    assert!(detail.contains(r#"<script src="/assets/footer.js"></script>"#));
    assert!(detail.contains(r#"<section class="image-viewer">"#));
    assert!(detail.contains(r#"class="focused-image-button" type="button" aria-expanded="false""#));
    assert!(detail.contains(r#"class="detail-metadata-panel" aria-hidden="true""#));
    assert!(detail.contains("<h2>Story</h2>"));
    assert!(detail.contains("A public description."));
    assert!(detail.contains("For the rebellion"));
    assert!(detail.contains("<h2>Provenance</h2>"));
    assert!(detail.contains("Example Convention"));
    assert!(detail.contains("Vendor table"));
    assert!(detail.contains("<h2>Certification</h2>"));
    assert!(detail.contains("PSA"));
    assert!(detail.contains("Card #1138"));
    assert!(detail.contains(r#"<script src="/assets/detail.js"></script>"#));
    assert!(detail.contains("public-footer"));
    assert!(architecture.contains("Autographs system overview"));
    assert!(architecture.contains(r#"<script src="/assets/footer.js"></script>"#));
    assert!(architecture.contains(r#"<link rel="icon" href="/favicon.ico" sizes="any">"#));
    assert!(architecture.contains("public-footer"));
    assert!(architecture.contains("./architecture-diagram.svg"));
    assert!(site_css.contains(".gallery-card"));
    for (label, html) in [
        ("landing", landing),
        ("collection", collection),
        ("detail", detail),
        ("architecture", architecture),
    ] {
        assert!(
            !html.contains("{{"),
            "{label} contains unresolved template token"
        );
    }
}

#[tokio::test]
async fn publisher_generates_phase7_signer_taxonomy_facets_and_detail_links() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());

    repository
        .create(AutographItemInput {
            title: "Two Signer Card".to_owned(),
            signer: "Legacy Two".to_owned(),
            description: Some("A two-signer item.".to_owned()),
            category: "Cards".to_owned(),
            tags: vec!["animated".to_owned()],
            signer_credits: vec![
                SignerCreditInput {
                    display_name: Some("Ashley Eckstein".to_owned()),
                    default_role: Some("Actor".to_owned()),
                    item_role: Some("Voice actor".to_owned()),
                    item_context: Some("Ahsoka Tano".to_owned()),
                    wikipedia_url: Some("https://en.wikipedia.org/wiki/Ashley_Eckstein".to_owned()),
                    imdb_url: None,
                    ..Default::default()
                },
                SignerCreditInput {
                    display_name: Some("Dave Filoni".to_owned()),
                    default_role: Some("Producer".to_owned()),
                    item_role: Some("Creator".to_owned()),
                    item_context: Some("The Clone Wars".to_owned()),
                    wikipedia_url: None,
                    imdb_url: Some("https://www.imdb.com/name/nm1396048/".to_owned()),
                    ..Default::default()
                },
            ],
            characters: vec!["Ahsoka Tano".to_owned()],
            format: "Trading Card".to_owned(),
            origin: ItemOrigin::Official,
            franchises: vec!["Star Wars".to_owned()],
            product_line: Some("Clone Wars".to_owned()),
            set_name: Some("Season One".to_owned()),
            language: "English".to_owned(),
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
            title: "Three Signer Custom".to_owned(),
            signer: "Legacy Three".to_owned(),
            description: Some("A three-signer custom item.".to_owned()),
            category: "Comics".to_owned(),
            tags: vec!["convention".to_owned()],
            signer_credits: vec![
                SignerCreditInput {
                    display_name: Some("Alpha Artist".to_owned()),
                    item_role: Some("Artist".to_owned()),
                    ..Default::default()
                },
                SignerCreditInput {
                    display_name: Some("Beta Writer".to_owned()),
                    item_role: Some("Writer".to_owned()),
                    ..Default::default()
                },
                SignerCreditInput {
                    display_name: Some("Gamma Editor".to_owned()),
                    item_role: Some("Editor".to_owned()),
                    ..Default::default()
                },
            ],
            characters: vec!["Original Hero".to_owned()],
            format: "Comic Book".to_owned(),
            origin: ItemOrigin::Custom,
            franchises: vec!["Originals".to_owned()],
            product_line: None,
            set_name: Some("Custom".to_owned()),
            language: "Japanese".to_owned(),
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

    let catalog: PublicCatalog = read_json(&current.join("data/collection.json"));
    let two = catalog
        .items
        .iter()
        .find(|item| item.slug == "two-signer-card")
        .unwrap();
    let three = catalog
        .items
        .iter()
        .find(|item| item.slug == "three-signer-custom")
        .unwrap();
    assert_eq!(two.signer_text, "Ashley Eckstein + Dave Filoni");
    assert_eq!(three.signer_text, "Alpha Artist, Beta Writer + 1 more");

    let facets: Value = read_json(&current.join("data/facets.json"));
    let facet_ids = facets["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|group| group["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    for expected in [
        "signer",
        "franchise",
        "productLine",
        "format",
        "language",
        "origin",
        "role",
        "tag",
    ] {
        assert!(facet_ids.contains(&expected), "missing facet {expected}");
    }
    assert!(!facet_ids.contains(&"category"));

    let collection_json = fs::read_to_string(current.join("data/collection.json")).unwrap();
    assert!(!collection_json.contains("wikipedia.org"));
    assert!(!collection_json.contains("imdb.com"));

    let two_html = fs::read_to_string(current.join("items/two-signer-card/index.html")).unwrap();
    assert!(two_html.contains(r#"class="profile-link profile-link-wikipedia""#));
    assert!(two_html.contains(r#"class="profile-link profile-link-imdb""#));
    assert!(two_html.contains(r#"aria-label="Open Wikipedia profile for Ashley Eckstein""#));
    assert!(two_html.contains(r#"aria-label="Open IMDb profile for Dave Filoni""#));
    assert!(two_html.contains(r#"rel="noopener noreferrer""#));
    assert!(!two_html.contains("https://en.wikipedia.org/wiki/Ashley_Eckstein</"));
    assert!(!two_html.contains("https://www.imdb.com/name/nm1396048/</"));
    assert!(!two_html.contains("Language</dt><dd>English"));
    assert!(!two_html.contains("Origin</dt><dd>Official"));

    let three_html =
        fs::read_to_string(current.join("items/three-signer-custom/index.html")).unwrap();
    assert!(three_html.contains("Language</dt><dd>Japanese"));
    assert!(three_html.contains("Origin</dt><dd>Custom"));
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
    let detail: PublicItemDetail = read_json(&current.join("data/items/signed-jedi-card.json"));
    let derivative_path = detail.images[0]
        .variants
        .iter()
        .find(|variant| variant.name == ImageVariantName::Detail)
        .unwrap()
        .path
        .trim_start_matches('/')
        .to_owned();
    let derivative = current.join(&derivative_path);
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
async fn publisher_validation_rejects_derivative_fingerprint_mismatch() {
    let fixture = fixture().await;
    let publisher = LocalPublisher::new(fixture.root.path());
    publisher
        .publish(&fixture.repository, &fixture.media, PublishMode::Full)
        .await
        .unwrap();
    let current = fixture.root.path().join("current");
    let detail_path = current.join("data/items/signed-jedi-card.json");
    let mut detail: PublicItemDetail = read_json(&detail_path);
    let variant = detail.images[0]
        .variants
        .iter_mut()
        .find(|variant| variant.name == ImageVariantName::Detail)
        .unwrap();
    let original_path = variant.path.trim_start_matches('/').to_owned();
    let bad_path = tamper_media_fingerprint(&original_path);

    fs::rename(current.join(&original_path), current.join(&bad_path)).unwrap();
    variant.path = format!("/{bad_path}");
    fs::write(&detail_path, serde_json::to_vec_pretty(&detail).unwrap()).unwrap();

    let manifest_path = current.join("manifest.json");
    let mut manifest: Value = read_json(&manifest_path);
    let artifact = manifest["artifacts"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["path"] == original_path)
        .unwrap();
    artifact["path"] = Value::from(bad_path);
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();

    assert!(
        validate_candidate(&current)
            .unwrap_err()
            .contains("fingerprint mismatch")
    );
}

#[tokio::test]
async fn publisher_validates_detail_derivatives_when_item_slug_contains_thumbnail() {
    let fixture = fixture().await;
    let item = fixture
        .repository
        .create(AutographItemInput {
            title: "Star Thumbnail Card".to_owned(),
            signer: "Carrie Fisher".to_owned(),
            description: Some("A title with thumbnail in the generated slug.".to_owned()),
            category: "Cards".to_owned(),
            tags: vec!["thumbnail-edge".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes_with_color([12, 120, 200]);
    fixture.media.write(&object_key, &bytes).await.unwrap();
    fixture
        .repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "thumbnail-title.png".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let publisher = LocalPublisher::new(fixture.root.path());
    publisher
        .publish(&fixture.repository, &fixture.media, PublishMode::Full)
        .await
        .unwrap();

    let current = fixture.root.path().join("current");
    let detail: PublicItemDetail = read_json(&current.join("data/items/star-thumbnail-card.json"));
    let detail_path = detail.images[0]
        .variants
        .iter()
        .find(|variant| variant.name == ImageVariantName::Detail)
        .unwrap()
        .path
        .trim_start_matches('/')
        .to_owned();
    assert!(
        detail_path.starts_with("media/star-thumbnail-card/image-1-detail-"),
        "unexpected detail derivative path: {detail_path}"
    );
    let manifest: PublishManifest = read_json(&current.join("manifest.json"));
    let artifact = manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.path == detail_path)
        .unwrap();
    assert_eq!(artifact.variant, Some(ImageVariantName::Detail));
    validate_candidate(&current).unwrap();
}

#[tokio::test]
async fn publisher_changes_public_media_paths_when_image_content_changes() {
    let fixture = fixture().await;
    let publisher = LocalPublisher::new(fixture.root.path());
    publisher
        .publish(&fixture.repository, &fixture.media, PublishMode::Full)
        .await
        .unwrap();
    let current = fixture.root.path().join("current");
    let first_detail: PublicItemDetail =
        read_json(&current.join("data/items/signed-jedi-card.json"));
    let first_detail_path = first_detail.images[0]
        .variants
        .iter()
        .find(|variant| variant.name == ImageVariantName::Detail)
        .unwrap()
        .path
        .clone();

    let replacement_id = Uuid::new_v4();
    let replacement_key = build_original_object_key(fixture.published.id, replacement_id);
    let replacement_bytes = png_bytes_with_color([220, 24, 24]);
    fixture
        .media
        .write(&replacement_key, &replacement_bytes)
        .await
        .unwrap();
    fixture
        .repository
        .replace_image_metadata(
            fixture.published.id,
            fixture.private_image_id,
            ImageReplacementInput {
                image: AutographImage {
                    id: replacement_id,
                    object_key: replacement_key,
                    original_filename: "replacement.png".to_owned(),
                    content_type: "image/png".to_owned(),
                    byte_size: replacement_bytes.len(),
                    is_primary: false,
                    sort_order: 99,
                    alt_text: None,
                },
            },
        )
        .await
        .unwrap();

    publisher
        .publish(&fixture.repository, &fixture.media, PublishMode::Full)
        .await
        .unwrap();
    let second_detail: PublicItemDetail =
        read_json(&current.join("data/items/signed-jedi-card.json"));
    let second_detail_path = second_detail.images[0]
        .variants
        .iter()
        .find(|variant| variant.name == ImageVariantName::Detail)
        .unwrap()
        .path
        .clone();

    assert!(first_detail_path.starts_with("/media/signed-jedi-card/image-1-detail-"));
    assert!(second_detail_path.starts_with("/media/signed-jedi-card/image-1-detail-"));
    assert_ne!(first_detail_path, second_detail_path);
}

#[tokio::test]
async fn publisher_regenerates_derivatives_when_same_object_key_and_size_changes_bytes() {
    let fixture = fixture().await;
    let publisher = LocalPublisher::new(fixture.root.path());
    publisher
        .publish(&fixture.repository, &fixture.media, PublishMode::Full)
        .await
        .unwrap();
    let current = fixture.root.path().join("current");
    let first_detail: PublicItemDetail =
        read_json(&current.join("data/items/signed-jedi-card.json"));
    let first_detail_path = first_detail.images[0]
        .variants
        .iter()
        .find(|variant| variant.name == ImageVariantName::Detail)
        .unwrap()
        .path
        .clone();
    let original_image = fixture
        .repository
        .get(fixture.published.id)
        .await
        .unwrap()
        .unwrap()
        .images
        .into_iter()
        .find(|image| image.id == fixture.private_image_id)
        .unwrap();
    let original_bytes = fixture
        .media
        .read(&original_image.object_key)
        .await
        .unwrap();
    assert_eq!(original_image.byte_size, original_bytes.len());

    let replacement_bytes = same_size_replacement_png_bytes(&original_bytes);
    assert_eq!(replacement_bytes.len(), original_image.byte_size);
    fixture
        .media
        .write(&original_image.object_key, &replacement_bytes)
        .await
        .unwrap();

    publisher
        .publish(&fixture.repository, &fixture.media, PublishMode::Full)
        .await
        .unwrap();
    let second_detail: PublicItemDetail =
        read_json(&current.join("data/items/signed-jedi-card.json"));
    let second_detail_path = second_detail.images[0]
        .variants
        .iter()
        .find(|variant| variant.name == ImageVariantName::Detail)
        .unwrap()
        .path
        .clone();

    assert_eq!(
        original_image.object_key,
        fixture.published.images[0].object_key
    );
    assert_eq!(
        original_image.byte_size,
        fixture.published.images[0].byte_size
    );
    assert_ne!(original_bytes, replacement_bytes);
    assert_ne!(first_detail_path, second_detail_path);
}

#[test]
fn publisher_detail_derivative_cap_reduces_large_sample() {
    let source = large_png_bytes();
    let derivative = generate_derivative(&source, DerivativeVariant::Detail).unwrap();
    if std::env::var_os("AUTOGRAPHS_PRINT_DERIVATIVE_MEASUREMENT").is_some() {
        eprintln!(
            "detail derivative sample before={} after={} width={} height={}",
            source.len(),
            derivative.bytes.len(),
            derivative.width,
            derivative.height
        );
    }

    assert_eq!(derivative.content_type, "image/webp");
    assert_eq!(derivative.variant, DerivativeVariant::Detail);
    assert!(derivative.width <= 960);
    assert!(derivative.height <= 1280);
    assert!(
        derivative.bytes.len() < source.len(),
        "expected capped detail derivative to shrink from {} bytes, got {} bytes",
        source.len(),
        derivative.bytes.len()
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
async fn publisher_allows_generic_private_filenames_in_admin_shell_copy() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Generic Filename Card".to_owned(),
            signer: "Admin Copy".to_owned(),
            description: Some("A published item with a generic private file name.".to_owned()),
            category: "Cards".to_owned(),
            tags: vec!["generic".to_owned()],
            signer_credits: Vec::new(),
            characters: Vec::new(),
            format: "Trading Card".to_owned(),
            origin: ItemOrigin::Official,
            franchises: Vec::new(),
            product_line: None,
            set_name: None,
            language: "English".to_owned(),
            object_reference: Some("Card #1138".to_owned()),
            event_name: Some("Example Convention".to_owned()),
            event_location: Some("Orlando".to_owned()),
            source: Some("Vendor table".to_owned()),
            inscription: Some("For the rebellion".to_owned()),
            certification_company: Some("PSA".to_owned()),
            certification_id: Some("ABC123".to_owned()),
            estimated_year: Some(2026),
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "upload".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap();

    let current = root.path().join("current");
    assert!(current.join("admin/admin.js").is_file());
    assert!(
        fs::read_to_string(current.join("admin/admin.js"))
            .unwrap()
            .contains("uploadImages")
    );
    let public_catalog = read_tree(&current.join("data")) + &read_tree(&current.join("items"));
    assert!(!public_catalog.contains("upload"));
}

#[tokio::test]
async fn publisher_allows_original_filenames_that_only_match_generated_media_paths() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Generated Media Path Card".to_owned(),
            signer: "Path Copy".to_owned(),
            description: Some(
                "A published item with filenames matching derivative path terms.".to_owned(),
            ),
            category: "Cards".to_owned(),
            tags: vec!["paths".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    for (sort_order, original_filename) in ["image", "detail"].into_iter().enumerate() {
        let image_id = Uuid::new_v4();
        let object_key = build_original_object_key(item.id, image_id);
        let bytes = png_bytes();
        media.write(&object_key, &bytes).await.unwrap();
        repository
            .attach_image(
                item.id,
                AutographImage {
                    id: image_id,
                    object_key,
                    original_filename: original_filename.to_owned(),
                    content_type: "image/png".to_owned(),
                    byte_size: bytes.len(),
                    is_primary: sort_order == 0,
                    sort_order: sort_order as i32,
                    alt_text: None,
                },
            )
            .await
            .unwrap();
    }

    LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap();

    let current = root.path().join("current");
    let detail: PublicItemDetail =
        read_json(&current.join("data/items/generated-media-path-card.json"));
    assert_versioned_media_file(
        &current,
        &detail.images[0]
            .variants
            .iter()
            .find(|variant| variant.name == ImageVariantName::Detail)
            .unwrap()
            .path,
        "generated-media-path-card",
        "image-1",
        ImageVariantName::Detail,
    );
    assert_versioned_media_file(
        &current,
        &detail.images[1]
            .variants
            .iter()
            .find(|variant| variant.name == ImageVariantName::Thumbnail)
            .unwrap()
            .path,
        "generated-media-path-card",
        "image-2",
        ImageVariantName::Thumbnail,
    );
}

#[tokio::test]
async fn publisher_rejects_original_filename_embedded_in_catalog_path_token() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Embedded Filename Leak".to_owned(),
            signer: "Filename Scan".to_owned(),
            description: Some("/media/item/private-scan.jpg-detail.webp".to_owned()),
            category: "Cards".to_owned(),
            tags: vec!["privacy".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "private-scan.jpg".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_original_filename_leak_with_case_normalization() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Case Normalized Filename Leak".to_owned(),
            signer: "Filename Scan".to_owned(),
            description: Some("Rendered lower-case leak: private-scan.jpg".to_owned()),
            category: "Cards".to_owned(),
            tags: vec!["privacy".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "Private-Scan.JPG".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_original_filename_leak_with_unicode_case_normalization() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Unicode Case Normalized Filename Leak".to_owned(),
            signer: "Filename Scan".to_owned(),
            description: Some("Rendered lower-case leak: \u{00e9}vidence.jpg".to_owned()),
            category: "Cards".to_owned(),
            tags: vec!["privacy".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "\u{00c9}vidence.JPG".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_original_filename_leak_with_url_escaped_basename() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Escaped Filename Leak".to_owned(),
            signer: "Filename Scan".to_owned(),
            description: Some(
                "Rendered escaped basename leak: /public/original%20private%20scan.png".to_owned(),
            ),
            category: "Cards".to_owned(),
            tags: vec!["privacy".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "incoming/private/original private scan.png".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_original_filename_leak_with_double_url_escaped_basename() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Double Escaped Filename Leak".to_owned(),
            signer: "Filename Scan".to_owned(),
            description: Some(
                "Rendered escaped basename leak: /public/original%2520private%2520scan.png"
                    .to_owned(),
            ),
            category: "Cards".to_owned(),
            tags: vec!["privacy".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "incoming/private/original private scan.png".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_original_filename_path_leak_with_double_url_escaping() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Double Escaped Path Leak".to_owned(),
            signer: "Filename Scan".to_owned(),
            description: Some(
                "Rendered private path leak: incoming%252Fprivate%252Fimage.jpg".to_owned(),
            ),
            category: "Cards".to_owned(),
            tags: vec!["privacy".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "incoming/private/image.jpg".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_original_filename_path_leak_with_generic_basename() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Generic Basename Path Leak".to_owned(),
            signer: "Filename Scan".to_owned(),
            description: Some("Rendered private path leak: incoming/private/image.jpg".to_owned()),
            category: "Cards".to_owned(),
            tags: vec!["privacy".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "incoming/private/image.jpg".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_backslash_original_filename_path_leak_in_json_with_generic_basename() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Backslash Generic Basename Path Leak".to_owned(),
            signer: "Filename Scan".to_owned(),
            description: Some(
                "Rendered private path leak: incoming\\private\\image.jpg".to_owned(),
            ),
            category: "Cards".to_owned(),
            tags: vec!["privacy".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "incoming\\private\\image.jpg".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_original_filename_matching_static_not_found_quotes() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Quote Adjacent Card".to_owned(),
            signer: "Static Copy".to_owned(),
            description: Some(
                "A published item whose private filename matches static 404 copy.".to_owned(),
            ),
            category: "Cards".to_owned(),
            tags: vec!["quotes".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = build_original_object_key(item.id, image_id);
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "Star Wars".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_percent_encoded_private_object_key() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let image_id = Uuid::new_v4();
    let object_key = format!("originals/{}/{}", Uuid::new_v4(), image_id);
    let item = repository
        .create(AutographItemInput {
            title: "Encoded Private Key Card".to_owned(),
            signer: "Admin Copy".to_owned(),
            description: Some(format!(
                "A published item with a private object key: {}",
                object_key.replace('/', "%2F")
            )),
            category: "Cards".to_owned(),
            tags: vec!["private-key".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "private-key.png".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_double_percent_encoded_private_object_key() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let image_id = Uuid::new_v4();
    let object_key = format!("originals/{}/{}", Uuid::new_v4(), image_id);
    let item = repository
        .create(AutographItemInput {
            title: "Double Encoded Private Key Card".to_owned(),
            signer: "Admin Copy".to_owned(),
            description: Some(format!(
                "A published item with a private object key: {}",
                object_key.replace('/', "%252F")
            )),
            category: "Cards".to_owned(),
            tags: vec!["private-key".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "private-key.png".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_rejects_private_object_key_in_admin_shell_copy() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Private Key Card".to_owned(),
            signer: "Admin Copy".to_owned(),
            description: Some("A published item with a private object key.".to_owned()),
            category: "Cards".to_owned(),
            tags: vec!["private-key".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let object_key = "uploadImages".to_owned();
    let bytes = png_bytes();
    media.write(&object_key, &bytes).await.unwrap();
    repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key,
                original_filename: "private-key.png".to_owned(),
                content_type: "image/png".to_owned(),
                byte_size: bytes.len(),
                is_primary: true,
                sort_order: 0,
                alt_text: None,
            },
        )
        .await
        .unwrap();

    let error = LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap_err();

    assert!(error.contains("private source reference"));
}

#[tokio::test]
async fn publisher_detail_template_tokens_in_operator_content_render_literally() {
    let root = tempdir().unwrap();
    let media_root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(media_root.path());
    let title = "Literal {{ image_viewer }} token";
    let signer = "Literal {{ detail_groups }} token";
    let item = repository
        .create(AutographItemInput {
            title: title.to_owned(),
            signer: signer.to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: vec!["template-token".to_owned()],
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
            publication_status: PublicationStatus::Published,
        })
        .await
        .unwrap();

    LocalPublisher::new(root.path())
        .publish(&repository, &media, PublishMode::Full)
        .await
        .unwrap();

    let detail_html = fs::read_to_string(
        root.path()
            .join("current")
            .join("items")
            .join(slug_for_test(&item.title))
            .join("index.html"),
    )
    .unwrap();

    assert!(detail_html.contains(title));
    assert!(detail_html.contains(signer));
    assert_eq!(detail_html.matches("image-viewer-fallback").count(), 1);
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
    assert!(detail_html.contains("image-1-detail-"));
    assert!(detail_html.contains("image-2-detail-"));
    assert!(detail_html.contains("image-thumbnails"));
    assert_versioned_media_file(
        &current,
        &detail.images[0]
            .variants
            .iter()
            .find(|variant| variant.name == ImageVariantName::Thumbnail)
            .unwrap()
            .path,
        "primary-selection-card",
        "image-1",
        ImageVariantName::Thumbnail,
    );
    assert_versioned_media_file(
        &current,
        &detail.images[1]
            .variants
            .iter()
            .find(|variant| variant.name == ImageVariantName::Thumbnail)
            .unwrap()
            .path,
        "primary-selection-card",
        "image-2",
        ImageVariantName::Thumbnail,
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
async fn publisher_incremental_reuses_cached_derivatives_for_metadata_changes() {
    let fixture = fixture().await;
    let media = CountingMediaStore::new(fixture.media.clone());
    let publisher = LocalPublisher::new(fixture.root.path());

    publisher
        .publish(&fixture.repository, &media, PublishMode::Full)
        .await
        .unwrap();

    let current = fixture.root.path().join("current");
    let detail: PublicItemDetail = read_json(&current.join("data/items/signed-jedi-card.json"));
    let before_detail_path = detail.images[0]
        .variants
        .iter()
        .find(|variant| variant.name == ImageVariantName::Detail)
        .unwrap()
        .path
        .clone();
    let reads_after_full_publish = media.read_count();
    assert!(reads_after_full_publish > 0);

    fixture
        .repository
        .update(
            fixture.published.id,
            AutographItemUpdate {
                description: FieldPatch::Set("Updated metadata-only description".to_owned()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    publisher
        .publish(&fixture.repository, &media, PublishMode::Incremental)
        .await
        .unwrap();

    assert_eq!(
        media.read_count(),
        reads_after_full_publish + 1,
        "metadata-only incremental publish should read the source once to validate the cache key"
    );

    let current = fixture.root.path().join("current");
    let detail: PublicItemDetail = read_json(&current.join("data/items/signed-jedi-card.json"));
    let after_detail_path = detail.images[0]
        .variants
        .iter()
        .find(|variant| variant.name == ImageVariantName::Detail)
        .unwrap()
        .path
        .clone();
    assert_eq!(
        detail.description.as_deref(),
        Some("Updated metadata-only description")
    );
    assert_eq!(after_detail_path, before_detail_path);
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
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::CREATED);

    let status = app
        .clone()
        .oneshot(
            Request::get("/admin/api/publish/status")
                .header(header::COOKIE, admin_cookie(&app).await)
                .header(header::ORIGIN, "https://autographs.example.test")
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
    assert!(rendered.contains(r#""stage":"succeeded""#));
    assert!(rendered.contains(r#""itemCount":1"#));
    assert!(rendered.contains(r#""imageCount":1"#));
    assert!(rendered.contains(r#""derivativeCount":2"#));
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
    assert_eq!(status.stage.as_deref(), Some("failed"));
    assert_eq!(
        status.error.as_deref(),
        Some("Static publish failed. Check controller logs for details.")
    );
}

#[tokio::test]
async fn publisher_retention_prunes_promoted_releases_without_deleting_current_target() {
    let fixture = fixture().await;
    let publisher = LocalPublisher::with_retention_policy(
        fixture.root.path(),
        ReleaseRetentionPolicy {
            promoted_release_retain_count: 2,
            failed_candidate_retain_count: 1,
        },
    );

    for _ in 0..4 {
        publisher
            .publish(&fixture.repository, &fixture.media, PublishMode::Full)
            .await
            .unwrap();
    }

    let current = fixture.root.path().join("current");
    let current_target = current.canonicalize().unwrap();
    assert!(current.join("manifest.json").is_file());
    assert!(current_target.join("manifest.json").is_file());

    let releases = fs::read_dir(fixture.root.path().join("releases"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    assert_eq!(releases.len(), 2);
    assert!(
        releases
            .iter()
            .any(|release| release.canonicalize().unwrap() == current_target),
        "retention must keep the active current release target"
    );

    let retention = publisher.retention_status().unwrap();
    assert_eq!(retention.promoted_release_retain_count, 2);
    assert_eq!(retention.promoted_release_count, 2);
    assert_eq!(
        retention.active_release_id.as_deref(),
        publisher.status().release_id.as_deref()
    );
}

#[tokio::test]
async fn publisher_retention_prunes_failed_candidates_to_configured_newest_count() {
    let fixture = fixture().await;
    fixture
        .media
        .write(&fixture.published.images[0].object_key, b"not an image")
        .await
        .unwrap();
    let publisher = LocalPublisher::with_retention_policy(
        fixture.root.path(),
        ReleaseRetentionPolicy {
            promoted_release_retain_count: 5,
            failed_candidate_retain_count: 2,
        },
    );

    for _ in 0..4 {
        assert!(
            publisher
                .publish(&fixture.repository, &fixture.media, PublishMode::Full)
                .await
                .is_err()
        );
    }

    let failed = fs::read_dir(fixture.root.path().join("failed"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    assert_eq!(failed.len(), 2);

    let retention = publisher.retention_status().unwrap();
    assert_eq!(retention.failed_candidate_retain_count, 2);
    assert_eq!(retention.failed_candidate_count, 2);
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
            signer_credits: Vec::new(),
            characters: Vec::new(),
            format: "Trading Card".to_owned(),
            origin: ItemOrigin::Official,
            franchises: Vec::new(),
            product_line: None,
            set_name: None,
            language: "English".to_owned(),
            object_reference: Some("Card #1138".to_owned()),
            event_name: Some("Example Convention".to_owned()),
            event_location: Some("Orlando".to_owned()),
            source: Some("Vendor table".to_owned()),
            inscription: Some("For the rebellion".to_owned()),
            certification_company: Some("PSA".to_owned()),
            certification_id: Some("ABC123".to_owned()),
            estimated_year: Some(2026),
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
    png_bytes_with_color([0, 0, 0])
}

fn png_bytes_with_color(rgb: [u8; 3]) -> Vec<u8> {
    solid_png_bytes(32, 24, rgb)
}

fn same_size_replacement_png_bytes(original: &[u8]) -> Vec<u8> {
    (1..=255)
        .find_map(|value| {
            let bytes = png_bytes_with_color([value, 255 - value, value / 2]);
            (bytes.len() == original.len() && bytes != original).then_some(bytes)
        })
        .expect("test fixture should be able to create a same-size replacement PNG")
}

fn solid_png_bytes(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((height * (1 + width * 3)) as usize);
    for _ in 0..height {
        raw.push(0);
        for _ in 0..width {
            raw.extend_from_slice(&rgb);
        }
    }

    let mut zlib = vec![0x78, 0x01];
    let mut remaining = raw.as_slice();
    while !remaining.is_empty() {
        let chunk_len = remaining.len().min(u16::MAX as usize);
        let final_block = chunk_len == remaining.len();
        zlib.push(if final_block { 0x01 } else { 0x00 });
        let len = chunk_len as u16;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(&remaining[..chunk_len]);
        remaining = &remaining[chunk_len..];
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    write_png_chunk(&mut png, b"IHDR", &ihdr);
    write_png_chunk(&mut png, b"IDAT", &zlib);
    write_png_chunk(&mut png, b"IEND", &[]);
    png
}

fn write_png_chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(kind.len() + data.len());
    crc_input.extend_from_slice(kind);
    crc_input.extend_from_slice(data);
    png.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn adler32(bytes: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65_521;
    let mut a = 1u32;
    let mut b = 0u32;
    for byte in bytes {
        a = (a + *byte as u32) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }
    (b << 16) | a
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn large_png_bytes() -> Vec<u8> {
    let mut image = RgbImage::new(1800, 1400);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        *pixel = Rgb([
            ((x * 31 + y * 17) % 256) as u8,
            ((x * 13 + y * 47) % 256) as u8,
            ((x * 61 + y * 7) % 256) as u8,
        ]);
    }
    let image = DynamicImage::ImageRgb8(image);
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

fn assert_versioned_media_file(
    current: &Path,
    path: &str,
    item_slug: &str,
    image_slug: &str,
    variant: ImageVariantName,
) {
    let prefix = format!(
        "/media/{item_slug}/{image_slug}-{}-",
        variant.as_path_segment()
    );
    assert!(path.starts_with(&prefix), "unexpected media path: {path}");
    assert!(
        path.ends_with(".webp"),
        "unexpected media extension: {path}"
    );
    let fingerprint = path
        .trim_start_matches(prefix.as_str())
        .trim_end_matches(".webp");
    assert_eq!(
        fingerprint.len(),
        16,
        "unexpected media fingerprint length: {path}"
    );
    assert!(
        fingerprint
            .chars()
            .all(|character| character.is_ascii_hexdigit()),
        "unexpected media fingerprint: {path}"
    );
    assert!(
        current.join(path.trim_start_matches('/')).is_file(),
        "missing versioned media file: {path}"
    );
}

fn tamper_media_fingerprint(path: &str) -> String {
    let (prefix, suffix) = path
        .rsplit_once('-')
        .expect("fingerprinted media path separator");
    let (_, extension) = suffix
        .rsplit_once('.')
        .expect("fingerprinted media path extension");
    let replacement = if suffix.starts_with("0000000000000000") {
        "ffffffffffffffff"
    } else {
        "0000000000000000"
    };

    format!("{prefix}-{replacement}.{extension}")
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
