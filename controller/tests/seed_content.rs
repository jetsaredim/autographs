use autographs_controller::{
    catalog::{
        AutographImage, AutographItemInput, CatalogRepository, MemoryCatalogRepository,
        PublicationStatus,
    },
    media::{LocalMediaStore, PrivateMediaStore},
    storage_keys::build_original_object_key,
};
use tempfile::tempdir;
use uuid::Uuid;

#[tokio::test]
async fn seed_content_local_repository_and_media_use_filename_free_keys() {
    let root = tempdir().unwrap();
    let repository = MemoryCatalogRepository::default();
    let media = LocalMediaStore::new(root.path());
    let item = repository
        .create(AutographItemInput {
            title: "Signed Jedi Card".to_owned(),
            signer: "Mark Hamill".to_owned(),
            description: None,
            category: "Cards".to_owned(),
            tags: vec!["jedi".to_owned()],
            publication_status: PublicationStatus::Draft,
        })
        .await
        .unwrap();
    let image_id = Uuid::new_v4();
    let original_filename = "secret bucket photo.jpg";
    let key = build_original_object_key(item.id, image_id);

    assert!(!key.contains(original_filename));
    assert!(!key.contains(".jpg"));
    assert!(!key.contains(".png"));
    assert!(!key.contains(' '));

    media.write(&key, b"private-original-bytes").await.unwrap();
    assert_eq!(media.read(&key).await.unwrap(), b"private-original-bytes");

    let updated = repository
        .attach_image(
            item.id,
            AutographImage {
                id: image_id,
                object_key: key,
                original_filename: original_filename.to_owned(),
                content_type: "image/jpeg".to_owned(),
                byte_size: 22,
                is_primary: true,
                sort_order: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.images.len(), 1);
    assert!(updated.images[0].is_primary);
}

