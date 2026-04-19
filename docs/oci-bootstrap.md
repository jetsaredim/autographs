# OCI Bootstrap Runbook

This phase keeps OCI setup manual only where Terraform cannot bootstrap itself.
The long-term owner is still the Terraform root in `infra/terraform/`.

## Manual-Once Boundary

Manual work should stop after these prerequisites:

1. An OCI API-signing identity exists for the operator running Terraform.
2. That identity can create or update the Phase 1 compartment, policies, bucket,
   network, and VM in the chosen parent scope.
3. You know both the tenancy home region and the runtime region.

Everything after that should happen through Terraform, with any unavoidable
manual bootstrap resources imported immediately afterward.

## Required Local Inputs

- Terraform CLI: use the repo-local binary at `.tools/terraform/terraform`.
- Optional but helpful: OCI CLI for discovering OCIDs and validating identity.
- A local copy of `infra/terraform/environments/prod/terraform.tfvars`.
- A local backend config file derived from
  `infra/terraform/bootstrap/backend.hcl.example`.

## Region Model

- `home_region`: the OCI home region for IAM resources such as compartments and
  policies.
- `region`: the runtime region for the VCN, subnet, VM, and usually the
  Terraform state bucket.

Do not guess these values. Set them explicitly in `terraform.tfvars`.

## Bootstrap Flow

1. Copy the example variables file and fill in real values locally.

```bash
cp infra/terraform/environments/prod/terraform.tfvars.example \
  infra/terraform/environments/prod/terraform.tfvars
```

2. Start with local state so Terraform can create the remote backend bucket and
   the compartment-scoped baseline.

```bash
.tools/terraform/terraform -chdir=infra/terraform init -backend=false
.tools/terraform/terraform -chdir=infra/terraform plan \
  -var-file=environments/prod/terraform.tfvars
.tools/terraform/terraform -chdir=infra/terraform apply \
  -var-file=environments/prod/terraform.tfvars
```

3. If the bucket or compartment had to be created manually first, import it
   before migrating state. Follow [imports.md](/home/jgreenwa/dev/git/github.com/jetsaredim/autographs/infra/terraform/bootstrap/imports.md).

4. Create a local `infra/terraform/bootstrap/backend.hcl` from the example
   file. Keep sensitive credentials out of that file when possible and prefer
   environment variables or interactive entry.

5. Migrate the existing local state into OCI Object Storage.

```bash
.tools/terraform/terraform -chdir=infra/terraform init \
  -migrate-state \
  -backend-config=bootstrap/backend.hcl
```

6. Re-run plan/apply after migration. From this point on, treat the remote OCI
   backend as the source of truth.

```bash
.tools/terraform/terraform -chdir=infra/terraform plan \
  -var-file=environments/prod/terraform.tfvars
```

## IAM Boundary

The baseline is intentionally compartment-scoped:

- Deploy automation gets a dedicated policy seam for routine changes inside the
  project compartment.
- The human operator gets a separate policy seam for day-two management.
- Routine deployment should not need tenancy-wide `manage all-resources`.

If you need broader tenancy admin access during day zero, keep it outside the
steady-state deploy identity and remove it from normal operations once bootstrap
is complete.
