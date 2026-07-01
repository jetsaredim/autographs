use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::{
    catalog::{AutographImage, AutographItem, CatalogRepository, PublicationStatus},
    contracts::{
        FacetId, ImageVariantName, PUBLIC_SCHEMA_VERSION, PublicCatalog, PublicDetailField,
        PublicDetailGroup, PublicFacetGroup, PublicFacetOption, PublicFacets, PublicGalleryItem,
        PublicImage, PublicImageVariant, PublicItemDetail, PublishManifest, PublishManifestEntry,
    },
    derivatives::{DerivativeVariant, generate_derivative},
    media::PrivateMediaStore,
};

const LANDING_HTML: &str = include_str!("../static-public/index.html");
const NOT_FOUND_HTML: &str = include_str!("../static-public/404.html");
const COLLECTION_HTML: &str = include_str!("../static-public/collection/index.html");
const BROWSE_JS: &str = include_str!("../static-public/assets/browse.js");
const DETAIL_JS: &str = include_str!("../static-public/assets/detail.js");
const FOOTER_JS: &str = include_str!("../static-public/assets/footer.js");
const LANDING_JS: &str = include_str!("../static-public/assets/landing.js");
const NOT_FOUND_JS: &str = include_str!("../static-public/assets/not-found.js");
const NOT_FOUND_QUOTES_JS: &str = include_str!("../static-public/data/not-found-quotes.json");
const SITE_CSS: &str = include_str!("../static-public/assets/site.css");
const FAVICON_ICO: &[u8] = include_bytes!("../static-public/favicon.ico");
const APP_ICON_PNG: &[u8] = include_bytes!("../static-public/icon.png");
const ARCHITECTURE_HTML: &str = include_str!("../static-public/architecture/index.html");
const ARCHITECTURE_DIAGRAM_SVG: &[u8] =
    include_bytes!("../static-public/architecture/architecture-diagram.svg");
