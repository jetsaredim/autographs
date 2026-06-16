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
            .contains_key("collection/signed-collectible-001/index.html")
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

    assert!(rendered.contains(r#""schemaVersion": 1"#));
    assert!(rendered.contains("/media/signed-collectible-001/image-1-thumbnail.webp"));
    assert!(rendered.contains("/media/signed-collectible-001/image-1-detail.webp"));
}
