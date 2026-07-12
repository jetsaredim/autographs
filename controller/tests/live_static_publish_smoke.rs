#[cfg(feature = "live-persistence")]
mod live {
    use std::{env, io::Cursor, process::Command, time::Duration};

    use autographs_controller::{
        catalog::{CatalogRepository, PublicationStatus},
        contracts::{
            FacetId, ImageVariantName, PUBLIC_SCHEMA_VERSION, PublicCatalog, PublicFacets,
            PublicItemDetail,
        },
        media::PrivateMediaStore,
        oci_media::OciInstancePrincipalMediaStore,
        oracle_catalog::OracleCatalogRepository,
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
        println!(
            "live static smoke image version: {}",
            env::var("AUTOGRAPHS_SMOKE_IMAGE_VERSION").unwrap_or_else(|_| "unknown".to_owned())
        );

        let controller = required("AUTOGRAPHS_CONTROLLER_BASE_URL");
        let preview = required("AUTOGRAPHS_STATIC_PREVIEW_BASE_URL");
        let public_origin = public_origin();
        let admin_password = required("AUTOGRAPHS_ADMIN_PASSWORD");
        let admin_cookie = login_cookie(&controller, &admin_password);
        let _static_release_root = required("AUTOGRAPHS_STATIC_RELEASE_ROOT");
        let oracle_user = required("ORACLE_DB_USER");
        let oracle_password = required("ORACLE_DB_PASSWORD");
        let oracle_connect_string = required("ORACLE_DB_CONNECT_STRING");
        let storage_namespace = required("OCI_MEDIA_NAMESPACE");
        let bucket_name = required("OCI_MEDIA_BUCKET_NAME");

        let connection =
            Connection::connect(&oracle_user, &oracle_password, &oracle_connect_string)
                .expect("connect to Oracle Autonomous Database");
        let repository = OracleCatalogRepository::new(
            oracle_user.clone(),
            oracle_password.clone(),
            oracle_connect_string.clone(),
            storage_namespace.clone(),
            bucket_name.clone(),
        );
        let media =
            OciInstancePrincipalMediaStore::new(storage_namespace.clone(), bucket_name.clone())
                .expect("configure OCI instance-principal media store");

        let marker = Uuid::new_v4().simple().to_string();
        let title = format!("Live Static Smoke {marker}");
        let slug = format!("live-static-smoke-{marker}");
        let signer_name = format!("Live Static Smoke Signer {marker}");
        let signer_role = "actor";
        let format_name = "Trading Card";
        let legacy_category = format_name;
        let franchise = format!("Live Smoke Franchise {marker}");
        let product_line = format!("Live Smoke Product Line {marker}");
        let language = "Japanese";
        let origin = "Custom";
        let tag = format!("live-smoke-tag-{marker}");
        let create_body = json!({
            "title": title,
            "signer": signer_name,
            "description": "Temporary Phase 5 live static publish proof",
            "category": legacy_category,
            "signerCredits": [{
                "displayName": signer_name,
                "itemRole": signer_role
            }],
            "format": format_name,
            "origin": origin,
            "franchises": [franchise],
            "productLine": product_line,
            "language": language,
            "tags": [tag],
            "publicationStatus": "draft"
        })
        .to_string();
        let created: Value = json_request(
            "POST",
            &format!("{controller}/admin/api/items"),
            &public_origin,
            &admin_cookie,
            Some(&create_body),
        );
        let item_id = uuid_field(&created, "id");
        println!("live static smoke item id: {item_id}");
        let mut cleanup = LiveStaticSmokeCleanup {
            connection: &connection,
            item_id: item_id.to_string(),
            object_key: None,
            storage_namespace: storage_namespace.clone(),
            bucket_name: bucket_name.clone(),
            controller: controller.clone(),
            public_origin: public_origin.clone(),
            admin_cookie: admin_cookie.clone(),
            published: false,
        };

        let image_body = png_fixture();
        let mut upload = NamedTempFile::new().expect("create temporary live smoke image");
        std::io::Write::write_all(&mut upload, &image_body).expect("write live smoke image");
        let original_filename = upload
            .path()
            .file_name()
            .and_then(|name| name.to_str())
            .expect("temporary upload filename")
            .to_owned();
        let uploaded = curl_json(
            vec![
                "--request".to_owned(),
                "POST".to_owned(),
                "--header".to_owned(),
                format!("Cookie: {admin_cookie}"),
                "--header".to_owned(),
                format!("Origin: {public_origin}"),
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
        cleanup.object_key = Some(object_key.clone());
        println!("live static smoke object key: {object_key}");

        assert_oracle_image(&connection, item_id, image_id, &object_key);
        let stored = media
            .read(&object_key)
            .await
            .expect("read live static smoke original from OCI Object Storage");
        assert_eq!(stored, image_body);

        let publication = json_request(
            "POST",
            &format!("{controller}/admin/api/items/{item_id}/publication"),
            &public_origin,
            &admin_cookie,
            Some(r#"{"publicationStatus":"published"}"#),
        );
        assert_eq!(publication["publicationStatus"], "published");
        let published_item = repository
            .get(item_id)
            .await
            .expect("read live static smoke item after publication update")
            .expect("live static smoke item exists after publication update");
        assert_eq!(
            published_item.publication_status,
            PublicationStatus::Published
        );
        assert_eq!(published_item.images.len(), 1);
        cleanup.published = true;
        let published = json_request(
            "POST",
            &format!("{controller}/admin/api/publish/full"),
            &public_origin,
            &admin_cookie,
            None,
        );
        assert_eq!(published["state"], "succeeded");
        let release_id = published["releaseId"]
            .as_str()
            .expect("publish response includes release id");
        println!("live static smoke publish release id: {release_id}");

        let collection_html = fetch(&format!("{preview}/collection/"));
        let collection_json = fetch(&format!("{preview}/data/collection.json"));
        let facets_json = fetch(&format!("{preview}/data/facets.json"));

        let catalog: PublicCatalog =
            serde_json::from_str(&collection_json).expect("decode generated collection JSON");
        let facets: PublicFacets =
            serde_json::from_str(&facets_json).expect("decode generated facets JSON");
        assert_eq!(catalog.schema_version, PUBLIC_SCHEMA_VERSION);
        assert_eq!(facets.schema_version, PUBLIC_SCHEMA_VERSION);
        let facet_ids = facets
            .groups
            .iter()
            .map(|group| group.id)
            .collect::<Vec<_>>();
        for expected in [
            FacetId::Signer,
            FacetId::Franchise,
            FacetId::ProductLine,
            FacetId::Format,
            FacetId::Language,
            FacetId::Origin,
            FacetId::Role,
            FacetId::Tag,
        ] {
            assert!(
                facet_ids.contains(&expected),
                "generated facets missing {expected:?} in release {release_id}"
            );
        }
        assert!(
            !facets_json.contains(r#""id":"category""#),
            "schema version 2 facets must not include legacy category"
        );
        let matches = catalog
            .items
            .iter()
            .filter(|item| {
                item.title == title
                    && item.signer_names.contains(&signer_name)
                    && item.signer_roles.contains(&signer_role.to_owned())
                    && item.format == format_name
                    && item.origin == origin
                    && item.language == language
                    && item.franchises.contains(&franchise)
                    && item.product_line.as_deref() == Some(product_line.as_str())
                    && item.tags.contains(&tag)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "seeded item missing from generated collection in release {release_id}"
        );
        let generated_slug = matches[0].slug.clone();
        println!("live static smoke generated slug: {generated_slug}");
        assert_eq!(generated_slug, slug);

        let item_html = fetch(&format!("{preview}/items/{generated_slug}/"));
        let item_json = fetch(&format!("{preview}/data/items/{generated_slug}.json"));
        let public_item: PublicItemDetail =
            serde_json::from_str(&item_json).expect("decode generated item JSON");
        assert_eq!(public_item.schema_version, PUBLIC_SCHEMA_VERSION);
        assert_eq!(public_item.signer_names, vec![signer_name.clone()]);
        assert_eq!(public_item.signer_roles, vec![signer_role.to_owned()]);
        assert_eq!(public_item.format, format_name);
        assert_eq!(public_item.origin, origin);
        assert_eq!(public_item.language, language);
        assert_eq!(public_item.franchises, vec![franchise.clone()]);
        assert_eq!(
            public_item.product_line.as_deref(),
            Some(product_line.as_str())
        );
        assert!(public_item.tags.contains(&tag));
        let thumbnail_path = public_item.images[0]
            .variants
            .iter()
            .find(|variant| variant.name == ImageVariantName::Thumbnail)
            .expect("thumbnail variant")
            .path
            .clone();
        let detail_path = public_item.images[0]
            .variants
            .iter()
            .find(|variant| variant.name == ImageVariantName::Detail)
            .expect("detail variant")
            .path
            .clone();
        assert!(thumbnail_path.contains("-thumbnail-"));
        assert!(detail_path.contains("-detail-"));
        let thumbnail_url = format!("{preview}{thumbnail_path}");
        let detail_url = format!("{preview}{detail_path}");
        let thumbnail = fetch_bytes(&thumbnail_url);
        let detail = fetch_bytes(&detail_url);

        assert_eq!(public_item.slug, slug);
        assert_eq!(image::guess_format(&thumbnail).unwrap(), ImageFormat::WebP);
        assert_eq!(image::guess_format(&detail).unwrap(), ImageFormat::WebP);
        assert!(item_html.contains(&title));
        assert!(collection_html.contains("Collection"));
        assert_facet_contains(&facets, FacetId::Signer, &signer_name);
        assert_facet_contains(&facets, FacetId::Franchise, &franchise);
        assert_facet_contains(&facets, FacetId::ProductLine, &product_line);
        assert_facet_contains(&facets, FacetId::Format, format_name);
        assert_facet_contains(&facets, FacetId::Language, language);
        assert_facet_contains(&facets, FacetId::Origin, origin);
        assert_facet_contains(&facets, FacetId::Role, signer_role);
        assert_facet_contains(&facets, FacetId::Tag, &tag);

        assert!(
            catalog
                .items
                .iter()
                .filter(|item| item.format == "not-the-smoke-format")
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
            &object_key,
            &original_filename,
        );

        json_request(
            "POST",
            &format!("{controller}/admin/api/items/{item_id}/publication"),
            &public_origin,
            &admin_cookie,
            Some(r#"{"publicationStatus":"draft"}"#),
        );
        let unpublished = json_request(
            "POST",
            &format!("{controller}/admin/api/publish/incremental"),
            &public_origin,
            &admin_cookie,
            None,
        );
        assert_eq!(unpublished["state"], "succeeded");
        for url in [
            format!("{preview}/items/{generated_slug}/"),
            format!("{preview}/data/items/{generated_slug}.json"),
            thumbnail_url,
            detail_url,
        ] {
            assert_eq!(status(&url), 404, "stale public artifact remained: {url}");
        }
        delete_media_with_retries(&media, &object_key)
            .await
            .expect("delete live static smoke original from OCI Object Storage");
        cleanup.object_key = None;
    }

    struct LiveStaticSmokeCleanup<'a> {
        connection: &'a Connection,
        item_id: String,
        object_key: Option<String>,
        storage_namespace: String,
        bucket_name: String,
        controller: String,
        public_origin: String,
        admin_cookie: String,
        published: bool,
    }

    impl Drop for LiveStaticSmokeCleanup<'_> {
        fn drop(&mut self) {
            if self.published {
                eprintln!(
                    "live static smoke cleanup drafting and republishing item {}",
                    self.item_id
                );
                let publication_drafted = best_effort_json_request(
                    "POST",
                    &format!(
                        "{}/admin/api/items/{}/publication",
                        self.controller, self.item_id
                    ),
                    &self.public_origin,
                    &self.admin_cookie,
                    Some(r#"{"publicationStatus":"draft"}"#),
                )
                .is_some();
                let static_cleanup_succeeded = if publication_drafted {
                    best_effort_json_request(
                        "POST",
                        &format!("{}/admin/api/publish/incremental", self.controller),
                        &self.public_origin,
                        &self.admin_cookie,
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
                } else {
                    eprintln!(
                        "live static smoke cleanup republished without item {}",
                        self.item_id
                    );
                }
            } else {
                eprintln!(
                    "live static smoke cleaning unpublished temporary item {}",
                    self.item_id
                );
            }
            if let Some(object_key) = self.object_key.clone() {
                eprintln!("live static smoke cleanup deleting OCI object {object_key}");
                std::thread::scope(|scope| {
                    let storage_namespace = self.storage_namespace.clone();
                    let bucket_name = self.bucket_name.clone();
                    let _ = scope
                        .spawn(move || {
                            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                            else {
                                return;
                            };
                            let result = runtime.block_on(async {
                                let media = match OciInstancePrincipalMediaStore::new(
                                    storage_namespace,
                                    bucket_name,
                                ) {
                                    Ok(media) => media,
                                    Err(error) => {
                                        return Err(format!(
                                            "configure OCI media cleanup client: {error}"
                                        ));
                                    }
                                };
                                delete_media_with_retries(&media, &object_key).await
                            });
                            match result {
                                Ok(()) => {}
                                Err(error) => eprintln!(
                                    "live static smoke cleanup could not delete OCI object: {error}"
                                ),
                            }
                        })
                        .join();
                });
            }
            eprintln!(
                "live static smoke cleanup deleting Oracle rows for item {}",
                self.item_id
            );
            let _ = self.connection.execute(
                "delete from autograph_images where item_id = :1",
                &[&self.item_id],
            );
            let _ = self.connection.execute(
                "delete from autograph_items where id = :1",
                &[&self.item_id],
            );
            let _ = self.connection.commit();
            eprintln!(
                "live static smoke cleanup complete for item {}",
                self.item_id
            );
        }
    }

    async fn delete_media_with_retries(
        media: &OciInstancePrincipalMediaStore,
        object_key: &str,
    ) -> Result<(), String> {
        let delays = [
            Duration::ZERO,
            Duration::from_secs(2),
            Duration::from_secs(5),
        ];
        let mut last_error = None;
        for (attempt, delay) in delays.into_iter().enumerate() {
            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
            }
            match tokio::time::timeout(Duration::from_secs(75), media.delete(object_key)).await {
                Ok(Ok(())) => return Ok(()),
                Ok(Err(error)) => {
                    last_error = Some(error);
                }
                Err(_) => {
                    last_error = Some(format!(
                        "timed out deleting OCI object {object_key} on attempt {}",
                        attempt + 1
                    ));
                }
            }
        }
        Err(last_error.unwrap_or_else(|| format!("delete OCI object failed: {object_key}")))
    }

    fn json_request(
        method: &str,
        url: &str,
        origin: &str,
        cookie: &str,
        body: Option<&str>,
    ) -> Value {
        let args = json_request_args(method, url, origin, cookie, body);
        curl_json(args, &format!("{method} {url}"))
    }

    fn best_effort_json_request(
        method: &str,
        url: &str,
        origin: &str,
        cookie: &str,
        body: Option<&str>,
    ) -> Option<Value> {
        let output = Command::new("curl")
            .args([
                "--fail-with-body",
                "--silent",
                "--show-error",
                "--connect-timeout",
                "5",
                "--max-time",
                "60",
            ])
            .args(json_request_args(method, url, origin, cookie, body))
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        serde_json::from_slice(&output.stdout).ok()
    }

    fn json_request_args(
        method: &str,
        url: &str,
        origin: &str,
        cookie: &str,
        body: Option<&str>,
    ) -> Vec<String> {
        let mut args = vec![
            "--request".to_owned(),
            method.to_owned(),
            "--header".to_owned(),
            format!("Cookie: {cookie}"),
            "--header".to_owned(),
            format!("Origin: {origin}"),
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

    fn public_origin() -> String {
        env::var("AUTOGRAPHS_PUBLIC_ORIGIN")
            .ok()
            .map(|origin| origin.trim_end_matches('/').to_owned())
            .filter(|origin| !origin.is_empty())
            .unwrap_or_else(|| {
                let domain = required("AUTOGRAPHS_DOMAIN");
                let domain = domain.trim_end_matches('/');
                if domain.starts_with("http://") || domain.starts_with("https://") {
                    domain.to_owned()
                } else {
                    format!("https://{domain}")
                }
            })
    }

    fn login_cookie(controller: &str, password: &str) -> String {
        let body = json!({ "password": password }).to_string();
        let output = Command::new("curl")
            .args([
                "--fail-with-body",
                "--silent",
                "--show-error",
                "--include",
                "--connect-timeout",
                "5",
                "--max-time",
                "60",
                "--request",
                "POST",
                "--header",
                "Content-Type: application/json",
                "--data",
                &body,
            ])
            .arg(format!("{controller}/admin/api/login"))
            .output()
            .unwrap_or_else(|error| panic!("run curl for live smoke login: {error}"));
        assert!(
            output.status.success(),
            "live smoke login: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| {
                line.strip_prefix("set-cookie:")
                    .or_else(|| line.strip_prefix("Set-Cookie:"))
                    .and_then(|value| value.trim().split(';').next())
                    .map(str::to_owned)
            })
            .expect("live smoke login returned session cookie")
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
            .args([
                "--fail-with-body",
                "--silent",
                "--show-error",
                "--connect-timeout",
                "5",
                "--max-time",
                "60",
            ])
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
                "--connect-timeout",
                "5",
                "--max-time",
                "60",
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
            .args([
                "--fail-with-body",
                "--silent",
                "--show-error",
                "--connect-timeout",
                "5",
                "--max-time",
                "60",
            ])
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

    fn assert_facet_contains(facets: &PublicFacets, id: FacetId, value: &str) {
        let group = facets
            .groups
            .iter()
            .find(|group| group.id == id)
            .unwrap_or_else(|| panic!("missing facet group {id:?}"));
        assert!(
            group.options.iter().any(|option| option.value == value),
            "facet {id:?} missing option {value}"
        );
    }

    fn scan_public_text(
        values: &[&str],
        image_id: Uuid,
        object_key: &str,
        original_filename: &str,
    ) {
        let private_image_id = image_id.to_string();
        for value in values {
            for denied in [
                "storageNamespace",
                "bucketName",
                "objectKey",
                "https://objectstorage",
                "objectstorage",
                &private_image_id,
                object_key,
                original_filename,
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
