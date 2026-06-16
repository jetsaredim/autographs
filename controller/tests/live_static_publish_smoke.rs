#[cfg(feature = "live-persistence")]
mod live {
    use std::{env, io::Cursor, process::Command};

    use autographs_controller::{
        contracts::{PublicCatalog, PublicFacets, PublicItemDetail},
        media::PrivateMediaStore,
        oci_media::OciInstancePrincipalMediaStore,
        storage_keys::build_original_object_key,
    };
    use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
    use oracle::Connection;
    use serde_json::{Value, json};
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    #[tokio::test]
    #[ignore = "requires deployed controller, Caddy preview, Oracle wallet, and OCI instance-principal media access"]
    async fn live_static_publish_smoke_proves_seed_to_static_runtime() {
        if env::var("AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE").as_deref() != Ok("true") {
            println!(
                "skipping live static publish smoke: AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE is not true"
            );
            return;
        }

        let controller = required("AUTOGRAPHS_CONTROLLER_BASE_URL");
        let preview = required("AUTOGRAPHS_STATIC_PREVIEW_BASE_URL");
        let operator_token = required("AUTOGRAPHS_OPERATOR_API_TOKEN");
        let _static_release_root = required("AUTOGRAPHS_STATIC_RELEASE_ROOT");
        let oracle_user = required("ORACLE_DB_USER");
        let oracle_password = required("ORACLE_DB_PASSWORD");
        let oracle_connect_string = required("ORACLE_DB_CONNECT_STRING");
        let storage_namespace = required("OCI_MEDIA_NAMESPACE");
        let bucket_name = required("OCI_MEDIA_BUCKET_NAME");

        let connection =
            Connection::connect(&oracle_user, &oracle_password, &oracle_connect_string)
                .expect("connect to Oracle Autonomous Database");
        let media = OciInstancePrincipalMediaStore::new(storage_namespace, bucket_name)
            .expect("configure OCI instance-principal media store");

        let marker = Uuid::new_v4().simple().to_string();
        let title = format!("Live Static Smoke {marker}");
        let slug = format!("live-static-smoke-{marker}");
        let category = format!("Live Smoke Category {marker}");
        let tag = format!("live-smoke-tag-{marker}");
        let create_body = json!({
            "title": title,
            "signer": "Live Static Smoke Signer",
            "description": "Temporary Phase 5 live static publish proof",
            "category": category,
            "tags": [tag],
            "publicationStatus": "draft"
        })
        .to_string();
        let created: Value = json_request(
            "POST",
            &format!("{controller}/admin/api/items"),
            &operator_token,
            Some(&create_body),
        );
        let item_id = uuid_field(&created, "id");

        let image_body = png_fixture();
        let mut upload = NamedTempFile::new().expect("create temporary live smoke image");
        std::io::Write::write_all(&mut upload, &image_body).expect("write live smoke image");
        let uploaded = curl_json(
            vec![
                "--request".to_owned(),
                "POST".to_owned(),
                "--header".to_owned(),
                format!("Authorization: Bearer {operator_token}"),
                "--form".to_owned(),
                format!("image=@{};type=image/png", upload.path().display()),
                "--form".to_owned(),
                "altText=Temporary Phase 5 static smoke image".to_owned(),
                format!("{controller}/admin/api/items/{item_id}/images"),
            ],
            "upload live smoke image",
        );
        let image_id = Uuid::parse_str(
            uploaded["images"][0]["id"]
                .as_str()
                .expect("uploaded image id"),
        )
        .expect("parse uploaded image id");
        let object_key = build_original_object_key(item_id, image_id);
        let _cleanup = LiveStaticSmokeCleanup {
            connection: &connection,
            media: media.clone(),
            item_id: item_id.to_string(),
            object_key: object_key.clone(),
            controller: controller.clone(),
            operator_token: operator_token.clone(),
        };
        println!("live static smoke item id: {item_id}");
        println!("live static smoke object key: {object_key}");

        assert_oracle_image(&connection, item_id, image_id, &object_key);
        let stored = media
            .read(&object_key)
            .await
            .expect("read live static smoke original from OCI Object Storage");
        assert_eq!(stored, image_body);

        json_request(
            "POST",
            &format!("{controller}/admin/api/items/{item_id}/publication"),
            &operator_token,
            Some(r#"{"publicationStatus":"published"}"#),
        );
        let published = json_request(
            "POST",
            &format!("{controller}/admin/api/publish/full"),
            &operator_token,
            None,
        );
        assert_eq!(published["state"], "succeeded");
        assert!(published["releaseId"].as_str().is_some());

        let item_html = fetch(&format!("{preview}/items/{slug}/"));
        let item_json = fetch(&format!("{preview}/data/items/{slug}.json"));
        let collection_html = fetch(&format!("{preview}/collection/"));
        let collection_json = fetch(&format!("{preview}/data/collection.json"));
        let facets_json = fetch(&format!("{preview}/data/facets.json"));
        let thumbnail_url = format!("{preview}/media/{slug}/image-1-thumbnail.webp");
        let detail_url = format!("{preview}/media/{slug}/image-1-detail.webp");
        let thumbnail = fetch_bytes(&thumbnail_url);
        let detail = fetch_bytes(&detail_url);

        let public_item: PublicItemDetail =
            serde_json::from_str(&item_json).expect("decode generated item JSON");
        let catalog: PublicCatalog =
            serde_json::from_str(&collection_json).expect("decode generated collection JSON");
        let facets: PublicFacets =
            serde_json::from_str(&facets_json).expect("decode generated facets JSON");
        assert_eq!(public_item.slug, slug);
        assert_eq!(image::guess_format(&thumbnail).unwrap(), ImageFormat::WebP);
        assert_eq!(image::guess_format(&detail).unwrap(), ImageFormat::WebP);
        assert!(item_html.contains(&title));
        assert!(collection_html.contains("Collection"));
        assert!(facets.groups.iter().any(|group| {
            group
                .options
                .iter()
                .any(|option| option.value == category || option.value == tag)
        }));

        let matches = catalog
            .items
            .iter()
            .filter(|item| item.category == category && item.tags.contains(&tag))
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].slug, slug);
        assert!(
            catalog
                .items
                .iter()
                .filter(|item| item.category == "not-the-smoke-category")
                .all(|item| item.slug != slug)
        );
        assert!(
            catalog
                .items
                .iter()
                .filter(|item| item.tags.contains(&"not-the-smoke-tag".to_owned()))
                .all(|item| item.slug != slug)
        );
        scan_public_text(
            &[
                &item_html,
                &item_json,
                &collection_html,
                &collection_json,
                &facets_json,
                &thumbnail_url,
                &detail_url,
            ],
            image_id,
        );

