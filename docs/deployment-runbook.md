# Deployment Runbook

This runbook gets the project from a clean checkout to an OCI VM running systemd-managed Podman quadlets: public Caddy serving the generated static release, the Rust private controller for admin and publishing, and shared static release storage on a dedicated Podman network, with Terraform-managed hooks for Oracle Autonomous Database Free and private OCI Object Storage media.

## Preconditions

- OCI tenancy exists.
- An OCI user or deploy identity has API signing keys for the current deploy path.
- The tenancy bootstrap root has created or imported the project compartment, state bucket, deploy user, groups, and policies.
- The runtime VM image OCID, availability domain, and SSH public key are known.
- The target VM accepts SSH for the deploy user. Podman, firewalld, swap, service masking, secrets, and quadlets are managed by the merge-triggered Ansible deploy.
- GitHub repo-level GitHub Secrets and GitHub Variables from [configuration-contract.md](configuration-contract.md) are populated.
- Oracle Autonomous Database and Object Storage creation toggles are set intentionally before enabling data services.
- Optional GitHub Environments may be configured for approval gates, but they are not required for the baseline path.

## Local Validation

Run the same broad checks used by GitHub Actions:

```bash
cargo fmt --manifest-path controller/Cargo.toml --check
cargo test --manifest-path controller/Cargo.toml
cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings
cargo check --manifest-path controller/Cargo.toml --features production-persistence
terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff
```

To validate the committed runtime deployment shape:

```bash
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/system-cleanup.yml
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/
```

Those commands run the same Rust, Terraform, and Ansible checks used by CI for the controller/static runtime.

## OCI Bootstrap

1. Follow [oci-bootstrap.md](oci-bootstrap.md) to prepare the tenancy, compartment path, and Terraform state bucket.
2. This tenancy is codified as `us-ashburn-1` for both the OCI home region and runtime region.
3. Run the tenancy bootstrap root locally with an administrative/operator identity.
4. The deploy workflow codifies the Terraform state bucket as `autographs-tf-state` and the runtime state object key as `envs/prod/terraform.tfstate`.
5. If resources were created manually, import them using [imports.md](../infra/terraform/bootstrap/imports.md).
6. Run runtime Terraform locally once if needed to prove the baseline and obtain outputs.
7. Follow [dns-runbook.md](dns-runbook.md) when enabling the public app hostname through Porkbun.

Important operator inputs:

- `OCI_COMPARTMENT_OCID`
- `OCI_AVAILABILITY_DOMAIN`
- `OCI_RUNTIME_IMAGE_OCID`
- `OCI_RUNTIME_SSH_PUBLIC_KEYS`
- `OCI_OBJECT_STORAGE_NAMESPACE`
- `OCI_CREATE_AUTONOMOUS_DATABASE`
- `OCI_AUTONOMOUS_DATABASE_NAME`
- `OCI_CREATE_MEDIA_BUCKET`
- `OCI_MEDIA_BUCKET_NAME`
- `OCI_MEDIA_NAMESPACE`
- `VM_PUBLIC_IP` when not relying on Terraform output

## GitHub Configuration

Populate repo-level GitHub Secrets:

- `OCI_CLI_USER_OCID`
- `OCI_TENANCY_OCID`
- `OCI_FINGERPRINT`
- `OCI_PRIVATE_KEY_PEM`
- `DEPLOY_SSH_PRIVATE_KEY`
- `ADB_ADMIN_PASSWORD`
- `ORACLE_DB_PASSWORD`
- `ORACLE_DB_WALLET_ZIP_BASE64` when using an mTLS wallet
- `ORACLE_DB_WALLET_PASSWORD` when using the mTLS wallet path
- `AUTOGRAPHS_OPERATOR_API_TOKEN`
- `AUTOGRAPHS_ADMIN_PASSWORD_HASH`

Populate repo-level GitHub Variables:

