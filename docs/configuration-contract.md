# Configuration Contract

Phase 1 uses one explicit contract for local work, GitHub validation, and GitHub deployment. Sensitive values live in repo-level GitHub Secrets. Non-sensitive coordinates live in committed examples or repo-level GitHub Variables. Optional GitHub Environments can add approval gates later, but they are optional and additive, not required for the baseline delivery path.

## Committed Examples

| File | Purpose |
|------|---------|
| `.env.example` | Local development and operator reference values |
| `.github/.env.github.example` | Repo-level GitHub Secrets and GitHub Variables checklist |
| `infra/terraform/tenancy/environments/prod/terraform.tfvars.example` | Tenancy/bootstrap Terraform variable shape for local operator runs |
| `infra/terraform/environments/prod/terraform.tfvars.example` | Runtime/app Terraform variable shape for local operator runs |
| `infra/terraform/bootstrap/backend.hcl.example` | OCI remote-state backend coordinate shape |

Never put real private keys, API signing material, SSH private keys, or Terraform state in committed files.

## GitHub Secrets

These are repo-level GitHub Secrets for the Phase 1 baseline.

| Secret | Used By | Purpose |
|--------|---------|---------|
| `OCI_CLI_USER_OCID` | deploy workflow | OCI API signing identity user OCID |
| `OCI_TENANCY_OCID` | deploy workflow | OCI tenancy OCID |
| `OCI_FINGERPRINT` | deploy workflow | OCI API signing key fingerprint |
| `OCI_PRIVATE_KEY_PEM` | deploy workflow | OCI API signing private key PEM |
| `DEPLOY_SSH_PRIVATE_KEY` | deploy workflow | SSH private key for the OCI runtime VM |

The current Phase 1 OCI authentication path uses OCI API signing keys because that is the initial locked decision. Treat this as a replaceable auth adapter: the workflow isolates these inputs so a future OIDC or other short-lived auth path can replace OCI API signing keys without redesigning the image build, Terraform, or VM deployment steps.

## GitHub Variables

These are repo-level GitHub Variables unless an optional GitHub Environment overrides them.

| Variable | Purpose |
|----------|---------|
| `OCI_COMPARTMENT_OCID` | Project compartment OCID produced by the tenancy bootstrap root |
| `OCI_AVAILABILITY_DOMAIN` | Availability domain for the runtime VM |
| `OCI_RUNTIME_IMAGE_OCID` | Oracle Linux image OCID for the runtime VM |
| `OCI_RUNTIME_SHAPE` | Optional OCI compute shape override; defaults to `VM.Standard.E2.1.Micro` |
| `OCI_RUNTIME_OCPUS` | Optional flex-shape OCPU count; ignored by fixed shapes |
| `OCI_RUNTIME_MEMORY_GBS` | Optional flex-shape memory in GB; ignored by fixed shapes |
| `OCI_RUNTIME_SSH_PUBLIC_KEYS` | JSON list of SSH public keys injected into the VM |
| `OCI_OBJECT_STORAGE_NAMESPACE` | OCI Object Storage namespace for Terraform remote state |
| `VM_PUBLIC_IP` | Runtime VM public IP; Terraform output can replace this when available |
| `DEPLOY_SSH_USER` | SSH user for deploys, usually `opc` |
| `DEPLOY_PATH` | Directory on the VM that stores compose and nginx runtime files |
| `GHCR_IMAGE_REPOSITORY` | Published app image path, for example `ghcr.io/jetsaredim/autographs/app` |

The deploy workflow intentionally codifies the single-region tenancy defaults: runtime region and home region are both `us-ashburn-1`, the Terraform state bucket is `autographs-tf-state`, and the runtime state object key is `envs/prod/terraform.tfstate`. `GHCR_IMAGE_REPOSITORY`, `OCI_COMPARTMENT_OCID`, `OCI_AVAILABILITY_DOMAIN`, `OCI_RUNTIME_IMAGE_OCID`, `OCI_RUNTIME_SHAPE`, `OCI_RUNTIME_OCPUS`, `OCI_RUNTIME_MEMORY_GBS`, `OCI_RUNTIME_SSH_PUBLIC_KEYS`, `OCI_OBJECT_STORAGE_NAMESPACE`, and `VM_PUBLIC_IP` are intentionally non-secret deployment coordinates. Keep them visible as GitHub Variables so deploy behavior can be audited without opening secrets.

## Local Operator Values

For local runs, copy `.env.example` to an untracked file such as `.env.local`, then copy both Terraform examples:

- `infra/terraform/tenancy/environments/prod/terraform.tfvars.example` to `infra/terraform/tenancy/environments/prod/terraform.tfvars`
- `infra/terraform/environments/prod/terraform.tfvars.example` to `infra/terraform/environments/prod/terraform.tfvars`

Local Terraform uses:

- `tenancy_ocid`
- `user_ocid`
- `fingerprint`
- `private_key_path`
- bootstrap parent compartment and deploy identity inputs in the tenancy root
- `compartment_ocid` in the runtime/app root
- runtime VM image, availability domain, and SSH public keys
- Object Storage namespace, bucket, and key when initializing the remote backend

GitHub Actions uses equivalent `TF_VAR_*` environment variables and writes `OCI_PRIVATE_KEY_PEM` to a temporary private key file at runtime.

## Optional GitHub Environments

GitHub Environments may be added later for manual approval, deployment history, or environment-specific variable overrides. They are optional GitHub Environments for this baseline. The repo-level contract above remains sufficient for Phase 1 and avoids making the deploy path depend on environment availability or repository plan features.

## Runtime Image Contract

Deployments publish a prebuilt app image to `ghcr.io` and set `AUTOGRAPHS_APP_IMAGE` on the VM. The VM does not build the application. It pulls the exact image published by GitHub Actions, restarts Docker Compose, and checks the nginx-fronted `/health` route before the workflow succeeds.
