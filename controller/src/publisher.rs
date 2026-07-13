use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::{
    catalog::{AutographImage, AutographItem, CatalogRepository, PublicationStatus},
    contracts::{
        FacetId, ImageVariantName, PUBLIC_SCHEMA_VERSION, PublicCatalog, PublicDetailField,
        PublicDetailGroup, PublicFacetGroup, PublicFacetOption, PublicFacets, PublicGalleryItem,
        PublicImage, PublicImageVariant, PublicImageVariantParams, PublicItemDetail,
        PublicSignerCredit, PublicSignerLink, PublishManifest, PublishManifestEntry,
    },
    derivatives::{DerivativeVariant, GeneratedDerivative, generate_derivative},
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
        signer_text: item.signer.clone(),
        signer_names: vec![item.signer.clone()],
        signer_roles: vec!["Signer".to_owned()],
        description: Some(item.description.clone()),
        characters: vec![item.title.clone()],
        franchises: item.tags.first().cloned().into_iter().collect(),
        product_line: Some(item.category.clone()),
        set_name: None,
        format: item.category.clone(),
        origin: "Official".to_owned(),
        language: "English".to_owned(),
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
        signer_text: item.signer.clone(),
        signer_names: vec![item.signer.clone()],
        signer_roles: vec!["Signer".to_owned()],
        signers: vec![PublicSignerCredit {
            display_name: item.signer.clone(),
            role: Some("Signer".to_owned()),
            context: None,
            links: PublicSignerLink {
                wikipedia: None,
                imdb: None,
            },
        }],
        description: Some(item.description.clone()),
        characters: vec![item.title.clone()],
        franchises: item.tags.first().cloned().into_iter().collect(),
        product_line: Some(item.category.clone()),
        set_name: None,
        format: item.category.clone(),
        origin: "Official".to_owned(),
        language: "English".to_owned(),
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
    let thumbnail_fingerprint =
        fixture_derivative_fingerprint(&item.slug, &image.public_slug, ImageVariantName::Thumbnail);
    let detail_fingerprint =
        fixture_derivative_fingerprint(&item.slug, &image.public_slug, ImageVariantName::Detail);

    PublicImage {
        alt_text: format!("{} signed by {}", item.title, item.signer),
        variants: vec![
            PublicImageVariant::new(PublicImageVariantParams {
                item_slug: &item.slug,
                image_slug: &image.public_slug,
                name: ImageVariantName::Thumbnail,
                fingerprint: &thumbnail_fingerprint,
                extension: "webp",
                width: 480,
                height: 640,
                content_type: "image/webp",
            }),
            PublicImageVariant::new(PublicImageVariantParams {
                item_slug: &item.slug,
                image_slug: &image.public_slug,
                name: ImageVariantName::Detail,
                fingerprint: &detail_fingerprint,
                extension: "webp",
                width: 1200,
                height: 1600,
                content_type: "image/webp",
            }),
        ],
    }
}

fn fixture_derivative_fingerprint(
    item_slug: &str,
    image_slug: &str,
    variant: ImageVariantName,
) -> String {
    let seed = format!("{item_slug}/{image_slug}/{}", variant.as_path_segment());
    public_derivative_fingerprint(seed.as_bytes())
}