- `OCI_COMPARTMENT_OCID`
- `OCI_AVAILABILITY_DOMAIN`
- `OCI_RUNTIME_IMAGE_OCID`
- `OCI_RUNTIME_SHAPE`
- `OCI_RUNTIME_OCPUS`
- `OCI_RUNTIME_MEMORY_GBS`
- `OCI_RUNTIME_SSH_PUBLIC_KEYS`
- `OCI_OBJECT_STORAGE_NAMESPACE`
- `OCI_CREATE_AUTONOMOUS_DATABASE`
- `OCI_AUTONOMOUS_DATABASE_NAME`
- `OCI_AUTONOMOUS_DATABASE_DISPLAY_NAME`
- `OCI_CREATE_MEDIA_BUCKET`
- `OCI_MEDIA_BUCKET_NAME`
- `OCI_MEDIA_NAMESPACE`
- `ORACLE_DB_USER`
- `ORACLE_DB_CONNECT_STRING`
- `ORACLE_DB_WALLET_DIR` when using an mTLS wallet instead of walletless TLS
- `VM_PUBLIC_IP`
- `DEPLOY_SSH_USER`
- `DEPLOY_PATH`
- `DEPLOY_SSH_READY_TIMEOUT_SECONDS`
- `DEPLOY_SSH_READY_INTERVAL_SECONDS`
- `GHCR_CONTROLLER_IMAGE_REPOSITORY`
- `GHCR_CLEANUP_RETAIN_TAGGED`
- `GHCR_CLEANUP_RETAIN_SEMVER_TAGGED`
- `GHCR_CLEANUP_MIN_AGE_DAYS`
- `GHCR_CLEANUP_PROTECTED_TAGS`
- `AUTOGRAPHS_LOCAL_IMAGE_RETAIN_COUNT`
- `AUTOGRAPHS_DOMAIN`
- `AUTOGRAPHS_CONTROLLER_DB_PROVIDER`
- `AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER`

`GHCR_CONTROLLER_IMAGE_REPOSITORY` should be the exact `ghcr.io` image path used for the controller image, such as `ghcr.io/jetsaredim/autographs/controller`. The deployed runtime no longer publishes, pulls, or starts the old Next.js runner or tools images.

`OCI_RUNTIME_SHAPE`, `OCI_RUNTIME_OCPUS`, `OCI_RUNTIME_MEMORY_GBS`, `VM_PUBLIC_IP`, `DEPLOY_SSH_USER`, `DEPLOY_PATH`, `DEPLOY_SSH_READY_TIMEOUT_SECONDS`, `DEPLOY_SSH_READY_INTERVAL_SECONDS`, `GHCR_CONTROLLER_IMAGE_REPOSITORY`, image cleanup settings, and `AUTOGRAPHS_DOMAIN` have workflow defaults or fallbacks. The OCPU and memory inputs are used only for `.Flex` shapes; fixed shapes such as `VM.Standard.E2.1.Micro` omit the Terraform `shape_config` block. The availability domain, runtime image OCID, SSH public keys, and Object Storage namespace are tenancy-specific and should be set explicitly.

Leave `OCI_CREATE_AUTONOMOUS_DATABASE` and `OCI_CREATE_MEDIA_BUCKET` as `false` until the tenancy-specific namespace, ADMIN password, and runtime connection values are ready. When enabling data services, Terraform provisions the ADB and bucket, while the deploy step passes runtime coordinates through VM-local quadlet environment files.

For the initial production path, use the ADB wallet-based mTLS connection. Set `OCI_AUTONOMOUS_DATABASE_IS_MTLS_CONNECTION_REQUIRED=true`, set `ORACLE_DB_CONNECT_STRING` to a wallet alias such as `autographsdb_medium`, set `ORACLE_DB_WALLET_DIR=/opt/autographs/wallet`, and store the base64-encoded wallet zip in the `ORACLE_DB_WALLET_ZIP_BASE64` GitHub Secret. Store the wallet download password in `ORACLE_DB_WALLET_PASSWORD`; the pure-Rust `oracledb` driver uses it to decrypt `ewallet.pem` even though the wallet has already been unpacked. The deploy workflow unpacks that wallet onto the VM and mounts it read-only into the Rust controller container.

Runtime secret values can be moved to OCI Vault after the corresponding secret
contents are populated outside Terraform. The tenancy root creates the runtime
secret shells and generates the runtime dynamic-group policy from those secret
OCIDs, so the controller can read only the Terraform-managed Autographs runtime
secret bundles. Use `terraform -chdir=infra/terraform/tenancy output
runtime_secret_id_env_vars` to get the GitHub repository variable name for each
runtime secret OCID, then set the matching repository variables:

