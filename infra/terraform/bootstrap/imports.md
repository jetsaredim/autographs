# Terraform Import Path

Use import only for resources that had to be created manually before Terraform
could take over. The intended steady state is still "managed by Terraform," so
leave the corresponding `create_*` flag enabled when you plan to import the
resource into state.

## Before You Import

1. Update `infra/terraform/tenancy/environments/prod/terraform.tfvars` so
   names, OCIDs, and regions match the already-existing OCI bootstrap
   resources.
2. Run a local-state init first:

```bash
terraform -chdir=infra/terraform/tenancy init -backend=false
```

3. Use the same `-var-file` during every import command so Terraform resolves
   module addresses consistently.

## Common Imports

### Project compartment created manually

Keep `create_compartment = true`, then import the existing compartment into the
managed resource address:

```bash
terraform -chdir=infra/terraform/tenancy import \
  -var-file=environments/prod/terraform.tfvars \
  'module.iam.oci_identity_compartment.project[0]' \
  'ocid1.compartment.oc1..replace_me'
```

### Compartment-scoped policies created manually in the home region

```bash
terraform -chdir=infra/terraform/tenancy import \
  -var-file=environments/prod/terraform.tfvars \
  module.iam.oci_identity_policy.deploy \
  'ocid1.policy.oc1..replace_me'

terraform -chdir=infra/terraform/tenancy import \
  -var-file=environments/prod/terraform.tfvars \
  module.iam.oci_identity_policy.operator \
  'ocid1.policy.oc1..replace_me'
```

### State bucket created manually to break the backend bootstrap paradox

Keep `create_state_bucket = true`, then import the bucket using the OCI Object
Storage import ID format:

```bash
terraform -chdir=infra/terraform/tenancy import \
  -var-file=environments/prod/terraform.tfvars \
  'module.state_bucket.oci_objectstorage_bucket.this[0]' \
  'n/<namespace>/b/<bucket_name>'
```

### Deploy group, deploy user, and API key created manually

```bash
terraform -chdir=infra/terraform/tenancy import \
  -var-file=environments/prod/terraform.tfvars \
  'module.iam.oci_identity_group.deploy[0]' \
  'ocid1.group.oc1..replace_me'

terraform -chdir=infra/terraform/tenancy import \
  -var-file=environments/prod/terraform.tfvars \
  'module.iam.oci_identity_user.deploy[0]' \
  'ocid1.user.oc1..replace_me'

terraform -chdir=infra/terraform/tenancy import \
  -var-file=environments/prod/terraform.tfvars \
  'module.iam.oci_identity_api_key.deploy[0]' \
  'users/<deploy-user-ocid>/apiKeys/<api-key-fingerprint>'
```

## After Import

1. Run `terraform plan` against local state and resolve any drift until the
   imported resources are clean.
2. Only after the state bucket is in Terraform state, migrate to the OCI remote
   backend with:

```bash
terraform -chdir=infra/terraform/tenancy init \
  -migrate-state \
  -backend-config=../bootstrap/backend.hcl \
  -backend-config=key=envs/prod/tenancy-bootstrap.tfstate
```

3. Re-run `terraform plan` after migration to confirm the remote backend and
   local configuration describe the same infrastructure.