fn derive_facets(catalog: &FixtureCatalog) -> Vec<PublicFacetGroup> {
    vec![
        facet_group(
            FacetId::Signer,
            "Signer",
            catalog.items.iter().map(|item| item.signer.clone()),
        ),
        facet_group(
            FacetId::Franchise,
            "Franchise",
            catalog
                .items
                .iter()
                .filter_map(|item| item.tags.first().cloned()),
        ),
        facet_group(
            FacetId::ProductLine,
            "Product Line",
            catalog.items.iter().map(|item| item.category.clone()),
        ),
        facet_group(
            FacetId::Format,
            "Format",
            catalog.items.iter().map(|item| item.category.clone()),
        ),
        facet_group(
            FacetId::Language,
            "Language",
            catalog.items.iter().map(|_| "English".to_owned()),
        ),
        facet_group(
            FacetId::Origin,
            "Origin",
            catalog.items.iter().map(|_| "Official".to_owned()),
        ),
        facet_group(
            FacetId::Role,
            "Role",
            catalog.items.iter().map(|_| "Signer".to_owned()),
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
    pub stage: Option<String>,
    pub release_id: Option<String>,
    pub artifact_count: usize,
    pub byte_size: usize,
    pub item_count: usize,
    pub image_count: usize,
    pub derivative_count: usize,
    pub started_at_epoch_seconds: Option<i64>,
    pub finished_at_epoch_seconds: Option<i64>,
    pub error: Option<String>,
}

impl Default for PublishStatus {
    fn default() -> Self {
        Self {
            state: "idle".to_owned(),
            stage: None,
            release_id: None,
            artifact_count: 0,
            byte_size: 0,
            item_count: 0,
            image_count: 0,
            derivative_count: 0,
            started_at_epoch_seconds: None,
            finished_at_epoch_seconds: None,
            error: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PublishStage {
    Accepted,
    LoadingCatalog,
    GeneratingDerivatives,
    WritingCandidate,
    ValidatingCandidate,
    PromotingRelease,
    Succeeded,
    Failed,
}

impl PublishStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::LoadingCatalog => "loadingCatalog",
            Self::GeneratingDerivatives => "generatingDerivatives",
            Self::WritingCandidate => "writingCandidate",
            Self::ValidatingCandidate => "validatingCandidate",
            Self::PromotingRelease => "promotingRelease",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, Default)]
struct PublishProgress {
    item_count: usize,
    image_count: usize,
    derivative_count: usize,
    generated_derivative_count: usize,
    reused_derivative_count: usize,
}

struct BuildPublicItemsResult {
    items: Vec<PublicSourceItem>,
    progress: PublishProgress,
}

struct DerivativeCache {
    root: PathBuf,
}

impl DerivativeCache {
    fn new(static_root: &Path) -> Self {
        Self {
            root: static_root.join(".derivative-cache"),
        }
    }

    fn path_for(&self, image: &AutographImage, variant: DerivativeVariant) -> PathBuf {
        let mut digest = Sha256::new();
        digest.update(b"autographs-derivative-cache-v1");
        digest.update([0]);
        digest.update(image.id.as_bytes());
        digest.update([0]);
        digest.update(image.object_key.as_bytes());
        digest.update([0]);
        digest.update(image.content_type.as_bytes());
        digest.update([0]);
        digest.update(image.byte_size.to_string().as_bytes());
        digest.update([0]);
        digest.update(variant.path_segment().as_bytes());
        let key = digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        self.root
            .join(variant.path_segment())
            .join(format!("{key}.webp"))
    }

    fn read(
        &self,
        image: &AutographImage,
        variant: DerivativeVariant,
    ) -> Result<Option<GeneratedDerivative>, String> {
        let path = self.path_for(image, variant);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path).map_err(|error| format!("read derivative cache: {error}"))?;
        let decoded = image::load_from_memory(&bytes)
            .map_err(|error| format!("decode derivative cache: {error}"))?;
        Ok(Some(GeneratedDerivative {
            variant,
            width: decoded.width(),
            height: decoded.height(),
            content_type: "image/webp",
            bytes,
        }))
    }

    fn write(
        &self,
        image: &AutographImage,
        derivative: &GeneratedDerivative,
    ) -> Result<(), String> {
        let path = self.path_for(image, derivative.variant);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("create derivative cache directory: {error}"))?;
        }
        fs::write(path, &derivative.bytes)
            .map_err(|error| format!("write derivative cache: {error}"))
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
            stage: Some(PublishStage::Accepted.as_str().to_owned()),
            release_id: Some(release_id.clone()),
            started_at_epoch_seconds: Some(started_at_epoch_seconds),
            ..Default::default()
        });
        tracing::info!(
            release_id = %release_id,
            mode = ?mode,
            stage = PublishStage::Accepted.as_str(),
            "static publish accepted"
        );

        let result = self
            .build_candidate(repository, media, mode, &release_id, &candidate)
            .await
            .and_then(|progress| {
                self.update_progress(&release_id, PublishStage::ValidatingCandidate, &progress);
                let manifest = validate_candidate(&candidate)?;
                tracing::info!(
                    release_id = %release_id,
                    mode = ?mode,
                    stage = PublishStage::ValidatingCandidate.as_str(),
                    artifact_count = manifest.artifacts.len(),
                    byte_size = manifest.artifacts.iter().map(|artifact| artifact.byte_size).sum::<usize>(),
                    "static publish validation completed"
                );
                Ok((manifest, progress))
            })
            .and_then(|(manifest, progress)| {
                self.update_progress(&release_id, PublishStage::PromotingRelease, &progress);
                promote_candidate(&self.root, &release_id)?;
                prune_promoted_releases(&self.root, self.retention_policy)?;
                tracing::info!(
                    release_id = %release_id,
                    mode = ?mode,
                    stage = PublishStage::PromotingRelease.as_str(),
                    "static publish release promotion completed"
                );
                Ok((manifest, progress))
            });

        match result {
            Ok((manifest, progress)) => {
                let status = PublishStatus {
                    state: "succeeded".to_owned(),
                    stage: Some(PublishStage::Succeeded.as_str().to_owned()),
                    release_id: Some(release_id),
                    artifact_count: manifest.artifacts.len(),
                    byte_size: manifest
                        .artifacts
                        .iter()
                        .map(|artifact| artifact.byte_size)
                        .sum(),
                    item_count: progress.item_count,
                    image_count: progress.image_count,
                    derivative_count: progress.derivative_count,
                    started_at_epoch_seconds: Some(started_at_epoch_seconds),
                    finished_at_epoch_seconds: Some(OffsetDateTime::now_utc().unix_timestamp()),
                    error: None,
                };
                self.set_status(status.clone());
                Ok(status)
            }
            Err(error) => {
                let failed_stage = self
                    .status()
                    .stage
                    .unwrap_or_else(|| PublishStage::Accepted.as_str().to_owned());
                let error_kind = classify_publish_error(&failed_stage, &error);
                tracing::error!(
                    release_id = %release_id,
                    mode = ?mode,
                    failed_stage = %failed_stage,
                    error_kind,
                    "static publish failed"
                );
                retain_failed_candidates(&self.root, &candidate, self.retention_policy)?;
                let status = PublishStatus {
                    state: "failed".to_owned(),
                    stage: Some(PublishStage::Failed.as_str().to_owned()),
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
    ) -> Result<PublishProgress, String> {
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

        self.update_progress(
            release_id,
            PublishStage::LoadingCatalog,
            &PublishProgress::default(),
        );
        let mut items = repository
            .list()
            .await?
            .into_iter()
            .filter(|item| item.publication_status == PublicationStatus::Published)
            .collect::<Vec<_>>();
        let mut progress = PublishProgress {
            item_count: items.len(),
            ..Default::default()
        };
        tracing::info!(
            release_id = %release_id,
            mode = ?mode,
            stage = PublishStage::LoadingCatalog.as_str(),
            item_count = progress.item_count,
            "static publish catalog load completed"
        );
        items.sort_by(|left, right| {
            left.title
                .cmp(&right.title)
                .then_with(|| left.id.cmp(&right.id))
        });
        self.update_progress(release_id, PublishStage::GeneratingDerivatives, &progress);
        let derivative_cache = DerivativeCache::new(&self.root);
        let public_items = build_public_items(&items, media, candidate, &derivative_cache).await?;
        progress.image_count = public_items.progress.image_count;
        progress.derivative_count = public_items.progress.derivative_count;
        progress.generated_derivative_count = public_items.progress.generated_derivative_count;
        progress.reused_derivative_count = public_items.progress.reused_derivative_count;
        tracing::info!(
            release_id = %release_id,
            mode = ?mode,
            stage = PublishStage::GeneratingDerivatives.as_str(),
            image_count = progress.image_count,
            derivative_count = progress.derivative_count,
            generated_derivative_count = progress.generated_derivative_count,
            reused_derivative_count = progress.reused_derivative_count,
            skipped_derivative_count = 0usize,
            "static publish derivative generation completed"
        );
        self.update_progress(release_id, PublishStage::WritingCandidate, &progress);
        write_release(candidate, release_id, &public_items.items)?;
        validate_private_source_absence(candidate, &items)?;
        tracing::info!(
            release_id = %release_id,
            mode = ?mode,
            stage = PublishStage::WritingCandidate.as_str(),
            item_count = progress.item_count,
            image_count = progress.image_count,
            derivative_count = progress.derivative_count,
            "static publish candidate release generation completed"
        );
        Ok(progress)
    }

    fn set_status(&self, status: PublishStatus) {
        *self.status.lock().expect("publisher status lock") = status;
    }

    fn update_progress(&self, release_id: &str, stage: PublishStage, progress: &PublishProgress) {
        let mut status = self.status();
        status.state = "running".to_owned();
        status.stage = Some(stage.as_str().to_owned());
        status.release_id = Some(release_id.to_owned());
        status.item_count = progress.item_count;
        status.image_count = progress.image_count;
        status.derivative_count = progress.derivative_count;
        self.set_status(status);
        tracing::info!(
            release_id = %release_id,
            stage = stage.as_str(),
            item_count = progress.item_count,
            image_count = progress.image_count,
            derivative_count = progress.derivative_count,
            "static publish stage started"
        );
    }
}

pub(crate) fn classify_publish_error(stage: &str, error: &str) -> &'static str {
    let normalized = error.to_ascii_lowercase();
    if normalized.contains("privacy scan") || normalized.contains("private source reference") {
        return "privacy_validation";
    }
    if normalized.contains("candidate")
        && (normalized.contains("missing")
            || normalized.contains("manifest")
            || normalized.contains("fingerprint")
            || normalized.contains("validation"))
    {
        return "candidate_validation";
    }
    if normalized.contains("derivative")
        || normalized.contains("image")
        || normalized.contains("webp")
        || normalized.contains("media")
    {
        return "media_derivative";
    }
    if normalized.contains("promote") {
        return "release_promotion";
    }
    if normalized.contains("prune") || normalized.contains("retain failed candidate") {
        return "release_retention";
    }
    if normalized.contains("catalog") || normalized.contains("repository") {
        return "catalog";
    }

    match stage {
        "loadingCatalog" => "catalog",
        "generatingDerivatives" => "media_derivative",
        "writingCandidate" => "candidate_generation",
        "validatingCandidate" => "candidate_validation",
        "promotingRelease" => "release_promotion",
        _ => "publish",
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
    derivative_cache: &DerivativeCache,
) -> Result<BuildPublicItemsResult, String> {
    let mut used_slugs = BTreeSet::new();
    let mut public_items = Vec::new();
    let mut progress = PublishProgress {
        item_count: items.len(),
        ..Default::default()
    };
    for item in items {
        let slug = unique_slug(&item.title, &mut used_slugs);
        let mut images = Vec::new();
        for (index, image) in primary_first_images(&item.images).into_iter().enumerate() {
            progress.image_count += 1;
            let image_slug = format!("image-{}", index + 1);
            let mut source = None;
            let mut variants = Vec::new();
            for variant in [DerivativeVariant::Thumbnail, DerivativeVariant::Detail] {
                let derivative = match derivative_cache.read(image, variant) {
                    Ok(Some(derivative)) => {
                        progress.reused_derivative_count += 1;
                        derivative
                    }
                    Ok(None) => {
                        let source = match &source {
                            Some(source) => source,
                            None => {
                                source = Some(media.read(&image.object_key).await?);
                                source.as_ref().expect("source loaded")
                            }
                        };
                        let derivative = generate_derivative(source, variant)?;
                        if let Err(error) = derivative_cache.write(image, &derivative) {
                            tracing::warn!(%error, "failed to update derivative cache");
                        }
                        progress.generated_derivative_count += 1;
                        derivative
                    }
                    Err(error) => {
                        tracing::warn!(%error, "ignored unreadable derivative cache entry");
                        let source = match &source {
                            Some(source) => source,
                            None => {
                                source = Some(media.read(&image.object_key).await?);
                                source.as_ref().expect("source loaded")
                            }
                        };
                        let derivative = generate_derivative(source, variant)?;
                        if let Err(error) = derivative_cache.write(image, &derivative) {
                            tracing::warn!(%error, "failed to update derivative cache");
                        }
                        progress.generated_derivative_count += 1;
                        derivative
                    }
                };
                progress.derivative_count += 1;
                let fingerprint = public_derivative_fingerprint(&derivative.bytes);
                let relative_path = format!(
                    "media/{slug}/{image_slug}-{}-{fingerprint}.webp",
                    derivative.variant.path_segment(),
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
            signer_text: public_signer_text(item),
            signer_names: public_signer_names(item),
            signer_roles: public_signer_roles(item),
            description: item.description.clone(),
            characters: item.characters.clone(),
            franchises: item.franchises.clone(),
            product_line: item.product_line.clone(),
            set_name: item.set_name.clone(),
            format: item.format.clone(),
            origin: format!("{:?}", item.origin),
            language: item.language.clone(),
            tags: item.tags.clone(),
            primary_image: images.first().cloned(),
        };
        let detail = PublicItemDetail {
            schema_version: PUBLIC_SCHEMA_VERSION,
            slug,
            title: item.title.clone(),
            signer_text: public_signer_text(item),
            signer_names: public_signer_names(item),
            signer_roles: public_signer_roles(item),
            signers: public_signer_credits(item),
            description: item.description.clone(),
            characters: item.characters.clone(),
            franchises: item.franchises.clone(),
            product_line: item.product_line.clone(),
            set_name: item.set_name.clone(),
            format: item.format.clone(),
            origin: format!("{:?}", item.origin),
            language: item.language.clone(),
            tags: item.tags.clone(),
            images,
            detail_groups: public_detail_groups(item),
        };
        public_items.push(PublicSourceItem { gallery, detail });
    }
    Ok(BuildPublicItemsResult {
        items: public_items,
        progress,
    })
}

fn public_signer_names(item: &AutographItem) -> Vec<String> {
    let names = item
        .signer_credits
        .iter()
        .map(|credit| credit.signer.display_name.clone())
        .filter(|name| !name.trim().is_empty())
        .collect::<Vec<_>>();
    if names.is_empty() {
        vec![item.signer.clone()]
    } else {
        names
    }
}

fn public_signer_roles(item: &AutographItem) -> Vec<String> {
    item.signer_credits
        .iter()
        .filter_map(|credit| {
            credit
                .item_role
                .clone()
                .or_else(|| credit.signer.default_role.clone())
        })
        .filter(|role| !role.trim().is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn public_signer_text(item: &AutographItem) -> String {
    compact_signer_text(&public_signer_names(item))
}

fn compact_signer_text(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [one, two] => format!("{one} + {two}"),
        [one, two, rest @ ..] => format!("{one}, {two} + {} more", rest.len()),
    }
}

fn public_signer_credits(item: &AutographItem) -> Vec<PublicSignerCredit> {
    let credits = item
        .signer_credits
        .iter()
        .map(|credit| PublicSignerCredit {
            display_name: credit.signer.display_name.clone(),
            role: credit
                .item_role
                .clone()
                .or_else(|| credit.signer.default_role.clone()),
            context: credit.item_context.clone(),
            links: PublicSignerLink {
                wikipedia: credit.signer.wikipedia_url.clone(),
                imdb: credit.signer.imdb_url.clone(),
            },
        })
        .collect::<Vec<_>>();
    if credits.is_empty() {
        vec![PublicSignerCredit {
            display_name: item.signer.clone(),
            role: None,
            context: None,
            links: PublicSignerLink {
                wikipedia: None,
                imdb: None,
            },
        }]
    } else {
        credits
    }
}

fn public_derivative_fingerprint(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn derivative_fingerprint_from_path(
    path: &str,
    variant: ImageVariantName,
) -> Result<String, String> {
    let marker = format!("-{}-", variant.as_path_segment());
    let fingerprint = path
        .strip_suffix(".webp")
        .and_then(|path| {
            path.rsplit_once(marker.as_str())
                .map(|(_, fingerprint)| fingerprint)
        })
        .ok_or_else(|| format!("candidate derivative path is not fingerprinted: {path}"))?;
    if fingerprint.len() != 16
        || !fingerprint
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(format!(
            "candidate derivative fingerprint is malformed: {path}"
        ));
    }
    Ok(fingerprint.to_owned())
}

fn public_detail_groups(item: &AutographItem) -> Vec<PublicDetailGroup> {
    let mut groups = vec![PublicDetailGroup {
        label: "Details".to_owned(),
        fields: compact_detail_fields(vec![
            ("Format", Some(item.format.clone())),
            (
                "Language",
                (item.language != "English").then(|| item.language.clone()),
            ),
            (
                "Origin",
                (item.origin != crate::catalog::ItemOrigin::Official)
                    .then(|| format!("{:?}", item.origin)),
            ),
            (
                "Characters",
                (!item.characters.is_empty()).then(|| item.characters.join(", ")),
            ),
            (
                "Franchise",
                (!item.franchises.is_empty()).then(|| item.franchises.join(", ")),
            ),
            ("Product Line", item.product_line.clone()),
            ("Set", item.set_name.clone()),
            (
                "Estimated year",
                item.estimated_year.as_ref().map(|year| year.to_string()),
            ),
            ("Object reference", item.object_reference.clone()),
        ]),
    }];

    let story = compact_detail_fields(vec![
        ("Description", item.description.clone()),
        ("Inscription", item.inscription.clone()),
    ]);
    if !story.is_empty() {
        groups.push(PublicDetailGroup {
            label: "Story".to_owned(),
            fields: story,
        });
    }

    let provenance = compact_detail_fields(vec![
        ("Event", item.event_name.clone()),
        ("Event location", item.event_location.clone()),
        ("Source", item.source.clone()),
    ]);
    if !provenance.is_empty() {
        groups.push(PublicDetailGroup {
            label: "Provenance".to_owned(),
            fields: provenance,
        });
    }

    let certification = compact_detail_fields(vec![
        ("Company", item.certification_company.clone()),
        ("Certification ID", item.certification_id.clone()),
    ]);
    if !certification.is_empty() {
        groups.push(PublicDetailGroup {
            label: "Certification".to_owned(),
            fields: certification,
        });
    }

    groups
}

fn compact_detail_fields(fields: Vec<(&str, Option<String>)>) -> Vec<PublicDetailField> {
    fields
        .into_iter()
        .filter_map(|(label, value)| {
            let value = value?;
            let value = value.trim();
            (!value.is_empty()).then(|| PublicDetailField {
                label: label.to_owned(),
                value: value.to_owned(),
            })
        })
        .collect()
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
        if let Some(variant) = artifact.variant {
            if artifact.content_type.as_deref() != Some("image/webp")
                || !artifact.path.ends_with(".webp")
            {
                return Err(format!(
                    "candidate derivative type mismatch: {}",
                    artifact.path
                ));
            }
            let derivative_bytes =
                fs::read(&path).map_err(|error| format!("read derivative artifact: {error}"))?;
            if image::guess_format(&derivative_bytes)
                .map_err(|error| format!("detect derivative artifact type: {error}"))?
                != image::ImageFormat::WebP
            {
                return Err(format!(
                    "candidate derivative is not WebP: {}",
                    artifact.path
                ));
            }
            let path_fingerprint = derivative_fingerprint_from_path(&artifact.path, variant)?;
            let byte_fingerprint = public_derivative_fingerprint(&derivative_bytes);
            if path_fingerprint != byte_fingerprint {
                return Err(format!(
                    "candidate derivative fingerprint mismatch: {}",
                    artifact.path
                ));
            }
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
            variant: derivative.then(|| media_variant_from_path(&path)),
        });
    }
    Ok(())
}

fn media_variant_from_path(path: &Path) -> ImageVariantName {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let stem = file_name.strip_suffix(".webp").unwrap_or(file_name);
    let Some((before_fingerprint, _fingerprint)) = stem.rsplit_once('-') else {
        return ImageVariantName::Detail;
    };
    match before_fingerprint
        .rsplit_once('-')
        .map(|(_, variant)| variant)
    {
        Some("thumbnail") => ImageVariantName::Thumbnail,
        Some("detail") => ImageVariantName::Detail,
        _ => ImageVariantName::Detail,
    }
}

fn public_facets(items: &[PublicSourceItem]) -> PublicFacets {
    PublicFacets::new(vec![
        public_facet_group(
            FacetId::Signer,
            "Signer",
            items
                .iter()
                .flat_map(|item| item.gallery.signer_names.clone()),
        ),
        public_facet_group(
            FacetId::Franchise,
            "Franchise",
            items
                .iter()
                .flat_map(|item| item.gallery.franchises.clone()),
        ),
        public_facet_group(
            FacetId::ProductLine,
            "Product Line",
            items
                .iter()
                .filter_map(|item| item.gallery.product_line.clone()),
        ),
        public_facet_group(
            FacetId::Format,
            "Format",
            items.iter().map(|item| item.gallery.format.clone()),
        ),
        public_facet_group(
            FacetId::Language,
            "Language",
            items.iter().map(|item| item.gallery.language.clone()),
        ),
        public_facet_group(
            FacetId::Origin,
            "Origin",
            items.iter().map(|item| item.gallery.origin.clone()),
        ),
        public_facet_group(
            FacetId::Role,
            "Role",
            items
                .iter()
                .flat_map(|item| item.gallery.signer_roles.clone()),
        ),
        public_facet_group(
            FacetId::Tag,
            "Tags",
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
        if contains_high_confidence_source_value(&rendered, &high_confidence_denied) {
            return Err("candidate privacy scan rejected private source reference".to_owned());
        }
        if contains_low_confidence_source_value(&rendered, &low_confidence_denied) {
            return Err("candidate privacy scan rejected private source reference".to_owned());
        }
    }
    Ok(())
}

fn contains_high_confidence_source_value(text: &str, denied: &[String]) -> bool {
    denied
        .iter()
        .filter(|value| !value.is_empty())
        .any(|value| text.contains(value))
        || {
            let normalized_text = normalize_source_scan_text(text);
            denied
                .iter()
                .filter(|value| !value.is_empty())
                .map(|value| normalize_source_scan_text(value))
                .filter(|value| !value.is_empty())
                .any(|value| normalized_text.contains(&value))
        }
}

fn contains_low_confidence_source_value(text: &str, denied: &[String]) -> bool {
    let normalized_text = normalize_source_scan_text(text);
    denied
        .iter()
        .filter(|value| !value.is_empty())
        .flat_map(|value| normalized_original_filename_values(value))
        .filter(|value| is_actionable_low_confidence_value(value))
        .any(|value| normalized_text.contains(&value))
}

fn is_actionable_low_confidence_value(value: &str) -> bool {
    let normalized = normalize_source_scan_text(value.trim());
    if normalized.contains('/') {
        return true;
    }

    let file_name = normalized.rsplit('/').next().unwrap_or(normalized.as_str());
    let (stem, extension) = file_name
        .rsplit_once('.')
        .map_or((file_name, None), |(stem, extension)| {
            (stem, Some(extension))
        });

    !(is_generic_original_filename_term(file_name)
        || is_generic_original_filename_term(stem)
            && extension.is_some_and(is_generic_original_filename_term))
}

fn is_generic_original_filename_term(value: &str) -> bool {
    matches!(
        value,
        "upload"
            | "image"
            | "images"
            | "detail"
            | "thumbnail"
            | "media"
            | "original"
            | "file"
            | "photo"
            | "jpg"
            | "jpeg"
            | "png"
            | "webp"
    )
}

fn normalized_original_filename_values(value: &str) -> Vec<String> {
    let normalized = normalize_source_scan_text(value.trim());
    let file_name = normalized.rsplit('/').next().unwrap_or(normalized.as_str());
    let mut values = Vec::new();
    for candidate in [normalized.as_str(), file_name] {
        if !candidate.is_empty() && !values.iter().any(|value| value == candidate) {
            values.push(candidate.to_owned());
        }
    }
    values
}

fn normalize_source_scan_text(value: &str) -> String {
    collapse_slash_runs(&iterative_percent_decode_lossy(value).replace('\\', "/")).to_lowercase()
}

fn iterative_percent_decode_lossy(value: &str) -> String {
    let mut decoded = value.to_owned();
    for _ in 0..4 {
        let next = percent_decode_lossy(&decoded);
        if next == decoded {
            break;
        }
        decoded = next;
    }
    decoded
}

fn collapse_slash_runs(value: &str) -> String {
    let mut collapsed = String::with_capacity(value.len());
    let mut previous_was_slash = false;
    for character in value.chars() {
        if character == '/' {
            if !previous_was_slash {
                collapsed.push(character);
            }
            previous_was_slash = true;
        } else {
            collapsed.push(character);
            previous_was_slash = false;
        }
    }
    collapsed
}

fn percent_decode_lossy(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
        {
            decoded.push(high << 4 | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
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
            ("item_signer", escape_html(&item.signer_text)),
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
        escape_html(&item.signer_text),
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
    let mut facts = vec![item.signer_text.clone(), item.format.clone()];
    facts.extend(item.franchises.clone());
    facts.extend(item.tags.clone());
    facts
        .into_iter()
        .map(|fact| format!("<span>{}</span>", escape_html(&fact)))
        .collect::<String>()
}

fn detail_groups(item: &PublicItemDetail) -> String {
    let mut rendered = signer_detail_group(item);
    rendered.push_str(
        &item
            .detail_groups
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
            .collect::<String>(),
    );
    rendered
}

fn signer_detail_group(item: &PublicItemDetail) -> String {
    if item.signers.is_empty() {
        return String::new();
    }
    let rows = item
        .signers
        .iter()
        .map(|signer| {
            let mut name = format!(
                "<span class=\"signer-name\">{}</span>",
                escape_html(&signer.display_name)
            );
            let links = signer_profile_links(signer);
            if !links.is_empty() {
                name.push_str(&format!("<span class=\"profile-links\">{links}</span>"));
            }
            let role_context = [signer.role.as_deref(), signer.context.as_deref()]
                .into_iter()
                .flatten()
                .filter(|value| !value.trim().is_empty())
                .map(escape_html)
                .collect::<Vec<_>>()
                .join(" - ");
            let meta = if role_context.is_empty() {
                String::new()
            } else {
                format!("<span class=\"signer-context\">{role_context}</span>")
            };
            format!("<div class=\"signer-credit-row\"><dt>{name}</dt><dd>{meta}</dd></div>")
        })
        .collect::<String>();
    format!(
        "<section class=\"metadata-group signer-metadata-group\"><h2>Signers</h2><dl>{rows}</dl></section>"
    )
}

fn signer_profile_links(signer: &PublicSignerCredit) -> String {
    let mut links = String::new();
    if let Some(url) = signer
        .links
        .wikipedia
        .as_deref()
        .filter(|url| !url.trim().is_empty())
    {
        links.push_str(&format!(
            "<a class=\"profile-link profile-link-wikipedia\" href=\"{}\" aria-label=\"Open Wikipedia profile for {}\" rel=\"noopener noreferrer\">W</a>",
            escape_html(url),
            escape_html(&signer.display_name)
        ));
    }
    if let Some(url) = signer
        .links
        .imdb
        .as_deref()
        .filter(|url| !url.trim().is_empty())
    {
        links.push_str(&format!(
            "<a class=\"profile-link profile-link-imdb\" href=\"{}\" aria-label=\"Open IMDb profile for {}\" rel=\"noopener noreferrer\">IMDb</a>",
            escape_html(url),
            escape_html(&signer.display_name)
        ));
    }
    links
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

    #[test]
    fn publish_error_classifier_keeps_diagnostics_privacy_safe() {
        assert_eq!(
            classify_publish_error(
                "generatingDerivatives",
                "read private media from bucket autographs-media-prod/private/originals/example.jpg",
            ),
            "media_derivative"
        );
        assert_eq!(
            classify_publish_error(
                "validatingCandidate",
                "candidate derivative fingerprint mismatch: media/item/image-detail-abcd.webp",
            ),
            "candidate_validation"
        );
        assert_eq!(
            classify_publish_error(
                "validatingCandidate",
                "candidate privacy scan rejected private source reference",
            ),
            "privacy_validation"
        );
        assert_eq!(
            classify_publish_error(
                "promotingRelease",
                "promote current pointer: permission denied"
            ),
            "release_promotion"
        );
        assert_eq!(
            classify_publish_error("loadingCatalog", "connection failed"),
            "catalog"
        );
    }

    fn set_modified(path: &Path, nanos: u32) {
        let timestamp = UNIX_EPOCH + Duration::new(1_800_000_000, nanos);
        let directory = fs::File::open(path).unwrap();
        directory
            .set_times(FileTimes::new().set_modified(timestamp))
            .unwrap();
    }
}
