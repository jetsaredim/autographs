# Phase 05: Static Runtime Migration Foundation - Research

**Researched:** 2026-05-28
**Status:** Complete
**Scope:** Planning research for static public runtime generation, Rust private controller, OCI Object Storage derivatives, Caddy validation/cutover, and operator-bridge retirement.

## Executive Findings

Phase 5 should be planned as a foundation migration, not as a polished admin rebuild. The least risky path is to add a small Rust workspace alongside the existing Next.js app, prove Oracle/Object Storage access and static publishing in local/CI modes first, then wire the Rust controller and generated artifacts into Ansible/Caddy behind private validation routes before retiring the Node public/operator runtime.

The riskiest technical choices are Rust Oracle connectivity, OCI S3-compatible Object Storage behavior, derivative metadata stripping, and Caddy media serving. Plan these as early spikes with local-mode fallbacks and explicit acceptance tests before broader admin or cutover work.

## External Research Notes

### OCI Object Storage S3 Compatibility

Oracle documents an S3 Compatibility API for OCI Object Storage that can be used with existing S3 tools. Key constraints for planning:

- The endpoint shape is `https://{namespace}.compat.objectstorage.{region}.oraclecloud.com`.
- Path-style access is required; virtual-host style access is not supported.
- Customer Secret Keys are required for S3-compatible access.
- Bucket/object support includes `GetObject`, `HeadObject`, `PutObject`, `DeleteObject`, and multipart upload operations.
- Oracle notes Object Storage data is congruent across native and S3-compatible APIs, so objects written by one API can be read by the other.

Implication: using Rust `aws-sdk-s3` or a Caddy S3 proxy is plausible, but Phase 5 must force path-style addressing, custom endpoint configuration, and least-privilege Customer Secret Key handling. Do not assume AWS default host addressing works against OCI.

Sources:
- Oracle OCI Object Storage Amazon S3 Compatibility API: https://docs.public.oneportal.content.oci.oraclecloud.com/iaas/Content/Object/Tasks/s3compatibleapi.htm
- Oracle S3 Compatibility API support matrix: https://docs.public.content.oci.oraclecloud.com/en-us/iaas/Content/Object/Tasks/s3compatibleapi_topic-Amazon_S3_Compatibility_API_Support.htm
- AWS SDK for Rust endpoint configuration: https://docs.aws.amazon.com/sdk-for-rust/latest/dg/endpoints.html

### Caddy Static, Private API, and Media Routing

Caddy already serves the public edge in this repo. Official Caddy `reverse_proxy`, `handle_path`, and `file_server` patterns support a clean target shape:

- root static file serving for generated public HTML/JSON;
- `/admin/api/*` reverse proxy to the Rust controller;
- `/admin/*` static admin shell serving;
- a local-only candidate listener or route for validation before promotion;
- optional response interception or a dedicated controller endpoint if media proxying through Caddy plugins proves too fragile.

The candidate `github.com/lindenlab/caddy-s3-proxy` package exists, but it is not a core Caddy module and appears relatively small/old. Treat it as an optional spike, not a locked dependency. A safer Phase 5 fallback is: generate public derivatives into a local release tree during publish, serve them with `file_server`, and keep Object Storage as the durable source of private originals. If the S3 proxy is adopted, it needs maintenance, OCI path-style, cache/header, missing-object, and read-only-prefix verification.

Sources:
- Caddy reverse_proxy directive: https://caddyserver.com/docs/caddyfile/directives/reverse_proxy
- Caddy directives index: https://caddyserver.com/docs/caddyfile/directives
- lindenlab caddy-s3-proxy package: https://pkg.go.dev/github.com/lindenlab/caddy-s3-proxy

### Rust Controller and Oracle Access

Rust has two plausible Oracle paths:

- `oracle` crate: ODPI-C based, mature-looking, but requires Oracle client/ODPI-C runtime considerations.
- `oracle-rs` crate: pure Rust/Tokio driver with TLS/wallet claims, promising for small containers, but should be treated as a Phase 5 spike before committing.

Plan the first Rust database task as a narrow connectivity probe that can run against local/CI mocks and live ADB only when secrets are available. Do not build admin CRUD before one path proves it can connect, query, and preserve the existing schema semantics.

Sources:
- `oracle` crate docs: https://docs.rs/oracle/latest/oracle/
- `oracle-rs` crate docs: https://docs.rs/oracle-rs/latest/oracle_rs/

