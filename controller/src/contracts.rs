use serde::{Deserialize, Serialize};

pub const PUBLIC_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicCatalog {
    pub schema_version: u32,
    pub items: Vec<PublicGalleryItem>,
}

impl PublicCatalog {
    pub fn new(items: Vec<PublicGalleryItem>) -> Self {
        Self {
            schema_version: PUBLIC_SCHEMA_VERSION,
            items,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicGalleryItem {
    pub slug: String,
    pub title: String,
    pub signer: String,
    pub description: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub primary_image: Option<PublicImage>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicItemDetail {
    pub schema_version: u32,
    pub slug: String,
    pub title: String,
    pub signer: String,
    pub description: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub images: Vec<PublicImage>,
    pub detail_groups: Vec<PublicDetailGroup>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicImage {
    pub alt_text: String,
    pub variants: Vec<PublicImageVariant>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicImageVariant {
    pub name: ImageVariantName,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub content_type: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublicImageVariantParams<'a> {
    pub item_slug: &'a str,
    pub image_slug: &'a str,
    pub name: ImageVariantName,
    pub fingerprint: &'a str,
    pub extension: &'a str,
    pub width: u32,
    pub height: u32,
    pub content_type: &'a str,
}

impl PublicImageVariant {
    pub fn new(params: PublicImageVariantParams<'_>) -> Self {
        Self {
            path: format!(
                "/media/{}/{}-{}-{}.{}",
                params.item_slug,
                params.image_slug,
                params.name.as_path_segment(),
                params.fingerprint,
                params.extension
            ),
            name: params.name,
            width: params.width,
            height: params.height,
            content_type: params.content_type.to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageVariantName {
    Thumbnail,
    Detail,
}

impl ImageVariantName {
    pub const fn as_path_segment(self) -> &'static str {
        match self {
            Self::Thumbnail => "thumbnail",
            Self::Detail => "detail",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicDetailGroup {
    pub label: String,
    pub fields: Vec<PublicDetailField>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicDetailField {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicFacetGroup {
    pub id: FacetId,
    pub label: String,
    pub options: Vec<PublicFacetOption>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicFacets {
    pub schema_version: u32,
    pub groups: Vec<PublicFacetGroup>,
}

impl PublicFacets {
    pub fn new(groups: Vec<PublicFacetGroup>) -> Self {
        Self {
            schema_version: PUBLIC_SCHEMA_VERSION,
            groups,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FacetId {
    Signer,
    Category,
    Tag,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicFacetOption {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishManifest {
    pub schema_version: u32,
    pub release_id: String,
    pub generated_at: String,
    pub artifacts: Vec<PublishManifestEntry>,
}

impl PublishManifest {
    pub fn new(
        release_id: impl Into<String>,
        generated_at: impl Into<String>,
        artifacts: Vec<PublishManifestEntry>,
    ) -> Self {
        Self {
            schema_version: PUBLIC_SCHEMA_VERSION,
            release_id: release_id.into(),
            generated_at: generated_at.into(),
            artifacts,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishManifestEntry {
    pub path: String,
    pub byte_size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<ImageVariantName>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_contract_serializes_camel_case_version_and_media_paths() {
        let variant = PublicImageVariant::new(PublicImageVariantParams {
            item_slug: "signed-jedi-card",
            image_slug: "front",
            name: ImageVariantName::Thumbnail,
            fingerprint: "0123456789abcdef",
            extension: "webp",
            width: 480,
            height: 640,
            content_type: "image/webp",
        });
        let catalog = PublicCatalog::new(vec![PublicGalleryItem {
            slug: "signed-jedi-card".to_owned(),
            title: "Signed Jedi Card".to_owned(),
            signer: "Mark Hamill".to_owned(),
            description: None,
            category: "Star Wars CCG".to_owned(),
            tags: vec!["jedi".to_owned()],
            primary_image: Some(PublicImage {
                alt_text: "Signed card front".to_owned(),
                variants: vec![variant],
            }),
        }]);

        let json = serde_json::to_string(&catalog).expect("serialize public catalog");

        assert!(json.contains(r#""schemaVersion":1"#));
        assert!(json.contains("/media/signed-jedi-card/front-thumbnail-0123456789abcdef.webp"));
    }
}
