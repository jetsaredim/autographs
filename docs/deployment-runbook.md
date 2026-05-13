# Deployment Runbook

This runbook gets the Phase 1 proof-of-life app from a clean checkout to an OCI VM running the committed Docker Compose topology: public `nginx` in front of a private `Next.js` app container.

## Preconditions

- OCI tenancy exists.
- An OCI user or deploy identity has API signing keys for Phase 1.
- The project compartment and state bucket are either ready to import or can be created by Terraform during bootstrap.
- The runtime VM image OCID, availability domain, and SSH public key are known.
- Docker and Docker Compose are installed on the target VM image or through the operator's VM bootstrap process.
- GitHub repo-level GitHub Secrets and GitHub Variables from [configuration-contract.md](configuration-contract.md) are populated.
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

That command builds the app image, starts Docker Compose, and checks `http://127.0.0.1:8080/health` through nginx.

## OCI Bootstrap

1. Follow [oci-bootstrap.md](oci-bootstrap.md) to prepare the tenancy, compartment path, and Terraform state bucket.
2. This tenancy is codified as `us-ashburn-1` for both the OCI home region and runtime region.
3. The deploy workflow codifies the Terraform state bucket as `autographs-tf-state` and the state object key as `envs/prod/terraform.tfstate`.
4. If resources were created manually, import them using [imports.md](../infra/terraform/bootstrap/imports.md).
5. Run `terraform apply` locally once if needed to prove the baseline and obtain outputs.

Important operator inputs:

- `OCI_PARENT_COMPARTMENT_OCID`
- `OCI_AVAILABILITY_DOMAIN`
- `OCI_RUNTIME_IMAGE_OCID`
- `OCI_RUNTIME_SSH_PUBLIC_KEYS`
- `OCI_OBJECT_STORAGE_NAMESPACE`
- `VM_PUBLIC_IP` when not relying on Terraform output

## GitHub Configuration

Populate repo-level GitHub Secrets:

- `OCI_CLI_USER_OCID`
- `OCI_TENANCY_OCID`
- `OCI_FINGERPRINT`
- `OCI_PRIVATE_KEY_PEM`
- `DEPLOY_SSH_PRIVATE_KEY`

Populate repo-level GitHub Variables:

- `OCI_PARENT_COMPARTMENT_OCID`
- `OCI_EXISTING_COMPARTMENT_OCID`
- `OCI_AVAILABILITY_DOMAIN`
- `OCI_RUNTIME_IMAGE_OCID`
- `OCI_RUNTIME_SHAPE`
- `OCI_RUNTIME_OCPUS`
- `OCI_RUNTIME_MEMORY_GBS`
- `OCI_RUNTIME_SSH_PUBLIC_KEYS`
- `OCI_OBJECT_STORAGE_NAMESPACE`
- `VM_PUBLIC_IP`
- `DEPLOY_SSH_USER`
- `DEPLOY_PATH`
- `GHCR_IMAGE_REPOSITORY`

`GHCR_IMAGE_REPOSITORY` should be a `ghcr.io` image path such as `ghcr.io/jetsaredim/autographs/app`.

`OCI_RUNTIME_SHAPE`, `OCI_RUNTIME_OCPUS`, `OCI_RUNTIME_MEMORY_GBS`, `VM_PUBLIC_IP`, `DEPLOY_SSH_USER`, `DEPLOY_PATH`, and `GHCR_IMAGE_REPOSITORY` have workflow defaults or fallbacks. The OCPU and memory inputs are used only for `.Flex` shapes; fixed shapes such as `VM.Standard.E2.1.Micro` omit the Terraform `shape_config` block. The availability domain, runtime image OCID, SSH public keys, and Object Storage namespace are tenancy-specific and should be set explicitly.

## Workflow Behavior

Pull requests run `.github/workflows/ci.yml`. The CI workflow installs runtime tooling, runs `bash scripts/validate-ci.sh`, checks the Next.js app, and validates Terraform with `terraform init -backend=false`.

Merges to `main` run `.github/workflows/deploy.yml`. The deploy workflow:

1. validates the repository,
2. publishes a prebuilt app image to `ghcr.io`,
3. runs `terraform apply`,
4. copies the committed compose and nginx files to the OCI VM,
5. runs `docker compose pull` and restarts the runtime,
6. checks the nginx-fronted `/health` proof-of-life route.

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
docker compose -f compose.prod.yaml ps
docker compose -f compose.prod.yaml logs app nginx
```

## Current Auth Note

Phase 1 uses OCI API signing keys in GitHub Secrets. That is intentionally documented as replaceable auth, so a later OIDC or short-lived credential flow can replace the auth step without changing the GHCR image publication, Terraform execution, or VM deployment shape.
