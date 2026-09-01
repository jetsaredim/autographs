# OCI Bootstrap Runbook

This phase keeps OCI setup manual only where Terraform cannot bootstrap itself.
The root in `infra/terraform/tenancy/` owns tenancy/home-region IAM and the state
bucket; the regional deployment root in `infra/terraform/` owns the runtime
Vault, key, secrets, network, compute, database, and media resources.

## Manual-Once Boundary

Manual work should stop after these prerequisites:

1. An OCI API-signing identity exists for the operator running Terraform.
2. That identity can create or update the Phase 1 compartment, policies, bucket,
   network, and VM in the chosen parent scope.
3. You know both the tenancy home region and the runtime region.

Everything after that should happen through Terraform, with any unavoidable
manual bootstrap resources imported immediately afterward.

## Required Local Inputs

- Terraform CLI v1.12.0 or greater; use the repo-local binary at
  `.tools/terraform/terraform` when available.
- Optional but helpful: OCI CLI for discovering OCIDs and validating identity.
- A local copy of
  `infra/terraform/tenancy/environments/prod/terraform.tfvars`.
- A local backend config file derived from
  `infra/terraform/bootstrap/backend.hcl.example`.

## Region Model

- `home_region`: the OCI home region for IAM resources such as compartments and
  policies.
- `region`: the region for the Terraform state bucket and regional deployment
  resources, including Vault, keys, and secrets.

Do not guess these values. Set them explicitly in `terraform.tfvars`.

## Bootstrap Flow

1. Copy the tenancy example variables file and fill in real values locally.

```bash
cp infra/terraform/tenancy/environments/prod/terraform.tfvars.example \
  infra/terraform/tenancy/environments/prod/terraform.tfvars
```

2. Start the tenancy root with local state.

```bash
.tools/terraform/terraform -chdir=infra/terraform/tenancy init -backend=false
```

3. If the bucket, compartment, deploy identity, or policy resources had to be
   created manually first, import them before the first plan/apply. Follow
   [imports.md](../infra/terraform/bootstrap/imports.md).

4. Plan and apply the tenancy root so Terraform creates or updates the remote
   backend bucket, project compartment, deploy/operator identities, and IAM
   policy baseline.

```bash
.tools/terraform/terraform -chdir=infra/terraform/tenancy plan \
  -var-file=environments/prod/terraform.tfvars
.tools/terraform/terraform -chdir=infra/terraform/tenancy apply \
  -var-file=environments/prod/terraform.tfvars
```

5. Create a local `infra/terraform/bootstrap/backend.hcl` from the example
   file. Keep sensitive credentials out of that file when possible and prefer
   environment variables or interactive entry.

6. Migrate the existing tenancy local state into OCI Object Storage. The
   tenancy state must use the `envs/prod/tenancy-bootstrap.tfstate` key.

```bash
.tools/terraform/terraform -chdir=infra/terraform/tenancy init \
  -migrate-state \
  -backend-config=../bootstrap/backend.hcl \
  -backend-config=key=envs/prod/tenancy-bootstrap.tfstate
```

7. Re-run the tenancy plan after migration. From this point on, treat the
   remote OCI backend as the source of truth.

```bash
.tools/terraform/terraform -chdir=infra/terraform/tenancy plan \
  -var-file=environments/prod/terraform.tfvars
```

8. After the tenancy root is applied, configure and run the regional deployment
   root in `infra/terraform/`. Its remote backend should continue to use the
   runtime state key, `envs/prod/terraform.tfstate`.

```bash
cp infra/terraform/environments/prod/terraform.tfvars.example \
  infra/terraform/environments/prod/terraform.tfvars

.tools/terraform/terraform -chdir=infra/terraform init \
  -backend-config=bootstrap/backend.hcl \
  -backend-config=key=envs/prod/terraform.tfstate
.tools/terraform/terraform -chdir=infra/terraform plan \
  -var-file=environments/prod/terraform.tfvars
.tools/terraform/terraform -chdir=infra/terraform apply \
  -var-file=environments/prod/terraform.tfvars
```

9. For a fresh environment, leave both `runtime_secrets_ready` and
   `create_autonomous_database` set to `false` for the first runtime-root
   apply. That apply creates the Vault, key, and three deliberately unusable
   secret shells without allowing ADB or deploy automation to consume them.
   Replace all three `CURRENT` secret versions out of band with the real Oracle
   database password, Oracle wallet password, and Argon2 admin password hash.
   Secret contents must not be passed through Terraform, committed files, or
   GitHub Variables.

10. After verifying all three secret values in OCI Vault, set the local
    `runtime_secrets_ready` input and the `OCI_RUNTIME_SECRETS_READY` GitHub
    Variable to `true`. Only then enable `create_autonomous_database` /
    `OCI_CREATE_AUTONOMOUS_DATABASE` and run the normal deployment. Terraform
    fails closed if ADB is requested before readiness, and the deploy workflow
    withholds the secret OCID outputs and stops before Ansible while readiness
    remains false.

## IAM Boundary

The baseline is intentionally compartment-scoped:

- Deploy automation gets a dedicated policy seam for routine changes inside the
  project compartment, including state-bucket object access but not direct media
  object access.
- Deploy automation can manage vaults, keys, and the secret family only inside
  the dedicated Autographs project compartment. This permits the runtime root to
  own those regional resources and later coordinate native ADB secret rotation
  without granting tenancy-wide security-resource management.
- The human operator gets a separate policy seam for day-two management.
- Runtime instances get a dynamic-group policy for private media bucket object
  access through OCI instance principals.
- Routine deployment should not need tenancy-wide `manage all-resources`.

The runtime dynamic group intentionally matches instances in the single-purpose
project compartment rather than a specific runtime instance OCID. That avoids a
circular dependency between tenancy IAM and the runtime instance created by the
deployment root. Do not place unrelated VMs in the Autographs project
compartment without first splitting the dynamic-group selector or compartment
model.

Terraform creates the `autographs-operators` group and policy boundary, but it
does not create or assign human operator users. Add your human or federated OCI
identity to that group manually through the tenancy's normal identity process.
The deploy group, operator group, and GitHub deploy user are end-state managed
by Terraform; import existing resources before applying if they were created
manually.

If you need broader tenancy admin access during day zero, keep it outside the
steady-state deploy identity and remove it from normal operations once bootstrap
is complete.
