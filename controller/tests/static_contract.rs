use autographs_controller::contracts::{FacetId, PublicCatalog, PublicFacets, PublicItemDetail};
use autographs_controller::publisher::{
    FixtureCatalog, FixtureRecipe, generate_split_artifacts, profile_shapes,
};

const FIXTURE: &str = include_str!("../fixtures/catalog-500.json");

#[test]
fn static_contract_profiles_shapes_and_generates_public_safe_split_artifacts() {
    let recipe: FixtureRecipe = serde_json::from_str(FIXTURE).expect("load fixture recipe");
    let catalog = FixtureCatalog::from_recipe(&recipe);

    assert_eq!(catalog.items.len(), 500);
    assert!(catalog.items.iter().all(|item| item.images.len() == 3));

    let profile = profile_shapes(&catalog);
    println!(
        "artifact shape bytes: single={}, split={}, hybrid={}",
        profile.single, profile.split, profile.hybrid
    );

    let generated = generate_split_artifacts(&catalog, "fixture-release");
    assert!(generated.artifacts.contains_key("collection.json"));
    assert!(generated.artifacts.contains_key("facets.json"));
    assert!(generated.artifacts.contains_key("manifest.json"));
    assert!(
        generated
            .artifacts
            .contains_key("items/signed-collectible-001.json")
    );
    assert!(
        generated
            .artifacts
            .contains_key("items/signed-collectible-001/index.html")
    );

    let rendered = generated
        .artifacts
        .iter()
        .map(|(path, bytes)| format!("{path}\n{}", String::from_utf8_lossy(bytes)))
        .collect::<Vec<_>>()
        .join("\n");
    let deny_list = [
        "storageNamespace",
        "bucketName",
        "objectKey",
        "objectstorage",
        "OCI_",
        "private-namespace",
        "private-originals",
        "private-original-001-1.jpg",
        "00000000-0000-4000-8000-000000010001",
    ];

    for denied in deny_list {
        assert!(
            !rendered.contains(denied),
            "generated public artifacts contain denied value: {denied}"
        );
    }

    assert!(rendered.contains(r#""schemaVersion": 2"#));
    assert!(rendered.contains("/media/signed-collectible-001/image-1-thumbnail-"));
    assert!(rendered.contains("/media/signed-collectible-001/image-1-detail-"));
}

#[test]
fn static_contract_deserializes_schema_v2_taxonomy_shapes() {
    let recipe: FixtureRecipe = serde_json::from_str(FIXTURE).expect("load fixture recipe");
    let catalog = FixtureCatalog::from_recipe(&recipe);
    let generated = generate_split_artifacts(&catalog, "fixture-release");

    let collection: PublicCatalog = serde_json::from_slice(
        generated
            .artifacts
            .get("collection.json")
            .expect("collection artifact"),
    )
    .expect("deserialize collection contract");
    assert_eq!(collection.schema_version, 2);
    let gallery = collection.items.first().expect("gallery item");
    assert!(!gallery.signer_text.is_empty());
    assert!(!gallery.signer_names.is_empty());
    assert!(!gallery.franchises.is_empty());
    assert!(!gallery.product_line.as_deref().unwrap_or("").is_empty());
    assert!(!gallery.format.is_empty());
    assert!(!gallery.origin.is_empty());
    assert!(!gallery.language.is_empty());

    let facets: PublicFacets = serde_json::from_slice(
        generated
            .artifacts
            .get("facets.json")
            .expect("facets artifact"),
    )
    .expect("deserialize facets contract");
    assert_eq!(facets.schema_version, 2);
    let facet_ids = facets
        .groups
        .iter()
        .map(|group| group.id)
        .collect::<Vec<_>>();
    assert!(facet_ids.contains(&FacetId::Signer));
    assert!(facet_ids.contains(&FacetId::Franchise));
    assert!(facet_ids.contains(&FacetId::ProductLine));
    assert!(facet_ids.contains(&FacetId::Format));
    assert!(facet_ids.contains(&FacetId::Language));
    assert!(facet_ids.contains(&FacetId::Origin));
    assert!(facet_ids.contains(&FacetId::Role));
    assert!(facet_ids.contains(&FacetId::Tag));

    let detail: PublicItemDetail = serde_json::from_slice(
        generated
            .artifacts
            .get("items/signed-collectible-001.json")
            .expect("detail artifact"),
    )
    .expect("deserialize item detail contract");
    assert_eq!(detail.schema_version, 2);
    assert!(!detail.signers.is_empty());
    assert_eq!(detail.signer_text, gallery.signer_text);
    assert_eq!(detail.signer_names, gallery.signer_names);
    assert_eq!(detail.franchises, gallery.franchises);
    assert_eq!(detail.product_line, gallery.product_line);
    assert_eq!(detail.format, gallery.format);
    assert_eq!(detail.origin, gallery.origin);
    assert_eq!(detail.language, gallery.language);
}
