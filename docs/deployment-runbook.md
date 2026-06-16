# Deployment Runbook

This runbook gets the app from a clean checkout to an OCI VM running
systemd-managed Podman quadlets: public `Caddy` serving the generated static
release, the Rust private controller for admin and publishing, and shared static
release storage on a dedicated Podman network, with Terraform-managed hooks for
Oracle Autonomous Database Free and private OCI Object Storage media.

## Preconditions

- OCI tenancy exists.
- An OCI user or deploy identity has API signing keys for Phase 1.
- The tenancy bootstrap root has created or imported the project compartment, state bucket, deploy user, groups, and policies.
- The runtime VM image OCID, availability domain, and SSH public key are known.
- The target VM accepts SSH for the deploy user. Podman, firewalld, swap, service masking, secrets, and quadlets are managed by the merge-triggered Ansible deploy.
- GitHub repo-level GitHub Secrets and GitHub Variables from [configuration-contract.md](configuration-contract.md) are populated.
- Oracle Autonomous Database and Object Storage creation toggles are set intentionally before enabling Phase 2 data services.
- Optional GitHub Environments may be configured for approval gates, but they are not required for the baseline path.

## Local Validation

Run the same broad checks used by GitHub Actions:

```bash
corepack pnpm install --frozen-lockfile
corepack pnpm --filter app lint
corepack pnpm --filter app typecheck
cargo test --manifest-path controller/Cargo.toml
cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings
terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff
```

To validate the committed runtime deployment shape:

```bash
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/
```

Those commands run the same Ansible syntax and lint checks used by CI for the quadlet deployment.

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
- `ORACLE_DB_WALLET_PASSWORD` when the node-oracledb Thin connection needs the downloaded wallet password
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
- `AUTOGRAPHS_MEDIA_STORAGE_PROVIDER`
- `VM_PUBLIC_IP`
- `DEPLOY_SSH_USER`
- `DEPLOY_PATH`
- `DEPLOY_SSH_READY_TIMEOUT_SECONDS`
- `DEPLOY_SSH_READY_INTERVAL_SECONDS`
- `GHCR_IMAGE_REPOSITORY`
- `GHCR_CLEANUP_RETAIN_TAGGED`
- `GHCR_CLEANUP_MIN_AGE_DAYS`
- `GHCR_CLEANUP_PROTECTED_TAGS`
- `AUTOGRAPHS_LOCAL_IMAGE_RETAIN_COUNT`
- `AUTOGRAPHS_DOMAIN`

`GHCR_IMAGE_REPOSITORY` should be the base `ghcr.io` image path used for the tools and controller images, such as `ghcr.io/jetsaredim/autographs/app`. The deployed runtime no longer publishes or starts the old Next.js runner image from that base path.

`OCI_RUNTIME_SHAPE`, `OCI_RUNTIME_OCPUS`, `OCI_RUNTIME_MEMORY_GBS`, `VM_PUBLIC_IP`, `DEPLOY_SSH_USER`, `DEPLOY_PATH`, `DEPLOY_SSH_READY_TIMEOUT_SECONDS`, `DEPLOY_SSH_READY_INTERVAL_SECONDS`, `GHCR_IMAGE_REPOSITORY`, image cleanup settings, and `AUTOGRAPHS_DOMAIN` have workflow defaults or fallbacks. The OCPU and memory inputs are used only for `.Flex` shapes; fixed shapes such as `VM.Standard.E2.1.Micro` omit the Terraform `shape_config` block. The availability domain, runtime image OCID, SSH public keys, and Object Storage namespace are tenancy-specific and should be set explicitly.

Leave `OCI_CREATE_AUTONOMOUS_DATABASE` and `OCI_CREATE_MEDIA_BUCKET` as `false` until the tenancy-specific namespace, ADMIN password, and runtime connection values are ready. When enabling Phase 2 data services, Terraform provisions the ADB and bucket, while the deploy step passes app runtime coordinates through the VM-local quadlet environment file.

For the initial production path, use the ADB wallet-based mTLS connection. Set `OCI_AUTONOMOUS_DATABASE_IS_MTLS_CONNECTION_REQUIRED=true`, set `ORACLE_DB_CONNECT_STRING` to a wallet alias such as `autographsdb_medium`, set `ORACLE_DB_WALLET_DIR=/opt/autographs/wallet`, and store the base64-encoded wallet zip in the `ORACLE_DB_WALLET_ZIP_BASE64` GitHub Secret. Also store the wallet download password in `ORACLE_DB_WALLET_PASSWORD` if the Thin driver requires it. The deploy workflow unpacks that wallet onto the VM and mounts it read-only into the Rust controller and tools containers.

