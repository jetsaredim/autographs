# Deployment Runbook

This runbook gets the app from a clean checkout to an OCI VM running systemd-managed Podman quadlets: public `Caddy` in front of a private `Next.js` app container on a dedicated Podman network, with Terraform-managed hooks for Oracle Autonomous Database Free and private OCI Object Storage media.

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
6. Migrate existing state with [terraform-tenancy-split.md](terraform-tenancy-split.md) if this is an existing environment.
7. Run runtime Terraform locally once if needed to prove the baseline and obtain outputs.
8. Follow [dns-runbook.md](dns-runbook.md) when enabling the public app hostname through Porkbun.

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
- `AUTOGRAPHS_DOMAIN`

`GHCR_IMAGE_REPOSITORY` should be a `ghcr.io` image path such as `ghcr.io/jetsaredim/autographs/app`.

`OCI_RUNTIME_SHAPE`, `OCI_RUNTIME_OCPUS`, `OCI_RUNTIME_MEMORY_GBS`, `VM_PUBLIC_IP`, `DEPLOY_SSH_USER`, `DEPLOY_PATH`, `DEPLOY_SSH_READY_TIMEOUT_SECONDS`, `DEPLOY_SSH_READY_INTERVAL_SECONDS`, `GHCR_IMAGE_REPOSITORY`, `GHCR_CLEANUP_RETAIN_TAGGED`, `GHCR_CLEANUP_MIN_AGE_DAYS`, and `AUTOGRAPHS_DOMAIN` have workflow defaults or fallbacks. The OCPU and memory inputs are used only for `.Flex` shapes; fixed shapes such as `VM.Standard.E2.1.Micro` omit the Terraform `shape_config` block. The availability domain, runtime image OCID, SSH public keys, and Object Storage namespace are tenancy-specific and should be set explicitly.

Leave `OCI_CREATE_AUTONOMOUS_DATABASE` and `OCI_CREATE_MEDIA_BUCKET` as `false` until the tenancy-specific namespace, ADMIN password, and runtime connection values are ready. When enabling Phase 2 data services, Terraform provisions the ADB and bucket, while the deploy step passes app runtime coordinates through the VM-local quadlet environment file.

For the initial production path, use the ADB wallet-based mTLS connection. Set `OCI_AUTONOMOUS_DATABASE_IS_MTLS_CONNECTION_REQUIRED=true`, set `ORACLE_DB_CONNECT_STRING` to a wallet alias such as `autographsdb_medium`, set `ORACLE_DB_WALLET_DIR=/opt/autographs/wallet`, and store the base64-encoded wallet zip in the `ORACLE_DB_WALLET_ZIP_BASE64` GitHub Secret. Also store the wallet download password in `ORACLE_DB_WALLET_PASSWORD` if the Thin driver requires it. The deploy workflow unpacks that wallet onto the VM and mounts it read-only into the app container.

The OCI API signing key remains a GitHub Secret named `OCI_PRIVATE_KEY_PEM`. Terraform uses it from the runner temp directory. Runtime deploy copies it to `${DEPLOY_PATH}/secrets/oci_api_key.pem`, mounts `${DEPLOY_PATH}/secrets` read-only into the app container, and sets `OCI_PRIVATE_KEY_PATH=/opt/autographs/secrets/oci_api_key.pem`. This preserves PEM newlines for the OCI SDK and avoids putting multiline private keys in the quadlet environment file.

## Data and Media Smoke

Basic `/health` remains a proof-of-life check and does not require Oracle or Object Storage secrets. Use the deeper VM smoke workflow only when data-service credentials are present:

Run `.github/workflows/data-smoke.yml` manually from GitHub Actions. The workflow resolves the runtime VM IP, starts the tools image on the VM's Podman network, runs migrations, loads representative seed records with generated SVG fixture images, creates a published smoke item, uploads a private smoke image, reads it back through the catalog/media service, and verifies the deployed app-mediated image route with `AUTOGRAPHS_SMOKE_BASE_URL`. It is intentionally manual because it requires live ADB and private Object Storage credentials. Seed records are additive; reset the target schema before rerunning if you need a pristine sample dataset.

The deployed app also exposes `GET /health/data` for configuration readiness and `GET /health/data?live=1` for guarded live checks. The live check requires `Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}` and verifies both Oracle catalog access and private media bucket readiness.

Published images are served through app-mediated URLs shaped as `/api/catalog/{itemId}/images/{imageId}`. The URL contains app-level catalog identifiers only; it does not expose Object Storage bucket credentials, signed direct URLs, or raw object keys as the public access contract.

## Workflow Behavior

Pull requests run `.github/workflows/ci.yml`. The CI workflow checks the Next.js app, builds the container images without pushing them, validates Terraform, and runs Ansible syntax/lint checks for the quadlet deployment.

Merges to `main` run `.github/workflows/deploy.yml`. The deploy workflow:

1. validates the repository,
2. publishes a prebuilt app image to `ghcr.io`,
3. runs `terraform apply`,
4. optionally taints and recreates the runtime VM when manually requested,
5. connects to the OCI VM over SSH through Ansible,
6. installs/maintains Podman, firewalld, swap, and masked systemd services,
7. copies wallet and OCI API key material to protected VM paths,
8. installs systemd quadlets for the dedicated Podman network, app container, and Caddy container,
9. pulls the published app image and restarts the quadlet services,
10. checks the Caddy-fronted `/health` proof-of-life route,
11. prunes unused local Podman images from the runtime VM only after the health check succeeds.

The VM pulls the image built by GitHub Actions. The VM does not build application code during deploy.

The Ansible deploy role keeps `/.swapfile` at 2 GiB and writes `vm.swappiness=20` through `/etc/sysctl.d/99-autographs-swap.conf`. This is intentional for the Always Free runtime shape because `tsx`, Next.js, and smoke/admin scripts can briefly exceed the VM's physical memory.

The post-health cleanup playbook runs `podman image prune --force` on the runtime VM. Podman only removes dangling images that are not used by containers, so the active app and Caddy images remain protected by their running quadlet-managed containers.

### Runtime VM Recreation

Terraform no longer embeds the runtime bootstrap state in cloud-init. If a clean VM is needed, manually run the deploy workflow with `recreate_runtime_instance=true`. The workflow taints `module.compute.oci_core_instance.runtime[0]` before `terraform apply`, forcing OCI to recreate the runtime VM and then letting Ansible converge the full production state onto the replacement instance.

GHCR cleanup runs separately through `.github/workflows/ghcr-cleanup.yml` on a weekly schedule and by manual dispatch. By default it keeps the newest 10 app image versions, keeps `latest`, and refuses to delete images newer than 7 days. Tune those guardrails with `GHCR_CLEANUP_RETAIN_TAGGED` and `GHCR_CLEANUP_MIN_AGE_DAYS`; use the manual `dry_run=true` input to preview deletions.

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
sudo systemctl status autographs-network.service autographs-app.service autographs-caddy.service
sudo journalctl -u autographs-app.service -u autographs-caddy.service --since "30 minutes ago"
sudo podman ps
```

## Current Auth Note

Phase 1 uses OCI API signing keys in GitHub Secrets. That is intentionally documented as replaceable auth, so a later OIDC or short-lived credential flow can replace the auth step without changing the GHCR image publication, Terraform execution, or VM deployment shape.
