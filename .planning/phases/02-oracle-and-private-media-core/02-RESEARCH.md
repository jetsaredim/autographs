# Phase 2 Research: Oracle and Private Media Core

**Researched:** 2026-05-14
**Scope:** Oracle Autonomous Database access from the Next.js app, private OCI Object Storage media access, and Terraform-managed OCI data resources.

## Recommended Direction

Use `node-oracledb` as the primary database driver and prefer Thin mode first. Oracle's current node-oracledb documentation states that Thin mode is the default and connects directly to Oracle Database, which avoids adding Oracle Instant Client to the existing app container unless a later feature forces Thick mode.

Use the OCI TypeScript/JavaScript SDK for private Object Storage access from server-side app code. Keep all object operations behind a server-only adapter so browser routes never receive storage credentials or public object URLs.

Extend Terraform with the official OCI provider resources for:

- `oci_database_autonomous_database` for the Autonomous Database target.
- `oci_objectstorage_bucket` for the private media bucket.

Keep live OCI credentials and wallet material in local environment/GitHub Secrets only. Commit only variable names, examples, docs, and non-sensitive resource coordinates.

## Implementation Implications

- **Container shape:** Thin-mode `node-oracledb` is the lowest-friction first choice for the existing Next.js container. If wallet/TLS behavior requires additional files, mount or inject wallet material through secrets rather than committing it.
- **Schema:** Model autograph records and image metadata relationally. Store storage namespace/bucket/object key and metadata server-side; do not store public object URLs as the media access contract.
- **Object reads:** Build a server-only media route that retrieves private objects and streams them with content-type/cache headers. This route becomes the stable image surface for Phase 3 gallery pages.
- **Terraform:** Keep ADB and the media bucket in the existing Terraform root/modules so `main` deployment remains the infrastructure path of record.
- **Verification:** CI should keep passing without production data secrets. Add optional live smoke scripts for Oracle/Object Storage when credentials are present.

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Oracle wallet handling complicates the Docker image | Start with Thin mode and runtime-injected wallet/config files only if required. |
| Live OCI resources are unavailable in CI | Separate compile/unit checks from optional live smoke checks. |
| App-mediated image delivery becomes memory-heavy | Prefer streaming object reads and document response size/cache behavior. |
| Mutation routes become public admin APIs too early | Keep mutation verification script-only or guarded until Phase 4 admin auth exists. |
| Terraform provider/resource semantics differ across versions | Use current official OCI provider docs and validate locally before live apply. |

## Sources

- Oracle node-oracledb initialization docs: https://node-oracledb.readthedocs.io/en/stable/user_guide/initialization.html
- Oracle node-oracledb connection handling docs: https://node-oracledb.readthedocs.io/en/latest/user_guide/connection_handling.html
- OCI TypeScript/JavaScript ObjectStorageClient docs: https://docs.oracle.com/en-us/iaas/tools/typescript/latest/classes/_objectstorage_lib_client_.objectstorageclient.html
- Terraform OCI `oci_database_autonomous_database` resource: https://registry.terraform.io/providers/oracle/oci/latest/docs/resources/database_autonomous_database
- Terraform OCI `oci_objectstorage_bucket` resource: https://registry.terraform.io/providers/oracle/oci/latest/docs/resources/objectstorage_bucket
