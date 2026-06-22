use async_trait::async_trait;
use serde::Deserialize;

use crate::catalog::{AutographItem, CatalogRepository, PublicationStatus};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminItemFilter {
    pub query: Option<String>,
    pub signer: Option<String>,
    pub title: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub publication_status: Option<PublicationStatus>,
}

#[async_trait]
pub trait AdminCatalogRepositoryExt {
    async fn list_admin_items(&self, filter: AdminItemFilter) -> Result<Vec<AutographItem>, String>;
}

#[async_trait]
impl<T> AdminCatalogRepositoryExt for T
where
    T: CatalogRepository + ?Sized,
{
    async fn list_admin_items(&self, filter: AdminItemFilter) -> Result<Vec<AutographItem>, String> {
        let mut items = self
            .list()
            .await?
            .into_iter()
            .filter(|item| admin_item_matches(item, &filter))
            .collect::<Vec<_>>();

        items.sort_by(|left, right| {
            left.title
                .to_lowercase()
                .cmp(&right.title.to_lowercase())
                .then_with(|| left.signer.to_lowercase().cmp(&right.signer.to_lowercase()))
                .then_with(|| left.id.cmp(&right.id))
        });

        Ok(items)
    }
}

fn admin_item_matches(item: &AutographItem, filter: &AdminItemFilter) -> bool {
    query_matches(item, &filter.query)
        && text_matches(&item.signer, &filter.signer)
        && text_matches(&item.title, &filter.title)
        && text_matches(&item.category, &filter.category)
        && tags_match(&item.tags, &filter.tag)
        && filter
            .publication_status
            .is_none_or(|status| item.publication_status == status)
}

fn query_matches(item: &AutographItem, query: &Option<String>) -> bool {
    let Some(query) = normalized_query(query) else {
        return true;
    };

    contains_normalized(&item.title, &query)
        || contains_normalized(&item.signer, &query)
        || contains_normalized(&item.category, &query)
        || item.tags.iter().any(|tag| contains_normalized(tag, &query))
}

fn text_matches(value: &str, query: &Option<String>) -> bool {
    normalized_query(query).is_none_or(|query| contains_normalized(value, &query))
}

fn tags_match(tags: &[String], query: &Option<String>) -> bool {
    normalized_query(query).is_none_or(|query| {
        tags.iter()
            .any(|tag| contains_normalized(tag, &query))
    })
}

fn normalized_query(query: &Option<String>) -> Option<String> {
    query
        .as_deref()
        .map(str::trim)
        .filter(|query| !query.is_empty())
        .map(str::to_lowercase)
}

fn contains_normalized(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}
