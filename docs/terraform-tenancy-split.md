# Terraform Tenancy Split Migration

This runbook migrates an existing single Terraform state into two roots:

- `infra/terraform/tenancy`: compartment, deploy user, groups, policies, and state bucket
- `infra/terraform`: runtime network and compute resources

The goal is a no-recreate migration. Do not run `apply` until both roots plan
cleanly against their intended state.

## State Keys

Use the same OCI Object Storage bucket and namespace, but separate object keys:

- tenancy bootstrap: `envs/prod/tenancy-bootstrap.tfstate`
- runtime/app: `envs/prod/terraform.tfstate`

## Before You Start

Work from a clean branch and keep local plan/state files out of Git.

```bash
terraform -chdir=infra/terraform init \
  -backend-config=bootstrap/backend.hcl \
  -reconfigure

terraform -chdir=infra/terraform plan \
  -var-file=environments/prod/terraform.tfvars
```

The runtime plan should be clean before migration. If it is not clean, resolve
that drift first.

## Pull Current State

Create a throwaway migration directory outside the Terraform roots:

```bash
mkdir -p /tmp/autographs-state-split
terraform -chdir=infra/terraform state pull > /tmp/autographs-state-split/current.tfstate
cp /tmp/autographs-state-split/current.tfstate /tmp/autographs-state-split/runtime.tfstate
cp /tmp/autographs-state-split/current.tfstate /tmp/autographs-state-split/tenancy.tfstate
```

## Move Bootstrap Resources Out Of Runtime State

Remove bootstrap-owned addresses from the runtime state file:

```bash
terraform state rm \
  -state=/tmp/autographs-state-split/runtime.tfstate \
  data.oci_objectstorage_namespace.ns \
  module.iam \
  module.state_bucket
```

Remove runtime-owned addresses from the tenancy state file:

```bash
terraform state rm \
  -state=/tmp/autographs-state-split/tenancy.tfstate \
  module.network \
  module.compute
```

If `terraform state list` shows a module is absent because those resources were
disabled, remove only the addresses that exist. Output values copied from the
old combined state may remain until the next plan/apply refreshes outputs; that
is expected as long as resources are not being recreated.

## Align New Tenancy Addresses

The tenancy root references the shared modules from one directory deeper, so the
bootstrap resource addresses remain `module.iam.*` and `module.state_bucket.*`.
New resources introduced by the split, such as the deploy user/group
membership, will be created by the tenancy root unless you import existing
manual resources first.

To import existing manual resources instead of creating them, initialize the
tenancy root and import each resource before the first tenancy `plan`:

```bash
terraform -chdir=infra/terraform/tenancy init -backend=false

terraform -chdir=infra/terraform/tenancy import \
  -var-file=environments/prod/terraform.tfvars \
  'module.iam.oci_identity_group.deploy[0]' '<deploy-group-ocid>'

terraform -chdir=infra/terraform/tenancy import \
  -var-file=environments/prod/terraform.tfvars \
  'module.iam.oci_identity_user.deploy[0]' '<deploy-user-ocid>'
```

Repeat for any manually created operator group, membership, or API key that
should be Terraform-owned.

## Push Split States

Initialize each root against its target backend key, then push the prepared
state file.

For the tenancy root, use a local backend config whose `key` is
`envs/prod/tenancy-bootstrap.tfstate`:

```bash
terraform -chdir=infra/terraform/tenancy init \
  -backend-config=../bootstrap/backend.hcl \
  -backend-config=key=envs/prod/tenancy-bootstrap.tfstate \
  -reconfigure

terraform -chdir=infra/terraform/tenancy state push /tmp/autographs-state-split/tenancy.tfstate
```

For the runtime root, keep the existing runtime key:

```bash
terraform -chdir=infra/terraform init \
  -backend-config=bootstrap/backend.hcl \
  -backend-config=key=envs/prod/terraform.tfstate \
  -reconfigure

terraform -chdir=infra/terraform state push /tmp/autographs-state-split/runtime.tfstate
```

## Validate Both Roots

Run plans in this order:

```bash
terraform -chdir=infra/terraform/tenancy plan \
  -var-file=environments/prod/terraform.tfvars

terraform -chdir=infra/terraform plan \
  -var-file=environments/prod/terraform.tfvars
```

Expected result:

- tenancy root: no changes, except intentional creation/import of deploy identity resources
- runtime root: no changes

If either root wants to recreate existing resources, stop and fix state
ownership before applying.

## GitHub Values After Migration

Set GitHub deploy values from tenancy/runtime outputs:

```bash
gh variable set OCI_COMPARTMENT_OCID --body "$(terraform -chdir=infra/terraform/tenancy output -raw compartment_ocid)" --repo jetsaredim/autographs
gh variable set OCI_OBJECT_STORAGE_NAMESPACE --body "$(terraform -chdir=infra/terraform/tenancy output -raw object_storage_namespace)" --repo jetsaredim/autographs
gh variable set VM_PUBLIC_IP --body "$(terraform -chdir=infra/terraform output -raw runtime_public_ip)" --repo jetsaredim/autographs
```

Set `OCI_CLI_USER_OCID`, `OCI_FINGERPRINT`, and `OCI_PRIVATE_KEY_PEM` from the
deploy user's API key setup. Generate the private key outside Terraform and keep
it only in GitHub Secrets or your operator secret store.