### Rust HTTP, Uploads, and Sessions

Axum is a good fit for a small Rust controller because it has stable extractors for multipart uploads, state, routing, and Tower middleware. The multipart extractor consumes the request body and must be last in handlers, which matters for upload endpoint design. A same-origin, secure, HTTP-only session cookie can be implemented in the controller without creating public accounts or multi-admin roles.

Source:
- Axum multipart extractor docs: https://docs.rs/axum/latest/axum/extract/struct.Multipart.html

### Image Derivatives and Metadata

The Rust `image` crate has orientation metadata support. Its docs explicitly warn that if Exif orientation is applied manually, orientation metadata must be cleared to avoid double-rotation by downstream software. Plan derivative generation to:

- decode original;
- normalize orientation;
- resize to thumbnail/detail sizes;
- strip metadata or encode fresh output without original metadata;
- write deterministic public-safe derivative paths;
- record byte sizes and checksums in a publish manifest.

`metastrip` is a candidate for metadata stripping, but Phase 5 should verify behavior with representative JPEG/PNG/HEIC inputs before depending on it. Avoid promising broad format support beyond what the selected crate path proves.

Sources:
- Rust `image::metadata::Orientation`: https://docs.rs/image/latest/image/metadata/enum.Orientation.html
- Rust `image::ImageDecoder`: https://docs.rs/image/latest/image/trait.ImageDecoder.html
- `metastrip` crate docs: https://docs.rs/metastrip

## Recommended Plan Shape

1. Define public artifact contracts, privacy rules, release layout, and local publisher fixtures before touching deployment.
2. Add a Rust workspace/controller skeleton with health, config, auth/session, and local-mode repository/media abstractions.
3. Prove Oracle/Object Storage and derivative-generation integration behind tests and optional live smoke.
4. Implement publisher candidate generation, validation, manifest scans, and atomic promotion.
5. Add the minimal static admin shell and seed/publish endpoints.
6. Wire Ansible/Caddy/CI/docs for private validation and cutover/retirement.

## Key Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Rust Oracle driver selection fails late | Controller cannot write source-of-truth data | Put DB connectivity and one read/write loop in the first Rust integration plan. Keep the old Node app untouched until the new path proves itself. |
| OCI S3-compatible path-style quirks break Object Storage operations | Upload/publish/media proxy fails in runtime | Force path-style endpoint config, add local-mode tests, and add a live smoke path gated by secrets. |
| Caddy S3 proxy is unmaintained or does not fit OCI | Public media path becomes fragile | Prefer generated local derivative files in release directories for Phase 5; keep S3 proxy as optional spike/future improvement. |
| Static output leaks private IDs or object keys | Violates core privacy requirement | Add deny-list scans over generated HTML/JSON/manifests and public media paths before promotion. |
| Admin auth is underbuilt because UI is "minimal" | Internet-reachable login becomes a real attack surface | Plan single-admin login, HTTP-only secure cookie, rate limiting/basic lockout, CSRF-conscious same-origin API design, redacted errors, and secret-store-only credentials. |
| Incremental publish creates incomplete releases | Broken public site after publish | Generate candidate release from current, validate full manifest/page/media consistency, then atomically promote. |

## Validation Architecture

Phase 5 validation should use three tiers:

- Local/CI tests: Rust unit/integration tests with local filesystem media, fixture catalog data, generated static output, leak scans, derivative existence checks, Caddyfile static assertions, and docs/runbook checks.
- Runtime smoke: optional credential-backed smoke that seeds one minimal item and one image through the Rust private API, triggers publish, validates candidate output locally, and verifies public static page/media after promotion.
- Cutover preflight: operator checklist for stopping old public Next.js serving, enabling static root/admin routes, confirming `/api/catalog/*` and `/api/operator/*` retirement behavior, and keeping a roll-forward full rebuild path.

## Planning Constraints

- Do not re-scaffold the existing app, infra, or delivery spine.
- Keep GitHub Actions out of catalog content generation.
- Keep private originals durable in Object Storage; generated derivatives are rebuildable artifacts.
- Keep Phase 5 admin UI functional and minimal; Phase 6 owns polished daily-use admin workflow, edit history, and media cleanup ergonomics.
- Keep public artifact contracts versioned and privacy-tested.

## RESEARCH COMPLETE
