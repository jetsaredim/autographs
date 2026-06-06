use uuid::Uuid;

pub fn build_original_object_key(item_id: Uuid, image_id: Uuid) -> String {
    format!("originals/{item_id}/{image_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_object_keys_are_uuid_only() {
        let item_id = Uuid::parse_str("c5f41123-9884-4e0c-bd3d-87ce15612c42").unwrap();
        let image_id = Uuid::parse_str("1c5bf127-0e52-4aad-8c5d-51f10f9806f6").unwrap();
        let key = build_original_object_key(item_id, image_id);

        assert_eq!(
            key,
            "originals/c5f41123-9884-4e0c-bd3d-87ce15612c42/1c5bf127-0e52-4aad-8c5d-51f10f9806f6"
        );
        assert!(!key.contains('.'));
        assert!(!key.contains(' '));
        assert!(!key.contains("secret bucket photo.jpg"));
    }
}
