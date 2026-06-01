use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::contracts::{
    FacetId, ImageVariantName, PUBLIC_SCHEMA_VERSION, PublicCatalog, PublicDetailField,
    PublicDetailGroup, PublicFacetGroup, PublicFacetOption, PublicFacets, PublicGalleryItem,
    PublicImage, PublicImageVariant, PublicItemDetail, PublishManifest, PublishManifestEntry,
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