The OCI API signing key remains a GitHub Secret named `OCI_PRIVATE_KEY_PEM`. Terraform uses it from the runner temp directory. Runtime deploy copies it to `${DEPLOY_PATH}/secrets/oci_api_key.pem`, mounts `${DEPLOY_PATH}/secrets` read-only into the Rust controller and tools containers, and sets `OCI_PRIVATE_KEY_PATH=/opt/autographs/secrets/oci_api_key.pem`. This preserves PEM newlines for the OCI SDK and avoids putting multiline private keys in the quadlet environment file.

## Data and Media Smoke

This is the pre-cutover Node runtime smoke. Keep it only as a manual diagnostic
while the tools image remains available; the deployed public runtime no longer
starts the Node app service.

Use the deeper VM smoke workflow only when data-service credentials are present:

Run `.github/workflows/data-smoke.yml` manually from GitHub Actions. The workflow resolves the runtime VM IP, starts the `production` tools image on the VM's Podman network by default, runs migrations, loads representative seed records with generated SVG fixture images, creates a published smoke item, uploads a private smoke image, reads it back through the catalog/media service, and verifies the deployed app-mediated image route with `AUTOGRAPHS_SMOKE_BASE_URL`. It is intentionally manual because it requires live ADB and private Object Storage credentials. Each Ansible smoke command has its own timeout: migrations and seed each default to 600 seconds, and `data:smoke` defaults to 1800 seconds with a 30-second forced-kill grace period. If a timeout names one of these commands, check Oracle ADB connectivity, Object Storage connectivity, and the cleanup notes below before rerunning.

The workflow dispatch input `tools_image_tag` defaults to `production`; override it only when intentionally validating a different tools image tag. The smoke-created published item and private image are deleted through the catalog service before the script exits, and the script fails if the smoke item still exists or the read-back media object remains readable after cleanup. Seed records remain additive; reset the target schema before rerunning if you need a pristine sample dataset. If a workflow run fails before cleanup, search for smoke records with tag `smoke` or signer `Phase Two Smoke` and remove them through the operator delete path in [temporary-production-data-entry.md](temporary-production-data-entry.md). If metadata was never written but an image upload may have succeeded, inspect Object Storage for an `autographs/{itemId}/...smoke.svg` object key from the failed run logs and delete that object manually.

The deployed app also exposes `GET /health/data` for configuration readiness and `GET /health/data?live=1` for guarded live checks. The live check requires `Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}` and verifies both Oracle catalog access and private media bucket readiness.

Published images are served through app-mediated URLs shaped as `/api/catalog/{itemId}/images/{imageId}`. The URL contains app-level catalog identifiers only; it does not expose Object Storage bucket credentials, signed direct URLs, or raw object keys as the public access contract.

Temporary production data entry is operator-only and documented in [temporary-production-data-entry.md](temporary-production-data-entry.md). It is current only until the Rust private controller and minimal static admin seed/publish path pass the Phase 5 live cutover checkpoint.

## Workflow Behavior

Pull requests run `.github/workflows/ci.yml`. The CI workflow checks the
Next.js app and Rust controller, builds the container images without pushing
them, validates Terraform, and runs Ansible syntax/lint checks for the quadlet
deployment.

Merges to `main` run `.github/workflows/deploy.yml`. The deploy workflow:

1. validates the repository,
2. publishes prebuilt tools and Rust controller images to `ghcr.io`,
3. runs `terraform apply`,
4. optionally taints and recreates the runtime VM when manually requested,
5. connects to the OCI VM over SSH through Ansible,
6. installs/maintains Podman, firewalld, swap, and masked systemd services,
7. copies wallet and OCI API key material to protected VM paths,
8. installs systemd quadlets for the dedicated Podman network, private controller, shared static volume, and Caddy container,
9. stops, disables, and removes the retired Next.js app runtime if present,
10. pulls the published runtime images and restarts the quadlet services,
11. checks the Caddy-fronted static release and Rust controller health routes.

The VM pulls images built by GitHub Actions. The VM does not build application
code or generate catalog content during deploy.