- `ORACLE_DB_PASSWORD_VAULT_SECRET_ID`
- `ORACLE_DB_WALLET_PASSWORD_VAULT_SECRET_ID`
- `AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID`
- `AUTOGRAPHS_OPERATOR_API_TOKEN_VAULT_SECRET_ID`

The controller resolves these OCIDs with runtime instance-principal
authentication during startup and then uses the existing runtime configuration
names in memory. Do not put the secret contents themselves in Terraform inputs
or repository variables. Rotate or replace the OCI-generated placeholder secret
versions with the real runtime values before switching deploy to the Vault
secret IDs.

Production admin authentication is hash-only. Do not deploy
`AUTOGRAPHS_ADMIN_PASSWORD` and do not create a Vault secret for it. Keep the
actual login password in the operator's password manager, and rotate access by
generating a new Argon2 hash for `AUTOGRAPHS_ADMIN_PASSWORD_HASH`.

The OCI API signing key remains a GitHub Secret named `OCI_PRIVATE_KEY_PEM` for Terraform and deployment control-plane operations. Runtime deploy does not copy it to the VM or mount it into the Rust controller. The controller's OCI media adapter uses the VM instance principal instead.

## Data and Media Smoke

Production verification uses the deployed Rust controller health route, static release manifest, and live static publish smoke from [static-runtime-runbook.md](static-runtime-runbook.md).

The deployed Rust controller exposes `GET /admin/api/health` for runtime readiness. A successful full or incremental publish verifies Oracle catalog access, private media access, generated derivatives, static release promotion, and Caddy static serving.

Published images are served as generated static derivatives from the promoted release. Public URLs contain generated artifact paths only; they do not expose Object Storage bucket credentials, signed direct URLs, or raw object keys as the public access contract.

Current operator work uses the static admin shell and Rust controller under `/admin` and `/admin/api/*`.

### Phase 8 CDN/cache verification probes

The Phase 8 CDN/cache contract and production probe list live in
[cdn-cache-contract.md](cdn-cache-contract.md). Do not enable production CDN
proxying until adjusted-media cache behavior is proven: an adjusted image must
publish new derivative bytes and a new fingerprinted `/media/*` URL without a
manual purge.

Use the contract probes after deploy and again when production CDN proxying is
enabled:

```bash
curl -I "https://${AUTOGRAPHS_DOMAIN}/admin/"
curl -I "https://${AUTOGRAPHS_DOMAIN}/admin/api/status"
curl -I "https://${AUTOGRAPHS_DOMAIN}/data/collection.json"
curl -I "https://${AUTOGRAPHS_DOMAIN}/media/...webp"
curl -I "https://${AUTOGRAPHS_DOMAIN}/media/...webp"
```

Inspect `Cache-Control` on every response and `CF-Cache-Status` on the repeated
media probes after Cloudflare is proxying the hostname.

## Oracle Schema Updates

The committed [`controller/db/schema.sql`](../controller/db/schema.sql) is the
canonical end state for fresh or recovered databases. Existing production
schemas should be advanced with the one-shot scripts under
[`controller/db/updates/`](../controller/db/updates/) before deploying the
controller image that depends on the new shape.

For Phase 06-03 media cleanup, run
[`controller/db/updates/06-03-media-cleanup.sql`](../controller/db/updates/06-03-media-cleanup.sql)
against the live Oracle catalog schema before merging or manually deploying the
updated controller. The script adds `autograph_cleanup_events`, ensures each
cleanup event has the private internal `target_object_key` needed for exact
retry cleanup, creates `autograph_cleanup_events_item_status_idx`, and replaces
`autograph_edit_events_type_ck` so edit history can record `cleanupChanged`.
After the script succeeds, deploy normally and verify `/admin/api/health`.
If an earlier version of this update created cleanup-warning rows before
`target_object_key` existed, the script backfills only targets that can be
proven from current image metadata and fails closed for unresolved legacy rows;
manually resolve or remove those warnings, then rerun the script.

With SQLcl or SQL*Plus configured for the same ADB wallet alias as the deployed
controller, the manual shape is:

```bash
export TNS_ADMIN="/path/to/adb-wallet"
sqlplus "${ORACLE_DB_USER}/${ORACLE_DB_PASSWORD}@${ORACLE_DB_CONNECT_STRING}" \
  @controller/db/updates/06-03-media-cleanup.sql
```