        json_request(
            "POST",
            &format!("{controller}/admin/api/items/{item_id}/publication"),
            &operator_token,
            Some(r#"{"publicationStatus":"draft"}"#),
        );
        let unpublished = json_request(
            "POST",
            &format!("{controller}/admin/api/publish/incremental"),
            &operator_token,
            None,
        );
        assert_eq!(unpublished["state"], "succeeded");
        for url in [
            format!("{preview}/items/{slug}/"),
            format!("{preview}/data/items/{slug}.json"),
            thumbnail_url,
            detail_url,
        ] {
            assert_eq!(status(&url), 404, "stale public artifact remained: {url}");
        }
    }

    struct LiveStaticSmokeCleanup<'a> {
        connection: &'a Connection,
        media: OciInstancePrincipalMediaStore,
        item_id: String,
        object_key: String,
        controller: String,
        operator_token: String,
    }

    impl Drop for LiveStaticSmokeCleanup<'_> {
        fn drop(&mut self) {
            let publication_drafted = best_effort_json_request(
                "POST",
                &format!(
                    "{}/admin/api/items/{}/publication",
                    self.controller, self.item_id
                ),
                &self.operator_token,
                Some(r#"{"publicationStatus":"draft"}"#),
            )
            .is_some();
            let static_cleanup_succeeded = if publication_drafted {
                best_effort_json_request(
                    "POST",
                    &format!("{}/admin/api/publish/incremental", self.controller),
                    &self.operator_token,
                    None,
                )
                .is_some_and(|unpublished| unpublished["state"] == "succeeded")
            } else {
                false
            };
            if !static_cleanup_succeeded {
                eprintln!(
                    "live static smoke could not confirm stale public artifact cleanup for item {}",
                    self.item_id
                );
            }
            std::thread::scope(|scope| {
                let media = self.media.clone();
                let object_key = self.object_key.clone();
                let _ = scope
                    .spawn(move || {
                        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        else {
                            return;
                        };
                        let _ = runtime.block_on(media.delete(&object_key));
                    })
                    .join();
            });
            let _ = self.connection.execute(
                "delete from autograph_images where item_id = :1",
                &[&self.item_id],
            );
            let _ = self.connection.execute(
                "delete from autograph_items where id = :1",
                &[&self.item_id],
            );
            let _ = self.connection.commit();
        }
    }

    fn json_request(method: &str, url: &str, token: &str, body: Option<&str>) -> Value {
        let args = json_request_args(method, url, token, body);
        curl_json(args, &format!("{method} {url}"))
    }

    fn best_effort_json_request(
        method: &str,
        url: &str,
        token: &str,
        body: Option<&str>,
    ) -> Option<Value> {
        let output = Command::new("curl")
            .args(["--fail-with-body", "--silent", "--show-error"])
            .args(json_request_args(method, url, token, body))
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        serde_json::from_slice(&output.stdout).ok()
    }

    fn json_request_args(method: &str, url: &str, token: &str, body: Option<&str>) -> Vec<String> {
        let mut args = vec![
            "--request".to_owned(),
            method.to_owned(),
            "--header".to_owned(),
            format!("Authorization: Bearer {token}"),
        ];
        if let Some(body) = body {
            args.extend([
                "--header".to_owned(),
                "Content-Type: application/json".to_owned(),
                "--data".to_owned(),
                body.to_owned(),
            ]);
        }
        args.push(url.to_owned());
        args
    }

    fn curl_json(args: Vec<String>, context: &str) -> Value {
        serde_json::from_str(&curl(args, context)).unwrap_or_else(|error| {
            panic!("decode JSON response for {context}: {error}");
        })
    }

    fn fetch(url: &str) -> String {
        curl(vec![url.to_owned()], &format!("fetch {url}"))
    }

    fn fetch_bytes(url: &str) -> Vec<u8> {
        let output = Command::new("curl")
            .args(["--fail-with-body", "--silent", "--show-error"])
            .arg(url)
            .output()
            .unwrap_or_else(|error| panic!("run curl for {url}: {error}"));
        assert!(
            output.status.success(),
            "fetch {url}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    }

    fn status(url: &str) -> u16 {
        let output = Command::new("curl")
            .args([
                "--silent",
                "--output",
                "/dev/null",
                "--write-out",
                "%{http_code}",
            ])
            .arg(url)
            .output()
            .unwrap_or_else(|error| panic!("read status for {url}: {error}"));
        assert!(output.status.success(), "read status for {url}");
        String::from_utf8(output.stdout)
            .expect("status response is UTF-8")
            .parse()
            .expect("parse HTTP status")
    }

    fn curl(args: Vec<String>, context: &str) -> String {
        let output = Command::new("curl")
            .args(["--fail-with-body", "--silent", "--show-error"])
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("run curl for {context}: {error}"));
        assert!(
            output.status.success(),
            "{context}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("curl response is UTF-8")
    }

    fn assert_oracle_image(
        connection: &Connection,
        item_id: Uuid,
        image_id: Uuid,
        object_key: &str,
    ) {
        let stored_key: String = connection
            .query_row_as(
                "select object_key from autograph_images where id = :1 and item_id = :2",
                &[&image_id.to_string(), &item_id.to_string()],
            )
            .expect("read live static smoke image metadata from Oracle");
        assert_eq!(stored_key, object_key);
    }

    fn scan_public_text(values: &[&str], image_id: Uuid) {
        let private_image_id = image_id.to_string();
        for value in values {
            for denied in [
                "storageNamespace",
                "bucketName",
                "objectKey",
                "https://objectstorage",
                "objectstorage",
                &private_image_id,
            ] {
                assert!(!value.contains(denied), "public output leaked {denied}");
            }
        }
    }

    fn png_fixture() -> Vec<u8> {
        let mut body = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(RgbImage::from_pixel(16, 16, Rgb([21, 92, 126])))
            .write_to(&mut body, ImageFormat::Png)
            .expect("encode live smoke PNG");
        body.into_inner()
    }

    fn uuid_field(value: &Value, name: &str) -> Uuid {
        Uuid::parse_str(
            value[name]
                .as_str()
                .unwrap_or_else(|| panic!("{name} is required")),
        )
        .unwrap_or_else(|error| panic!("parse {name}: {error}"))
    }

    fn required(name: &str) -> String {
        env::var(name)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| panic!("{name} is required for the live static publish smoke"))
    }
}

#[cfg(not(feature = "live-persistence"))]
#[test]
#[ignore = "compile with --features live-persistence and supply live runtime credentials"]
fn live_static_publish_smoke_requires_explicit_feature() {
    println!("skipping live static publish smoke: compile with --features live-persistence");
}
