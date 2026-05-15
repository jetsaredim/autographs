# Deployment Runbook

This runbook gets the app from a clean checkout to an OCI VM running the committed Compose topology with Podman: public `Caddy` in front of a private `Next.js` app container, with Terraform-managed hooks for Oracle Autonomous Database Free and private OCI Object Storage media.

## Preconditions

- OCI tenancy exists.
- An OCI user or deploy identity has API signing keys for Phase 1.
- The tenancy bootstrap root has created or imported the project compartment, state bucket, deploy user, groups, and policies.
- The runtime VM image OCID, availability domain, and SSH public key are known.
- Podman and `podman-compose` are installed on the target VM image or through the committed VM bootstrap process.
- GitHub repo-level GitHub Secrets and GitHub Variables from [configuration-contract.md](configuration-contract.md) are populated.
- Oracle Autonomous Database and Object Storage creation toggles are set intentionally before enabling Phase 2 data services.
- Optional GitHub Environments may be configured for approval gates, but they are not required for the baseline path.

## Local Validation

Run the same validation script used by GitHub Actions:

```bash
bash scripts/validate-ci.sh
```

To smoke-test the local runtime topology:

```bash
bash scripts/validate-runtime.sh
```

That command builds the app image, starts the Compose topology locally with Docker, and checks `http://127.0.0.1:8080/health` through Caddy.

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
- `GHCR_IMAGE_REPOSITORY`
- `AUTOGRAPHS_DOMAIN`

`GHCR_IMAGE_REPOSITORY` should be a `ghcr.io` image path such as `ghcr.io/jetsaredim/autographs/app`.

`OCI_RUNTIME_SHAPE`, `OCI_RUNTIME_OCPUS`, `OCI_RUNTIME_MEMORY_GBS`, `VM_PUBLIC_IP`, `DEPLOY_SSH_USER`, `DEPLOY_PATH`, `GHCR_IMAGE_REPOSITORY`, and `AUTOGRAPHS_DOMAIN` have workflow defaults or fallbacks. The OCPU and memory inputs are used only for `.Flex` shapes; fixed shapes such as `VM.Standard.E2.1.Micro` omit the Terraform `shape_config` block. The availability domain, runtime image OCID, SSH public keys, and Object Storage namespace are tenancy-specific and should be set explicitly.

Leave `OCI_CREATE_AUTONOMOUS_DATABASE` and `OCI_CREATE_MEDIA_BUCKET` as `false` until the tenancy-specific namespace, ADMIN password, and runtime connection values are ready. When enabling Phase 2 data services, Terraform provisions the ADB and bucket, while the deploy step passes app runtime coordinates through the VM-local Compose `.env` file.

For the initial production path, prefer the ADB console's walletless TLS connection descriptor when the database is configured with mTLS not required. In that mode, set `ORACLE_DB_CONNECT_STRING` to the full `(description=...)` descriptor and leave `ORACLE_DB_WALLET_DIR` empty.

## Data and Media Smoke

Basic `/health` remains a proof-of-life check and does not require Oracle or Object Storage secrets. Use the deeper smoke path only when data-service credentials are present:

```bash
bash scripts/smoke-data-media.sh
```

That command runs migrations, loads representative seed records with generated SVG fixture images, creates a published smoke item, uploads a private smoke image, reads it back through the catalog/media service, and confirms list/detail behavior. It is intentionally not part of CI because it requires live ADB and private Object Storage credentials. Seed records are additive; reset the target schema before rerunning if you need a pristine sample dataset.

To include the deployed app-mediated image route in the smoke proof, set `AUTOGRAPHS_SMOKE_BASE_URL` first:

```bash
AUTOGRAPHS_SMOKE_BASE_URL=https://autographs.jetsaredim.net bash scripts/smoke-data-media.sh
```

The deployed app also exposes `GET /health/data` for configuration readiness and `GET /health/data?live=1` for guarded live checks. The live check requires `Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}` and verifies both Oracle catalog access and private media bucket readiness.

Published images are served through app-mediated URLs shaped as `/api/catalog/{itemId}/images/{imageId}`. The URL contains app-level catalog identifiers only; it does not expose Object Storage bucket credentials, signed direct URLs, or raw object keys as the public access contract.

## Workflow Behavior

Pull requests run `.github/workflows/ci.yml`. The CI workflow installs runtime tooling, runs `bash scripts/validate-ci.sh`, checks the Next.js app, and validates Terraform with `terraform init -backend=false`.

Merges to `main` run `.github/workflows/deploy.yml`. The deploy workflow:

1. validates the repository,
2. publishes a prebuilt app image to `ghcr.io`,
3. runs `terraform apply`,
4. copies the committed compose and Caddy files to the OCI VM,
5. runs `podman-compose pull` and restarts the runtime,
6. checks the Caddy-fronted `/health` proof-of-life route.

The VM pulls the image built by GitHub Actions. The VM does not build application code during deploy.

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
cd /opt/autographs/compose
sudo podman-compose -f compose.prod.yaml ps
sudo podman-compose -f compose.prod.yaml logs app caddy
```

## Current Auth Note

Phase 1 uses OCI API signing keys in GitHub Secrets. That is intentionally documented as replaceable auth, so a later OIDC or short-lived credential flow can replace the auth step without changing the GHCR image publication, Terraform execution, or VM deployment shape.