Then verify the schema shape before deploying the new controller:

```sql
select table_name from user_tables
 where table_name = 'AUTOGRAPH_CLEANUP_EVENTS';

select column_name
  from user_tab_columns
 where table_name = 'AUTOGRAPH_CLEANUP_EVENTS'
   and column_name = 'TARGET_OBJECT_KEY';

select constraint_name, search_condition_vc
  from user_constraints
 where table_name = 'AUTOGRAPH_EDIT_EVENTS'
   and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_CK';
```

For the Phase 7 schema/taxonomy migration, apply the additive schema update and
reviewed taxonomy backfill before relying on the schema version 2 admin/public
code. Generate and review the migration report and PL/SQL from
`.planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json`,
optionally run the reviewed PL/SQL manually through SQL Developer, then deploy
the controller/admin/public code. A full rebuild is required after the
schema/taxonomy migration so the promoted static release rewrites
`collection.json`, `facets.json`, item detail JSON, and item pages with
schemaVersion 2 taxonomy data.

## Workflow Behavior

Pull requests run `.github/workflows/ci.yml`. The CI workflow checks the Rust controller, builds the controller image without pushing it, validates Terraform, and runs Ansible syntax/lint checks for the quadlet deployment.

Merges to `main` run `.github/workflows/deploy.yml`. The deploy workflow first
classifies the merged PR and updates the repo semver in `VERSION`,
`.release-status.json`, and the generated README release status block, then
creates the matching Git `v*` tag on the release source commit when it is
present on `origin/main`. If the source commit cannot be found on `origin/main`,
the workflow falls back to tagging the release-status commit that records the
version. This keeps GitHub release-note summaries focused on the actual merged
change instead of the bot-authored release-status chore. Version bumps are
semantic: explicit breaking changes bump major, feature signals such as
`version:minor` or `feat:` bump minor, and everything else defaults to patch.
Each push-triggered workflow records the triggering merge SHA as the release
source revision; if another merge has already advanced `main`, the workflow
still publishes its version event on the current `main` tip so every merge gets
a monotonically increasing repo version.

For controller image changes, the initial release status records the new repo
version while keeping the previously deployed controller version until the VM
deployment and health checks succeed. A successful controller deploy commits a
follow-up status update that marks the semver controller image as deployed.
Those bot-authored release-status commits are ignored by an explicit
commit/path detector rather than by broad `[skip ci]` handling. If build or
deploy fails, the README/status block correctly remains `repo ahead of deployed
controller`.

The workflow deploys only when changed paths require it:

- Controller image changes build and publish
  `GHCR_CONTROLLER_IMAGE_REPOSITORY:vX.Y.Z`, then deploy that semver image.
- Terraform, Ansible, or deploy wiring changes without controller image changes
  redeploy the last recorded controller semver image.
- Repo-only changes update release status and Git tags without rebuilding or
  redeploying the runtime.

Manual dispatches reuse the current release status instead of creating a new
repo version; the force inputs are operational redeploy/reuse controls, not a
second versioning path or an overwrite of an existing semver image tag.

When deployment is required, the deploy workflow:

1. publishes or selects the semver-tagged Rust controller image in `ghcr.io`,
2. enters the serialized `deploy-production` section and refuses superseded
   deploys before touching production,
3. optionally taints and recreates the runtime VM when manually requested,
4. runs `terraform apply`,
5. connects to the OCI VM over SSH through Ansible,
6. installs/maintains Podman, firewalld, swap, and masked systemd services,
7. copies wallet and OCI API key material to protected VM paths,
8. installs systemd quadlets for the dedicated Podman network, private controller, shared static volume, and Caddy container,
9. stops, disables, and removes the retired Next.js app runtime if present,
10. pulls the published controller image and restarts the quadlet services,
11. checks the Caddy-fronted static release and Rust controller health routes,
12. records the deployed controller semver in `.release-status.json` and the
    README status block when a controller image deploy succeeds.

The VM pulls images built by GitHub Actions. The VM does not build application code or generate catalog content during deploy.

Deploys intentionally ship code, infrastructure configuration, Caddy wiring,
quadlet shape, and runtime secrets only. They do not generate or export catalog
content from GitHub-hosted runners. Catalog reads, private original access,
derivative generation, candidate validation, and static release promotion remain
inside the OCI/runtime boundary through the Rust controller and shared static
volume. After structural public-contract changes such as the Phase 7
schema/taxonomy migration, deploy the code/config shape first and then trigger a
runtime full rebuild through the private controller.