const DETAIL_TEMPLATE: &str = include_str!("../static-public/templates/detail.html");
const ADMIN_HTML: &str = include_str!("../static-admin/index.html");
const ADMIN_JS: &str = include_str!("../static-admin/admin.js");
const ADMIN_CSS: &str = include_str!("../static-admin/admin.css");
const SAFE_PUBLISH_ERROR: &str = "Static publish failed. Check controller logs for details.";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureRecipe {
    pub item_count: usize,
    pub images_per_item: usize,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct FixtureCatalog {
    pub items: Vec<FixtureItem>,
}

#[derive(Clone, Debug)]
pub struct FixtureItem {
    pub slug: String,
    pub title: String,
    pub signer: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub images: Vec<FixtureImage>,
}

#[derive(Clone, Debug)]
pub struct FixtureImage {
    pub private_id: String,
    pub original_filename: String,
    pub storage_namespace: String,
    pub bucket_name: String,
    pub object_key: String,
    pub public_slug: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactShapeProfile {
    pub single: usize,
    pub split: usize,
    pub hybrid: usize,
}

#[derive(Clone, Debug)]
pub struct StaticArtifactSet {
    pub artifacts: BTreeMap<String, Vec<u8>>,
}

impl FixtureCatalog {
    pub fn from_recipe(recipe: &FixtureRecipe) -> Self {
        assert!(
            !recipe.categories.is_empty(),
            "fixture categories are required"
        );
        assert!(!recipe.tags.is_empty(), "fixture tags are required");
        assert!(recipe.images_per_item > 0, "fixture images are required");

        let items = (0..recipe.item_count)
            .map(|index| {
                let number = index + 1;
                let slug = format!("signed-collectible-{number:03}");
                let images = (0..recipe.images_per_item)
                    .map(|image_index| {
                        let image_number = image_index + 1;
                        FixtureImage {
                            private_id: format!(
                                "00000000-0000-4000-8000-{number:08}{image_number:04}"
                            ),
                            original_filename: format!(
                                "private-original-{number:03}-{image_number}.jpg"
                            ),
                            storage_namespace: "private-namespace".to_owned(),
                            bucket_name: "private-originals".to_owned(),
                            object_key: format!(
                                "autographs/private/{number:03}/private-original-{image_number}.jpg"
                            ),
                            public_slug: format!("image-{image_number}"),
                        }
                    })
                    .collect();

                FixtureItem {
                    slug,
                    title: format!("Signed Collectible {number:03}"),
                    signer: format!("Signer {:02}", index % 37),
                    description: format!("Public catalog fixture item {number:03}."),
                    category: recipe.categories[index % recipe.categories.len()].clone(),
                    tags: vec![
                        recipe.tags[index % recipe.tags.len()].clone(),
                        recipe.tags[(index + 3) % recipe.tags.len()].clone(),
                    ],
                    images,
                }
            })
            .collect();

        Self { items }
    }
}

pub fn profile_shapes(catalog: &FixtureCatalog) -> ArtifactShapeProfile {
    let details = catalog
        .items
        .iter()
        .map(to_public_detail)
        .collect::<Vec<_>>();
    let gallery = catalog
        .items
        .iter()
        .map(to_public_gallery_item)
        .collect::<Vec<_>>();

    let single = json_size(&details);
    let split = json_size(&PublicCatalog::new(gallery.clone()))
        + details.iter().map(json_size).sum::<usize>();
    let hybrid = json_size(&PublicCatalog::new(gallery)) + json_size(&details);

    ArtifactShapeProfile {
        single,
        split,
        hybrid,
    }
}

pub fn generate_split_artifacts(catalog: &FixtureCatalog, release_id: &str) -> StaticArtifactSet {
    let mut artifacts = BTreeMap::new();
    let gallery = catalog
        .items
        .iter()
        .map(to_public_gallery_item)
        .collect::<Vec<_>>();
    let facets = derive_facets(catalog);

    insert_json(
        &mut artifacts,
        "collection.json",
        &PublicCatalog::new(gallery),
    );
    insert_json(&mut artifacts, "facets.json", &PublicFacets::new(facets));

    for item in &catalog.items {
        insert_json(
            &mut artifacts,
            &format!("items/{}.json", item.slug),
            &to_public_detail(item),
        );
        artifacts.insert(
            format!("items/{}/index.html", item.slug),
            format!(
                "<!doctype html><title>{}</title><h1>{}</h1><p>Signed by {}</p>",
                item.title, item.title, item.signer
            )
            .into_bytes(),
        );
    }

    artifacts.insert(
        "collection/index.html".to_owned(),
        b"<!doctype html><title>Autograph Collection</title><h1>Collection</h1>".to_vec(),
    );

    let manifest_entries = artifacts
        .iter()
        .map(|(path, bytes)| PublishManifestEntry {
            path: path.clone(),
            byte_size: bytes.len(),
            content_type: None,
            variant: None,
        })
        .collect();
    insert_json(
        &mut artifacts,
        "manifest.json",
        &PublishManifest::new(release_id, "2026-01-01T00:00:00Z", manifest_entries),
    );

    StaticArtifactSet { artifacts }
}

fn to_public_gallery_item(item: &FixtureItem) -> PublicGalleryItem {
    PublicGalleryItem {
        slug: item.slug.clone(),
        title: item.title.clone(),
        signer: item.signer.clone(),
        description: Some(item.description.clone()),
        category: item.category.clone(),
        tags: item.tags.clone(),
        primary_image: item
            .images
            .first()
            .map(|image| to_public_image(item, image)),
    }
}

fn to_public_detail(item: &FixtureItem) -> PublicItemDetail {
    PublicItemDetail {
        schema_version: PUBLIC_SCHEMA_VERSION,
        slug: item.slug.clone(),
        title: item.title.clone(),
        signer: item.signer.clone(),
        description: Some(item.description.clone()),
        category: item.category.clone(),
        tags: item.tags.clone(),
        images: item
            .images
            .iter()
            .map(|image| to_public_image(item, image))
            .collect(),
        detail_groups: vec![PublicDetailGroup {
            label: "Essentials".to_owned(),
            fields: vec![
                PublicDetailField {
                    label: "Signer".to_owned(),
                    value: item.signer.clone(),
                },
                PublicDetailField {
                    label: "Category".to_owned(),
                    value: item.category.clone(),
                },
            ],
        }],
    }
}

fn to_public_image(item: &FixtureItem, image: &FixtureImage) -> PublicImage {
    PublicImage {
        alt_text: format!("{} signed by {}", item.title, item.signer),
        variants: vec![
            PublicImageVariant::new(
                &item.slug,
                &image.public_slug,
                ImageVariantName::Thumbnail,
                "webp",
                480,
                640,
                "image/webp",
            ),
            PublicImageVariant::new(
                &item.slug,
                &image.public_slug,
                ImageVariantName::Detail,
                "webp",
                1200,
                1600,
                "image/webp",
            ),
        ],
    }
}

fn derive_facets(catalog: &FixtureCatalog) -> Vec<PublicFacetGroup> {
    vec![
        facet_group(
            FacetId::Signer,
            "Signer",
            catalog.items.iter().map(|item| item.signer.clone()),
        ),
        facet_group(
            FacetId::Category,
            "Category",
            catalog.items.iter().map(|item| item.category.clone()),
        ),
        facet_group(
            FacetId::Tag,
            "IP / Genre",
            catalog.items.iter().flat_map(|item| item.tags.clone()),
        ),
    ]
}

fn facet_group(
    id: FacetId,
    label: &str,
    values: impl IntoIterator<Item = String>,
) -> PublicFacetGroup {
    let options = values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|value| PublicFacetOption {
            label: value.clone(),
            value,
        })
        .collect();

    PublicFacetGroup {
        id,
        label: label.to_owned(),
        options,
    }
}

fn insert_json<T: Serialize>(artifacts: &mut BTreeMap<String, Vec<u8>>, path: &str, value: &T) {
    artifacts.insert(
        path.to_owned(),
        serde_json::to_vec_pretty(value).expect("serialize public artifact"),
    );
}

fn json_size<T: Serialize>(value: &T) -> usize {
    serde_json::to_vec(value)
        .expect("serialize shape profile")
        .len()
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PublishMode {
    Full,
    Incremental,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishStatus {
    pub state: String,
    pub release_id: Option<String>,
    pub artifact_count: usize,
    pub byte_size: usize,
    pub started_at_epoch_seconds: Option<i64>,
    pub finished_at_epoch_seconds: Option<i64>,
    pub error: Option<String>,
}

impl Default for PublishStatus {
    fn default() -> Self {
        Self {
            state: "idle".to_owned(),
            release_id: None,
            artifact_count: 0,
            byte_size: 0,
            started_at_epoch_seconds: None,
            finished_at_epoch_seconds: None,
            error: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReleaseRetentionPolicy {
    pub promoted_release_retain_count: usize,
    pub failed_candidate_retain_count: usize,
}

impl ReleaseRetentionPolicy {
    pub const DEFAULT_PROMOTED_RELEASE_RETAIN_COUNT: usize = 5;
    pub const DEFAULT_FAILED_CANDIDATE_RETAIN_COUNT: usize = 1;

    pub fn new(promoted_release_retain_count: usize, failed_candidate_retain_count: usize) -> Self {
        Self {
            promoted_release_retain_count: retain_count_or_default(
                promoted_release_retain_count,
                Self::DEFAULT_PROMOTED_RELEASE_RETAIN_COUNT,
            ),
            failed_candidate_retain_count: retain_count_or_default(
                failed_candidate_retain_count,
                Self::DEFAULT_FAILED_CANDIDATE_RETAIN_COUNT,
            ),
        }
    }
}

impl Default for ReleaseRetentionPolicy {
    fn default() -> Self {
        Self {
            promoted_release_retain_count: Self::DEFAULT_PROMOTED_RELEASE_RETAIN_COUNT,
            failed_candidate_retain_count: Self::DEFAULT_FAILED_CANDIDATE_RETAIN_COUNT,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseRetentionStatus {
    pub active_release_id: Option<String>,
    pub promoted_release_retain_count: usize,
    pub promoted_release_count: usize,
    pub failed_candidate_retain_count: usize,
    pub failed_candidate_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublishChange {
    ItemMetadata,
    PublicationStatus,
    TagsAndFacets,
    Images,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactImpact {
    pub collection: bool,
    pub facets: bool,
    pub item_detail: bool,
    pub derivatives: bool,
}

pub fn artifact_impact_for(change: PublishChange) -> ArtifactImpact {
    match change {
        PublishChange::ItemMetadata => ArtifactImpact {
            collection: true,
            facets: false,
            item_detail: true,
            derivatives: false,
        },
        PublishChange::PublicationStatus => ArtifactImpact {
            collection: true,
            facets: true,
            item_detail: true,
            derivatives: true,
        },
        PublishChange::TagsAndFacets => ArtifactImpact {
            collection: true,
            facets: true,
            item_detail: true,
            derivatives: false,
        },
        PublishChange::Images => ArtifactImpact {
            collection: true,
            facets: false,
            item_detail: true,
            derivatives: true,
        },
    }
}

#[derive(Clone)]
pub struct LocalPublisher {
    root: Arc<PathBuf>,
    status: Arc<Mutex<PublishStatus>>,
    retention_policy: ReleaseRetentionPolicy,
}

impl LocalPublisher {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::with_retention_policy(root, ReleaseRetentionPolicy::default())
    }

    pub fn with_retention_policy(
        root: impl Into<PathBuf>,
        retention_policy: ReleaseRetentionPolicy,
    ) -> Self {
        Self {
            root: Arc::new(root.into()),
            status: Arc::new(Mutex::new(PublishStatus::default())),
            retention_policy,
        }
    }

    pub fn status(&self) -> PublishStatus {
        self.status.lock().expect("publisher status lock").clone()
    }

    pub fn retention_status(&self) -> Result<ReleaseRetentionStatus, String> {
        retention_status(&self.root, self.retention_policy)
    }

    pub async fn publish(
        &self,
        repository: &dyn CatalogRepository,
        media: &dyn PrivateMediaStore,
        mode: PublishMode,
    ) -> Result<PublishStatus, String> {
        let release_id = Uuid::new_v4().to_string();
        let candidate = self.root.join("releases").join(&release_id);
        let started_at_epoch_seconds = OffsetDateTime::now_utc().unix_timestamp();
        self.set_status(PublishStatus {
            state: "running".to_owned(),
            release_id: Some(release_id.clone()),
            started_at_epoch_seconds: Some(started_at_epoch_seconds),
            ..Default::default()
        });

        let result = self
            .build_candidate(repository, media, mode, &release_id, &candidate)
            .await
            .and_then(|_| validate_candidate(&candidate))
            .and_then(|manifest| {
                promote_candidate(&self.root, &release_id)?;
                prune_promoted_releases(&self.root, self.retention_policy)?;
                Ok(manifest)
            });

        match result {
            Ok(manifest) => {
                let status = PublishStatus {
                    state: "succeeded".to_owned(),
                    release_id: Some(release_id),
                    artifact_count: manifest.artifacts.len(),
                    byte_size: manifest
                        .artifacts
                        .iter()
                        .map(|artifact| artifact.byte_size)
                        .sum(),
                    started_at_epoch_seconds: Some(started_at_epoch_seconds),
                    finished_at_epoch_seconds: Some(OffsetDateTime::now_utc().unix_timestamp()),
                    error: None,
                };
                self.set_status(status.clone());
                Ok(status)
            }
            Err(error) => {
                retain_failed_candidates(&self.root, &candidate, self.retention_policy)?;
                let status = PublishStatus {
                    state: "failed".to_owned(),
                    release_id: Some(release_id),
                    started_at_epoch_seconds: Some(started_at_epoch_seconds),
                    finished_at_epoch_seconds: Some(OffsetDateTime::now_utc().unix_timestamp()),
                    error: Some(SAFE_PUBLISH_ERROR.to_owned()),
                    ..Default::default()
                };
                self.set_status(status);
                Err(error)
            }
        }
    }

    async fn build_candidate(
        &self,
        repository: &dyn CatalogRepository,
        media: &dyn PrivateMediaStore,
        mode: PublishMode,
        release_id: &str,
        candidate: &Path,
    ) -> Result<(), String> {
        if candidate.exists() {
            fs::remove_dir_all(candidate).map_err(|error| format!("reset candidate: {error}"))?;
        }
        if mode == PublishMode::Incremental {
            // Phase 5 uses the explicit map conservatively: without persisted
            // change events, a publish rebuilds the union of impacted surfaces.
            let _impact = [
                PublishChange::ItemMetadata,
                PublishChange::PublicationStatus,
                PublishChange::TagsAndFacets,
                PublishChange::Images,
            ]
            .map(artifact_impact_for);
        }
        fs::create_dir_all(candidate).map_err(|error| format!("create candidate: {error}"))?;
        clear_generated_surface(candidate)?;

        let mut items = repository
            .list()
            .await?
            .into_iter()
            .filter(|item| item.publication_status == PublicationStatus::Published)
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.title
                .cmp(&right.title)
                .then_with(|| left.id.cmp(&right.id))
        });
        let public_items = build_public_items(&items, media, candidate).await?;
        write_release(candidate, release_id, &public_items)?;
        validate_private_source_absence(candidate, &items)?;
        Ok(())
    }

    fn set_status(&self, status: PublishStatus) {
        *self.status.lock().expect("publisher status lock") = status;
    }
}

#[derive(Clone)]
struct PublicSourceItem {
    gallery: PublicGalleryItem,
    detail: PublicItemDetail,
}

async fn build_public_items(
    items: &[AutographItem],
    media: &dyn PrivateMediaStore,
    candidate: &Path,
) -> Result<Vec<PublicSourceItem>, String> {
    let mut used_slugs = BTreeSet::new();
    let mut public_items = Vec::new();
    for item in items {
        let slug = unique_slug(&item.title, &mut used_slugs);
        let mut images = Vec::new();
        for (index, image) in primary_first_images(&item.images).into_iter().enumerate() {
            let image_slug = format!("image-{}", index + 1);
            let source = media.read(&image.object_key).await?;
            let mut variants = Vec::new();
            for variant in [DerivativeVariant::Thumbnail, DerivativeVariant::Detail] {
                let derivative = generate_derivative(&source, variant)?;
                let relative_path = format!(
                    "media/{slug}/{image_slug}-{}.webp",
                    derivative.variant.path_segment()
                );
                write_bytes(candidate, &relative_path, &derivative.bytes)?;
                variants.push(PublicImageVariant {
                    name: match derivative.variant {
                        DerivativeVariant::Thumbnail => ImageVariantName::Thumbnail,
                        DerivativeVariant::Detail => ImageVariantName::Detail,
                    },
                    path: format!("/{relative_path}"),
                    width: derivative.width,
                    height: derivative.height,
                    content_type: derivative.content_type.to_owned(),
                });
            }
            images.push(PublicImage {
                alt_text: image
                    .alt_text
                    .clone()
                    .unwrap_or_else(|| format!("{} signed by {}", item.title, item.signer)),
                variants,
            });
        }
        let gallery = PublicGalleryItem {
            slug: slug.clone(),
            title: item.title.clone(),
            signer: item.signer.clone(),
            description: item.description.clone(),
            category: item.category.clone(),
            tags: item.tags.clone(),
            primary_image: images.first().cloned(),
        };
        let detail = PublicItemDetail {
            schema_version: PUBLIC_SCHEMA_VERSION,
            slug,
            title: item.title.clone(),
            signer: item.signer.clone(),
            description: item.description.clone(),
            category: item.category.clone(),
            tags: item.tags.clone(),
            images,
            detail_groups: vec![PublicDetailGroup {
                label: "Essentials".to_owned(),
                fields: vec![
                    PublicDetailField {
                        label: "Signer".to_owned(),
                        value: item.signer.clone(),
                    },
                    PublicDetailField {
                        label: "Category".to_owned(),
                        value: item.category.clone(),
                    },
                ],
            }],
        };
        public_items.push(PublicSourceItem { gallery, detail });
    }
    Ok(public_items)
}

fn primary_first_images(images: &[AutographImage]) -> Vec<&AutographImage> {
    let mut ordered = images.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        right
            .is_primary
            .cmp(&left.is_primary)
            .then_with(|| left.sort_order.cmp(&right.sort_order))
            .then_with(|| left.id.cmp(&right.id))
    });
    ordered
}

fn write_release(
    candidate: &Path,
    release_id: &str,
    items: &[PublicSourceItem],
) -> Result<(), String> {
    let catalog = PublicCatalog::new(items.iter().map(|item| item.gallery.clone()).collect());
    let facets = public_facets(items);
    write_bytes(candidate, "index.html", LANDING_HTML.as_bytes())?;
    write_bytes(candidate, "404.html", NOT_FOUND_HTML.as_bytes())?;
    write_bytes(candidate, "favicon.ico", FAVICON_ICO)?;
    write_bytes(candidate, "icon.png", APP_ICON_PNG)?;
    write_bytes(
        candidate,
        "collection/index.html",
        COLLECTION_HTML.as_bytes(),
    )?;
    write_bytes(candidate, "assets/browse.js", BROWSE_JS.as_bytes())?;
    write_bytes(candidate, "assets/detail.js", DETAIL_JS.as_bytes())?;
    write_bytes(candidate, "assets/footer.js", FOOTER_JS.as_bytes())?;
    write_bytes(candidate, "assets/landing.js", LANDING_JS.as_bytes())?;
    write_bytes(candidate, "assets/not-found.js", NOT_FOUND_JS.as_bytes())?;
    write_bytes(candidate, "assets/site.css", SITE_CSS.as_bytes())?;
    write_bytes(
        candidate,
        "architecture/index.html",
        ARCHITECTURE_HTML.as_bytes(),
    )?;
    write_bytes(
        candidate,
        "architecture/architecture-diagram.svg",
        ARCHITECTURE_DIAGRAM_SVG,
    )?;
    write_bytes(candidate, "admin/index.html", ADMIN_HTML.as_bytes())?;
    write_bytes(candidate, "admin/admin.js", ADMIN_JS.as_bytes())?;
    write_bytes(candidate, "admin/admin.css", ADMIN_CSS.as_bytes())?;
    write_json(candidate, "data/collection.json", &catalog)?;
    write_json(candidate, "data/facets.json", &facets)?;
    write_bytes(
        candidate,
        "data/not-found-quotes.json",
        NOT_FOUND_QUOTES_JS.as_bytes(),
    )?;
    for item in items {
        write_json(
            candidate,
            &format!("data/items/{}.json", item.detail.slug),
            &item.detail,
        )?;
        write_bytes(
            candidate,
            &format!("items/{}/index.html", item.detail.slug),
            detail_html(&item.detail).as_bytes(),
        )?;
    }
    let manifest = manifest_for(candidate, release_id)?;
    write_json(candidate, "manifest.json", &manifest)
}

pub fn validate_candidate(candidate: &Path) -> Result<PublishManifest, String> {
    for required in [
        "index.html",
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
        "assets/site.css",
        "data/collection.json",
        "data/facets.json",
        "manifest.json",
    ] {
        if !candidate.join(required).is_file() {
            return Err(format!(
                "candidate is missing required artifact: {required}"
            ));
        }
    }
    let manifest: PublishManifest = read_json(&candidate.join("manifest.json"))?;
    let catalog: PublicCatalog = read_json(&candidate.join("data/collection.json"))?;
    let _: PublicFacets = read_json(&candidate.join("data/facets.json"))?;
    for item in catalog.items {
        let detail_json = candidate.join(format!("data/items/{}.json", item.slug));
        let detail_html = candidate.join(format!("items/{}/index.html", item.slug));
        let detail: PublicItemDetail = read_json(&detail_json)?;
        if !detail_html.is_file() {
            return Err(format!(
                "candidate is missing item detail page: {}",
                item.slug
            ));
        }
        for image in detail.images {
            for variant in image.variants {
                let relative = variant.path.strip_prefix('/').ok_or_else(|| {
                    format!(
                        "candidate derivative path is not absolute: {}",
                        variant.path
                    )
                })?;
                if !candidate.join(relative).is_file() {
                    return Err(format!(
                        "candidate is missing referenced derivative: {relative}"
                    ));
                }
            }
        }
    }
    for artifact in &manifest.artifacts {
        let path = candidate.join(&artifact.path);
        if !path.is_file() {
            return Err(format!(
                "candidate is missing manifest artifact: {}",
                artifact.path
            ));
        }
        let actual_size = fs::metadata(&path)
            .map_err(|error| format!("inspect manifest artifact: {error}"))?
            .len() as usize;
        if actual_size != artifact.byte_size {
            return Err(format!(
                "candidate artifact byte size changed: {}",
                artifact.path
            ));
        }
        if artifact.variant.is_some()
            && (artifact.content_type.as_deref() != Some("image/webp")
                || !artifact.path.ends_with(".webp"))
        {
            return Err(format!(
                "candidate derivative type mismatch: {}",
                artifact.path
            ));
        }
        if artifact.variant.is_some()
            && image::guess_format(
                &fs::read(&path).map_err(|error| format!("read derivative artifact: {error}"))?,
            )
            .map_err(|error| format!("detect derivative artifact type: {error}"))?
                != image::ImageFormat::WebP
        {
            return Err(format!(
                "candidate derivative is not WebP: {}",
                artifact.path
            ));
        }
    }
    let manifest_paths = manifest
        .artifacts
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect::<BTreeSet<_>>();
    let mut files = Vec::new();
    collect_paths(candidate, &mut files)?;
    let actual_paths = files
        .into_iter()
        .filter_map(|path| {
            let relative = path
                .strip_prefix(candidate)
                .expect("candidate file relative path")
                .to_string_lossy()
                .replace('\\', "/");
            (relative != "manifest.json").then_some(relative)
        })
        .collect::<BTreeSet<_>>();
    if manifest_paths != actual_paths {
        return Err("candidate manifest inventory does not match release files".to_owned());
    }
    scan_privacy(candidate)?;
    Ok(manifest)
}

fn manifest_for(candidate: &Path, release_id: &str) -> Result<PublishManifest, String> {
    let mut artifacts = Vec::new();
    collect_files(candidate, candidate, &mut artifacts)?;
    artifacts.retain(|artifact| artifact.path != "manifest.json");
    Ok(PublishManifest::new(release_id, generated_at()?, artifacts))
}

fn generated_at() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("format publish timestamp: {error}"))
}

fn collect_files(
    root: &Path,
    path: &Path,
    artifacts: &mut Vec<PublishManifestEntry>,
) -> Result<(), String> {
    for entry in fs::read_dir(path).map_err(|error| format!("read candidate directory: {error}"))? {
        let entry = entry.map_err(|error| format!("read candidate entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, artifacts)?;
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .expect("candidate file relative path")
            .to_string_lossy()
            .replace('\\', "/");
        let derivative = relative.starts_with("media/");
        artifacts.push(PublishManifestEntry {
            path: relative,
            byte_size: entry
                .metadata()
                .map_err(|error| format!("inspect candidate artifact: {error}"))?
                .len() as usize,
            content_type: derivative.then(|| "image/webp".to_owned()),
            variant: derivative.then(|| {
                if path.to_string_lossy().contains("-thumbnail.webp") {
                    ImageVariantName::Thumbnail
                } else {
                    ImageVariantName::Detail
                }
            }),
        });
    }
    Ok(())
}

fn public_facets(items: &[PublicSourceItem]) -> PublicFacets {
    PublicFacets::new(vec![
        public_facet_group(
            FacetId::Signer,
            "Signer",
            items.iter().map(|item| item.gallery.signer.clone()),
        ),
        public_facet_group(
            FacetId::Category,
            "Category",
            items.iter().map(|item| item.gallery.category.clone()),
        ),
        public_facet_group(
            FacetId::Tag,
            "IP / Genre",
            items.iter().flat_map(|item| item.gallery.tags.clone()),
        ),
    ])
}

fn public_facet_group(
    id: FacetId,
    label: &str,
    values: impl IntoIterator<Item = String>,
) -> PublicFacetGroup {
    PublicFacetGroup {
        id,
        label: label.to_owned(),
        options: values
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|value| PublicFacetOption {
                label: value.clone(),
                value,
            })
            .collect(),
    }
}

fn clear_generated_surface(candidate: &Path) -> Result<(), String> {
    for path in [
        "index.html",
        "collection",
        "architecture",
        "admin",
        "items",
        "data",
        "media",
        "assets",
        "favicon.ico",
        "icon.png",
        "manifest.json",
    ] {
        let path = candidate.join(path);
        if path.is_dir() {
            fs::remove_dir_all(path)
                .map_err(|error| format!("remove stale candidate directory: {error}"))?;
        } else if path.exists() {
            fs::remove_file(path)
                .map_err(|error| format!("remove stale candidate file: {error}"))?;
        }
    }
    Ok(())
}

fn promote_candidate(root: &Path, release_id: &str) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|error| format!("create static root: {error}"))?;
    let current = root.join("current");
    let next = root.join(".current-next");
    if next.exists() {
        fs::remove_file(&next).map_err(|error| format!("remove stale current pointer: {error}"))?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(Path::new("releases").join(release_id), &next)
        .map_err(|error| format!("create current pointer: {error}"))?;
    #[cfg(not(unix))]
    return Err("atomic symlink promotion requires a Unix runtime".to_owned());
    fs::rename(next, current).map_err(|error| format!("promote current pointer: {error}"))
}

fn prune_promoted_releases(
    root: &Path,
    retention_policy: ReleaseRetentionPolicy,
) -> Result<(), String> {
    let releases_root = root.join("releases");
    let active_release_id = active_release_id(root)?;
    let mut retained = active_release_id.iter().cloned().collect::<BTreeSet<_>>();
    let retain_count = retention_policy.promoted_release_retain_count;

    for release in release_directories(&releases_root)? {
        if retained.len() >= retain_count {
            break;
        }
        retained.insert(release.name);
    }

    for release in release_directories(&releases_root)? {
        if !retained.contains(&release.name) {
            fs::remove_dir_all(&release.path)
                .map_err(|error| format!("prune promoted release: {error}"))?;
        }
    }
    Ok(())
}

fn retain_failed_candidates(
    root: &Path,
    candidate: &Path,
    retention_policy: ReleaseRetentionPolicy,
) -> Result<(), String> {
    let failed_root = root.join("failed");
    fs::create_dir_all(&failed_root)
        .map_err(|error| format!("create failed release root: {error}"))?;
    if candidate.exists() {
        let name = candidate.file_name().expect("candidate release id");
        fs::rename(candidate, failed_root.join(name))
            .map_err(|error| format!("retain failed candidate: {error}"))?;
    }
    let retained = release_directories(&failed_root)?
        .into_iter()
        .take(retention_policy.failed_candidate_retain_count)
        .map(|release| release.name)
        .collect::<BTreeSet<_>>();
    for release in release_directories(&failed_root)? {
        if !retained.contains(&release.name) {
            fs::remove_dir_all(&release.path)
                .map_err(|error| format!("prune failed candidate: {error}"))?;
        }
    }
    Ok(())
}

fn retention_status(
    root: &Path,
    retention_policy: ReleaseRetentionPolicy,
) -> Result<ReleaseRetentionStatus, String> {
    Ok(ReleaseRetentionStatus {
        active_release_id: active_release_id(root)?,
        promoted_release_retain_count: retention_policy.promoted_release_retain_count,
        promoted_release_count: release_directories(&root.join("releases"))?.len(),
        failed_candidate_retain_count: retention_policy.failed_candidate_retain_count,
        failed_candidate_count: release_directories(&root.join("failed"))?.len(),
    })
}

#[derive(Debug)]
struct ReleaseDirectory {
    name: String,
    path: PathBuf,
    modified: SystemTime,
}

fn release_directories(root: &Path) -> Result<Vec<ReleaseDirectory>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut releases = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| format!("read release root: {error}"))? {
        let entry = entry.map_err(|error| format!("read release entry: {error}"))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        releases.push(ReleaseDirectory {
            name,
            path,
            modified,
        });
    }
    releases.sort_by(|left, right| {
        right
            .modified
            .cmp(&left.modified)
            .then_with(|| right.name.cmp(&left.name))
    });
    Ok(releases)
}

fn active_release_id(root: &Path) -> Result<Option<String>, String> {
    let current = root.join("current");
    if !current.exists() {
        return Ok(None);
    }
    let target = fs::read_link(&current)
        .map_err(|error| format!("read current release pointer: {error}"))?;
    Ok(target
        .file_name()
        .map(|name| name.to_string_lossy().to_string()))
}

const fn retain_count_or_default(value: usize, default: usize) -> usize {
    if value == 0 { default } else { value }
}

fn scan_privacy(root: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    collect_paths(root, &mut files)?;
    for path in files {
        let relative = path.strip_prefix(root).expect("candidate path");
        let rendered = if path.extension().and_then(|extension| extension.to_str()) == Some("webp")
        {
            relative.display().to_string()
        } else {
            let text =
                fs::read(&path).map_err(|error| format!("read candidate privacy scan: {error}"))?;
            format!("{}\n{}", relative.display(), String::from_utf8_lossy(&text))
        };
        for denied in [
            "storageNamespace",
            "bucketName",
            "objectKey",
            "objectstorage",
            "OCI_",
        ] {
            if rendered.contains(denied) {
                return Err(format!(
                    "candidate privacy scan rejected denied term: {denied}"
                ));
            }
        }
    }
    Ok(())
}

fn validate_private_source_absence(root: &Path, items: &[AutographItem]) -> Result<(), String> {
    let mut high_confidence_denied = Vec::new();
    let mut low_confidence_denied = Vec::new();
    for item in items {
        for image in &item.images {
            high_confidence_denied.push(image.id.to_string());
            high_confidence_denied.push(image.object_key.clone());
            low_confidence_denied.push(image.original_filename.clone());
        }
    }
    let mut files = Vec::new();
    collect_paths(root, &mut files)?;
    for path in files {
        let relative = path.strip_prefix(root).expect("candidate path");
        let text = if is_webp_path(&path) {
            None
        } else {
            let bytes = fs::read(&path)
                .map_err(|error| format!("read candidate source privacy scan: {error}"))?;
            Some(String::from_utf8_lossy(&bytes).into_owned())
        };
        let rendered = text
            .as_ref()
            .map(|text| format!("{}\n{}", relative.display(), text))
            .unwrap_or_else(|| relative.display().to_string());
        if high_confidence_denied
            .iter()
            .filter(|value| !value.is_empty())
            .any(|value| rendered.contains(value))
        {
            return Err("candidate privacy scan rejected private source reference".to_owned());
        }
        if is_catalog_content_surface(relative)
            && text.as_deref().is_some_and(|text| {
                contains_low_confidence_source_value(text, &low_confidence_denied)
            })
        {
            return Err("candidate privacy scan rejected private source reference".to_owned());
        }
    }
    Ok(())
}

fn is_catalog_content_surface(relative: &Path) -> bool {
    relative.starts_with("data") || relative.starts_with("items")
}

fn contains_low_confidence_source_value(text: &str, denied: &[String]) -> bool {
    denied
        .iter()
        .filter(|value| !value.is_empty())
        .filter(|value| is_actionable_low_confidence_value(value))
        .any(|value| contains_standalone_value(text, value))
}

fn is_actionable_low_confidence_value(value: &str) -> bool {
    !matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "media" | "image" | "images" | "detail" | "thumbnail" | "upload"
    )
}

fn contains_standalone_value(text: &str, value: &str) -> bool {
    text.match_indices(value).any(|(index, _)| {
        let before = text[..index].chars().next_back();
        let after = text[index + value.len()..].chars().next();
        !before.is_some_and(is_filename_or_path_char)
            && !after.is_some_and(is_filename_or_path_char)
    })
}

fn is_filename_or_path_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '.' | '/')
}

