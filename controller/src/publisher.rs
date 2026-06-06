use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
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
}

impl LocalPublisher {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Arc::new(root.into()),
            status: Arc::new(Mutex::new(PublishStatus::default())),
        }
    }

    pub fn status(&self) -> PublishStatus {
        self.status.lock().expect("publisher status lock").clone()
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
                retain_latest_failed_candidate(&self.root, &candidate)?;
                let status = PublishStatus {
                    state: "failed".to_owned(),
                    release_id: Some(release_id),
                    started_at_epoch_seconds: Some(started_at_epoch_seconds),
                    finished_at_epoch_seconds: Some(OffsetDateTime::now_utc().unix_timestamp()),
                    error: Some(redact_error(&error)),
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
            if let Some(current) = current_release(&self.root)? {
                copy_tree(&current, candidate)?;
            }
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
    write_bytes(candidate, "index.html", landing_html().as_bytes())?;
    write_bytes(
        candidate,
        "collection/index.html",
        collection_html().as_bytes(),
    )?;
    write_bytes(candidate, "assets/browse.js", browse_script().as_bytes())?;
    write_json(candidate, "data/collection.json", &catalog)?;
    write_json(candidate, "data/facets.json", &facets)?;
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
        "collection/index.html",
        "assets/browse.js",
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
        "items",
        "data",
        "media",
        "assets",
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

fn current_release(root: &Path) -> Result<Option<PathBuf>, String> {
    let current = root.join("current");
    if !current.exists() {
        return Ok(None);
    }
    Ok(Some(fs::canonicalize(current).map_err(|error| {
        format!("resolve current release: {error}")
    })?))
}

fn retain_latest_failed_candidate(root: &Path, candidate: &Path) -> Result<(), String> {
    let failed_root = root.join("failed");
    fs::create_dir_all(&failed_root)
        .map_err(|error| format!("create failed release root: {error}"))?;
    for entry in
        fs::read_dir(&failed_root).map_err(|error| format!("read failed release root: {error}"))?
    {
        let path = entry
            .map_err(|error| format!("read failed release entry: {error}"))?
            .path();
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|error| format!("prune failed candidate: {error}"))?;
        }
    }
    if candidate.exists() {
        let name = candidate.file_name().expect("candidate release id");
        fs::rename(candidate, failed_root.join(name))
            .map_err(|error| format!("retain failed candidate: {error}"))?;
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("create incremental candidate: {error}"))?;
    for entry in fs::read_dir(source).map_err(|error| format!("read current release: {error}"))? {
        let entry = entry.map_err(|error| format!("read current entry: {error}"))?;
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)
                .map_err(|error| format!("copy current artifact: {error}"))?;
        }
    }
    Ok(())
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
    let mut denied = Vec::new();
    for item in items {
        for image in &item.images {
            denied.push(image.id.to_string());
            denied.push(image.original_filename.clone());
            denied.push(image.object_key.clone());
        }
    }
    let mut files = Vec::new();
    collect_paths(root, &mut files)?;
    for path in files {
        let relative = path.strip_prefix(root).expect("candidate path");
        let rendered = if path.extension().and_then(|extension| extension.to_str()) == Some("webp")
        {
            relative.display().to_string()
        } else {
            let text = fs::read(&path)
                .map_err(|error| format!("read candidate source privacy scan: {error}"))?;
            format!("{}\n{}", relative.display(), String::from_utf8_lossy(&text))
        };
        if denied
            .iter()
            .filter(|value| !value.is_empty())
            .any(|value| rendered.contains(value))
        {
            return Err("candidate privacy scan rejected private source reference".to_owned());
        }
    }
    Ok(())
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

fn landing_html() -> &'static str {
    "<!doctype html><title>Autographs</title><main><h1>Autographs</h1><a href=\"/collection/\">Browse collection</a></main>"
}

fn collection_html() -> &'static str {
    "<!doctype html><title>Autograph Collection</title><main><h1>Collection</h1><label>Category <select id=\"category\"></select></label><label>Tag <select id=\"tag\"></select></label><div id=\"collection\"></div><script src=\"/assets/browse.js\"></script></main>"
}

fn browse_script() -> &'static str {
    r#"Promise.all([fetch('/data/collection.json').then(r=>r.json()),fetch('/data/facets.json').then(r=>r.json())]).then(([catalog,facets])=>{const root=document.querySelector('#collection');const category=document.querySelector('#category');const tag=document.querySelector('#tag');const options=(id)=>(facets.groups.find(g=>g.id===id)||{options:[]}).options;const option=(value,label)=>{const node=document.createElement('option');node.value=value;node.textContent=label;return node};const fill=(node,values)=>{node.replaceChildren(option('','All'),...values.map(v=>option(v.value,v.value)))};fill(category,options('category'));fill(tag,options('tag'));const itemNode=(item)=>{const article=document.createElement('article');const link=document.createElement('a');link.href=`/items/${encodeURIComponent(item.slug)}/`;link.textContent=item.title;const signer=document.createElement('p');signer.textContent=item.signer;article.append(link,signer);return article};const render=()=>{root.replaceChildren(...catalog.items.filter(i=>(!category.value||i.category===category.value)&&(!tag.value||i.tags.includes(tag.value))).map(itemNode))};category.onchange=render;tag.onchange=render;render()})"#
}

fn detail_html(item: &PublicItemDetail) -> String {
    format!(
        "<!doctype html><title>{}</title><main><a href=\"/collection/\">Collection</a><h1>{}</h1><p>Signed by {}</p></main>",
        escape_html(&item.title),
        escape_html(&item.title),
        escape_html(&item.signer)
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn redact_error(error: &str) -> String {
    let redacted = [
        "storageNamespace",
        "bucketName",
        "objectKey",
        "objectstorage",
        "OCI_",
    ]
    .into_iter()
    .fold(error.to_owned(), |text, denied| {
        text.replace(denied, "[redacted]")
    });
    let truncated = redacted.chars().take(240).collect::<String>();
    if redacted.chars().count() > 240 {
        format!("{truncated}...")
    } else {
        truncated
    }
}