The retired Next.js source tree has been removed from the active repository. Public behavior now lives in `controller/static-public/` and the Rust publisher/controller code.

### Static Preview

On the VM, Caddy exposes the current generated release through a localhost-bound preview in addition to the public hostname:

```bash
curl --fail --silent http://127.0.0.1:8081/releases/<release-id>/collection/
```

The `/admin` shell and `/admin/api/*` controller proxy are available through the normal hostname. The public static cutover route shape is now the deployed Caddy shape:

- `/admin/api/*` reverse-proxies to `autographs-controller:8080` on the private Podman network.
- `/admin/*` serves the static admin shell from `/srv/autographs/static/current/admin`.
- `/api/operator/*` continues to return `404`.
- Anonymous public traffic serves `/srv/autographs/static/current` directly.
- `127.0.0.1:8081` on the VM maps to Caddy's static preview listener and serves `/srv/autographs/static`.

The deploy role stops and disables the retired `autographs-app.service`, removes its quadlet, and force-removes any leftover `autographs-app` Podman container.

The Ansible-managed `/opt/autographs/env/controller.env` intentionally uses persistent controller providers in deployment: `AUTOGRAPHS_CONTROLLER_DB_PROVIDER=oracle` and `AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER=oci-instance-principal`. The controller-specific environment also sets `OCI_AUTH_MODE=instance_principal` and relies on `OCI_MEDIA_NAMESPACE` plus `OCI_MEDIA_BUCKET_NAME` from the Ansible-managed runtime environment. Do not rely on a manual edit to `controller.env`; Ansible owns that file on each deploy. The deploy workflow verifies `/admin/api/health` reports those active provider modes before it succeeds.

The mandatory Phase 5 live static publish smoke from [static-runtime-runbook.md](static-runtime-runbook.md) is recorded in the Phase 5 closeout artifacts. Continue using that smoke path for future controller/publisher changes that need live Oracle, Object Storage, generated static output, and Caddy verification. The public hostname now serves the generated Rust/static runtime through Caddy. Recovery is roll-forward oriented: correct the source or controller, run a full rebuild, validate through the localhost candidate listener, and promote the repaired release.

The public Caddy route serves the same generated static release. `/api/catalog/*` and `/api/operator/*` are no longer part of the public runtime contract.

### Cache Headers

Caddy now sets explicit origin cache headers for the deployed static runtime:

- `/admin`, `/admin/*`, and `/admin/api/*`: `Cache-Control: no-store`.
- Generated public media under `/media/*`:
  `Cache-Control: public, max-age=86400`.
- Public assets such as `/assets/*`, `/favicon.ico`, and `/icon.png`:
  `Cache-Control: public, max-age=3600`.
- Public documents/data such as `/`, `/collection/*`, `/items/*`,
  `/architecture/*`, `/data/*`, `/manifest.json`, `index.html`, and `404.html`:
  `Cache-Control: public, max-age=60, must-revalidate`.

The media lifetime is intentionally longer than HTML/JSON because images are
large and generated derivative URLs include a 16-hex content fingerprint, for
example `/media/<item-slug>/<image-slug>-detail-<fingerprint>.webp`. Replacement,
rollback, and publish repair generate or restore the derivative filename that
matches the WebP bytes instead of reusing a stale public path. Keep CDN purge
access available for emergency takedown, accidental public exposure, or CDN
incident response, but routine image replacement should not need a purge.

The Ansible deploy role keeps `/.swapfile` at 2 GiB and writes `vm.swappiness=20` through `/etc/sysctl.d/99-autographs-swap.conf`. This is intentional for the Always Free runtime shape because controller publishing, image processing, and tools/smoke scripts can briefly exceed the VM's physical memory.

The role also installs `python3-oci-cli` from the Oracle Linux 10 Development Packages repo for operator diagnostics. The application does not depend on the OCI CLI, but keeping it on the VM lets an operator verify instance-principal Object Storage access independently from the Rust controller, including emergency listing or deletion of orphaned private media objects.

### Runtime VM Recreation