fn is_webp_path(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("webp")
}

fn collect_paths(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(path).map_err(|error| format!("scan candidate directory: {error}"))? {
        let entry = entry.map_err(|error| format!("scan candidate entry: {error}"))?;
        if entry.path().is_dir() {
            collect_paths(&entry.path(), files)?;
        } else {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn write_json<T: Serialize>(root: &Path, relative: &str, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize public artifact: {error}"))?;
    write_bytes(root, relative, &bytes)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|error| format!("read JSON artifact: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parse JSON artifact: {error}"))
}

fn write_bytes(root: &Path, relative: &str, bytes: &[u8]) -> Result<(), String> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create artifact directory: {error}"))?;
    }
    fs::write(path, bytes).map_err(|error| format!("write public artifact: {error}"))
}

fn unique_slug(title: &str, used: &mut BTreeSet<String>) -> String {
    let base = slugify(title);
    let mut slug = base.clone();
    let mut suffix = 2;
    while !used.insert(slug.clone()) {
        slug = format!("{base}-{suffix}");
        suffix += 1;
    }
    slug
}

fn slugify(value: &str) -> String {
    let slug = value
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
        .join("-");
    if slug.is_empty() {
        "item".to_owned()
    } else {
        slug
    }
}

fn detail_html(item: &PublicItemDetail) -> String {
    let facts = detail_facts(item);
    let groups = detail_groups(item);
    let images = image_viewer(item);
    render_template(
        DETAIL_TEMPLATE,
        &[
            ("item_title", escape_html(&item.title)),
            ("item_signer", escape_html(&item.signer)),
            ("image_viewer", images),
            ("detail_facts", facts),
            ("detail_groups", groups),
        ],
    )
}