### Static Preview

On the VM, Caddy exposes the current generated release through a localhost-bound
preview in addition to the public hostname:

```bash
curl --fail --silent http://127.0.0.1:8081/releases/<release-id>/collection/
```

The `/admin` shell and `/admin/api/*` controller proxy are available through
the normal hostname. Plan 05-07 owns the explicit public static cutover after a
candidate release passes local validation.

The public cutover route shape is now the deployed Caddy shape:

- `/admin/api/*` reverse-proxies to `autographs-controller:8080` on the private
  Podman network.
- `/admin/*` serves the static admin shell from `/srv/autographs/admin`.
- `/api/operator/*` continues to return `404`.
- Anonymous public traffic serves `/srv/autographs/static/current` directly.
- `127.0.0.1:8081` on the VM maps to Caddy's static preview listener and serves
  `/srv/autographs/static`.

The deploy role stops and disables the retired `autographs-app.service`, removes
its quadlet, and force-removes any leftover `autographs-app` Podman container.

The Ansible-managed `/opt/autographs/env/controller.env` intentionally uses
persistent controller providers in deployment:
`AUTOGRAPHS_CONTROLLER_DB_PROVIDER=oracle` and
`AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER=oci-instance-principal`. The
controller-specific environment also sets `OCI_AUTH_MODE=instance_principal`
and relies on `OCI_MEDIA_NAMESPACE` plus `OCI_MEDIA_BUCKET_NAME` from the
Ansible-managed runtime environment. Do not rely on a manual edit to
`controller.env`; Ansible owns that file on each deploy. The deploy workflow
verifies `/admin/api/health` reports those active provider modes before it
succeeds.

Run the mandatory live static publish smoke from
[static-runtime-runbook.md](static-runtime-runbook.md) before changing the
public hostname. Planned downtime is acceptable for the first switch. Recovery
is roll-forward oriented: correct the source or controller, run a full rebuild,
validate through the localhost candidate listener, and promote the repaired
release.

The public Caddy route serves the same generated static release. `/api/catalog/*`
and `/api/operator/*` are no longer part of the public runtime contract.

The Ansible deploy role keeps `/.swapfile` at 2 GiB and writes `vm.swappiness=20` through `/etc/sysctl.d/99-autographs-swap.conf`. This is intentional for the Always Free runtime shape because controller publishing, image processing, and tools/smoke scripts can briefly exceed the VM's physical memory.

### Runtime VM Recreation

Terraform no longer embeds the runtime bootstrap state in cloud-init. If a clean VM is needed, manually run the deploy workflow with `recreate_runtime_instance=true`. The workflow taints `module.compute.oci_core_instance.runtime[0]` before `terraform apply`, forcing OCI to recreate the runtime VM and then letting Ansible converge the full production state onto the replacement instance.

Image cleanup runs separately through `.github/workflows/image-cleanup.yml` on a weekly schedule and by manual dispatch. One job prunes old VM-local app/tools images while keeping the active image from `${DEPLOY_PATH}/env/app.env`, `latest`, `GHCR_CLEANUP_PROTECTED_TAGS`, and the newest `AUTOGRAPHS_LOCAL_IMAGE_RETAIN_COUNT` matching images per repository. Another job prunes old GHCR package versions while keeping `latest`, protected tags, the newest `GHCR_CLEANUP_RETAIN_TAGGED` versions, and versions newer than `GHCR_CLEANUP_MIN_AGE_DAYS`. Use the manual `dry_run=true` input to preview deletions.

## Manual Smoke Path

After deployment, verify from your workstation:

```bash
curl --fail --silent "http://${VM_PUBLIC_IP}/health"
```

Expected response:

```json
{"ok":true,"service":"autographs","scope":"proof-of-life"}
```

If this fails, check the VM:

```bash
ssh opc@"${VM_PUBLIC_IP}"
sudo systemctl status autographs-network.service autographs-controller.service autographs-caddy.service
sudo journalctl -u autographs-controller.service -u autographs-caddy.service --since "30 minutes ago"
sudo podman ps
```

## Current Auth Note

Phase 1 uses OCI API signing keys in GitHub Secrets. That is intentionally documented as replaceable auth, so a later OIDC or short-lived credential flow can replace the auth step without changing the GHCR image publication, Terraform execution, or VM deployment shape.