Terraform no longer embeds the runtime bootstrap state in cloud-init. If a clean VM is needed, manually run the deploy workflow with `recreate_runtime_instance=true`. The workflow taints `module.compute.oci_core_instance.runtime[0]` before `terraform apply`, forcing OCI to recreate the runtime VM and then letting Ansible converge the full production state onto the replacement instance.

Image cleanup runs separately through `.github/workflows/image-cleanup.yml` on a weekly schedule and by manual dispatch. One job prunes old VM-local controller images while keeping the active controller image from `${DEPLOY_PATH}/env/app.env`, `latest`, `GHCR_CLEANUP_PROTECTED_TAGS`, and the newest `AUTOGRAPHS_LOCAL_IMAGE_RETAIN_COUNT` matching images per repository. Another job prunes old GHCR controller package versions while keeping `latest`, protected tags, the deployed controller semver tag from `.release-status.json`, the newest `GHCR_CLEANUP_RETAIN_SEMVER_TAGGED` semver-tagged images, the newest `GHCR_CLEANUP_RETAIN_TAGGED` package versions, and versions newer than `GHCR_CLEANUP_MIN_AGE_DAYS`. Repo `v*` tags from non-image merges do not need matching GHCR image tags. Use the manual `dry_run=true` input to preview deletions.

### Post-Phase 6 Runtime Cleanup Checklist

Use this checklist after deploying Phase 6 optimization changes. These are
operator-run checks; the repository closeout only validates code and playbook
shape.

1. Preview VM-local image cleanup before deleting anything:

   ```bash
   ANSIBLE_LOCAL_TEMP=/tmp/ansible-local \
   ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote \
   ANSIBLE_CONFIG=deploy/ansible/ansible.cfg \
   ansible-playbook -i deploy/ansible/inventory/ci.ini \
     deploy/ansible/playbooks/system-cleanup.yml \
     --extra-vars autographs_system_cleanup_dry_run=true
   ```

2. Confirm the active controller image in `${DEPLOY_PATH}/env/app.env` is not in
   the removal set, then rerun without dry-run only if the preview is correct.
3. Confirm retired service absence on the VM:

   ```bash
   sudo systemctl status autographs-app.service || true
   sudo podman ps --all --filter name=autographs-app
   ```

   The service/container should be absent or inactive; do not recreate the
   retired Next.js runtime.
4. Verify static release retention from the controller publish status and VM
   filesystem:

   ```bash
   curl --fail --silent "https://${AUTOGRAPHS_DOMAIN}/admin/api/health"
   sudo find /var/lib/autographs/static/releases -maxdepth 1 -mindepth 1 -type d | wc -l
   sudo find /var/lib/autographs/static/failed -maxdepth 1 -mindepth 1 -type d | wc -l
   ```

   The expected defaults are five promoted releases and one failed candidate
   unless the Ansible-rendered retention variables were intentionally changed.
5. Verify Caddy route shape and cache headers:

   ```bash
   curl -I "https://${AUTOGRAPHS_DOMAIN}/media/<item-slug>/<image-slug>-detail-<fingerprint>.webp"
   curl -I "https://${AUTOGRAPHS_DOMAIN}/data/collection.json"
   curl -I "https://${AUTOGRAPHS_DOMAIN}/admin/"
   test "$(curl --silent --output /dev/null --write-out '%{http_code}' "https://${AUTOGRAPHS_DOMAIN}/api/operator/catalog")" = "404"
   ```

   Public media/assets should be cacheable, HTML/JSON should be short-lived,
   admin routes should be `no-store`, and retired operator paths should remain
   unavailable.

## Manual Smoke Path

After deployment, verify from your workstation:

```bash
curl --fail --silent "https://${AUTOGRAPHS_DOMAIN}/manifest.json"
curl --fail --silent "https://${AUTOGRAPHS_DOMAIN}/admin/api/health"
```

If this fails, check the VM:

```bash
ssh opc@"${VM_PUBLIC_IP}"
sudo systemctl status autographs-network.service autographs-controller.service autographs-caddy.service
sudo journalctl -u autographs-controller.service -u autographs-caddy.service --since "30 minutes ago"
sudo podman ps
```

## Current Auth Note

The current deploy path uses OCI API signing keys in GitHub Secrets. That is intentionally documented as replaceable auth, so a later OIDC or short-lived credential flow can replace the auth step without changing the GHCR controller image publication, Terraform execution, or VM deployment shape.