fn render_template(template: &str, values: &[(&str, String)]) -> String {
    let values = values
        .iter()
        .map(|(key, value)| (*key, value.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut rendered = String::with_capacity(template.len());
    let mut remaining = template;

    while let Some(start) = remaining.find("{{") {
        let token_start = start + 2;
        let Some(end) = remaining[token_start..].find("}}") else {
            break;
        };
        let token_end = token_start + end;
        let key = remaining[token_start..token_end].trim();

        rendered.push_str(&remaining[..start]);
        if let Some(value) = values.get(key) {
            rendered.push_str(value);
        } else {
            rendered.push_str(&remaining[start..token_end + 2]);
        }
        remaining = &remaining[token_end + 2..];
    }
    rendered.push_str(remaining);
    rendered
}

fn image_viewer(item: &PublicItemDetail) -> String {
    let Some(image) = item.images.first() else {
        return format!(
            "<div class=\"image-viewer-fallback\">No public image is available for {}.</div>",
            escape_html(&item.title)
        );
    };
    let Some(variant) = image_variant(image, ImageVariantName::Detail) else {
        return String::new();
    };
    let thumbnails = if item.images.len() > 1 {
        let buttons = item
            .images
            .iter()
            .enumerate()
            .filter_map(|(index, image)| {
                let thumbnail = image_variant(image, ImageVariantName::Thumbnail)?;
                let detail = image_variant(image, ImageVariantName::Detail).unwrap_or(thumbnail);
                Some(format!(
                    "<button class=\"thumbnail-button\" type=\"button\" aria-label=\"View image {}\" aria-pressed=\"{}\" data-detail-src=\"{}\" data-detail-alt=\"{}\" data-detail-width=\"{}\" data-detail-height=\"{}\"><img src=\"{}\" alt=\"{}\" width=\"{}\" height=\"{}\" draggable=\"false\"></button>",
                    index + 1,
                    if index == 0 { "true" } else { "false" },
                    escape_html(&detail.path),
                    escape_html(&image.alt_text),
                    detail.width,
                    detail.height,
                    escape_html(&thumbnail.path),
                    escape_html(&image.alt_text),
                    thumbnail.width,
                    thumbnail.height
                ))
            })
            .collect::<String>();
        format!(
            "<div class=\"image-thumbnails\" aria-label=\"{} images\">{}</div>",
            escape_html(&item.title),
            buttons
        )
    } else {
        String::new()
    };
    format!(
        "<button class=\"focused-image-button\" type=\"button\" aria-expanded=\"false\"><img src=\"{}\" alt=\"{}\" width=\"{}\" height=\"{}\" draggable=\"false\"><span class=\"sr-only\">Toggle details for {} signed by {}</span></button>{}",
        escape_html(&variant.path),
        escape_html(&image.alt_text),
        variant.width,
        variant.height,
        escape_html(&item.title),
        escape_html(&item.signer),
        thumbnails
    )
}

fn image_variant(image: &PublicImage, name: ImageVariantName) -> Option<&PublicImageVariant> {
    image
        .variants
        .iter()
        .find(|variant| variant.name == name)
        .or_else(|| image.variants.first())
}

fn detail_facts(item: &PublicItemDetail) -> String {
    let mut facts = vec![item.signer.clone(), item.category.clone()];
    facts.extend(item.tags.clone());
    facts
        .into_iter()
        .map(|fact| format!("<span>{}</span>", escape_html(&fact)))
        .collect::<String>()
}

fn detail_groups(item: &PublicItemDetail) -> String {
    item.detail_groups
        .iter()
        .map(|group| {
            let fields = group
                .fields
                .iter()
                .map(|field| {
                    format!(
                        "<div><dt>{}</dt><dd>{}</dd></div>",
                        escape_html(&field.label),
                        escape_html(&field.value)
                    )
                })
                .collect::<String>();
            format!(
                "<section class=\"metadata-group\"><h2>{}</h2><dl>{}</dl></section>",
                escape_html(&group.label),
                fields
            )
        })
        .collect::<String>()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::BTreeSet,
        fs::FileTimes,
        time::{Duration, UNIX_EPOCH},
    };

    #[test]
    fn release_directories_sort_newest_with_subsecond_precision() {
        let root = tempfile::tempdir().unwrap();
        let oldest = root.path().join("zzz-oldest");
        let middle = root.path().join("mmm-middle");
        let newest = root.path().join("aaa-newest");
        fs::create_dir(&oldest).unwrap();
        fs::create_dir(&middle).unwrap();
        fs::create_dir(&newest).unwrap();
        set_modified(&oldest, 100);
        set_modified(&middle, 200);
        set_modified(&newest, 300);

        let releases = release_directories(root.path()).unwrap();
        let modified_seconds = releases
            .iter()
            .map(|release| {
                release
                    .modified
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(modified_seconds.len(), 1);
        let names = releases
            .into_iter()
            .map(|release| release.name)
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["aaa-newest", "mmm-middle", "zzz-oldest"]);
    }

    fn set_modified(path: &Path, nanos: u32) {
        let timestamp = UNIX_EPOCH + Duration::new(1_800_000_000, nanos);
        let directory = fs::File::open(path).unwrap();
        directory
            .set_times(FileTimes::new().set_modified(timestamp))
            .unwrap();
    }
}
