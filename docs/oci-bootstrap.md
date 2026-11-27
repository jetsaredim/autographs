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

8. After the tenancy root is applied, configure the regional deployment
   inputs in `infra/terraform/`. Its remote backend uses the runtime state key,
   `envs/prod/terraform.tfstate`. Initialize locally only if an operator needs
   to inspect the state; the GitHub bootstrap operation initializes its own
   runner and performs the first targeted apply.

```bash
cp infra/terraform/environments/prod/terraform.tfvars.example \
  infra/terraform/environments/prod/terraform.tfvars

.tools/terraform/terraform -chdir=infra/terraform init \
  -backend-config=bootstrap/backend.hcl \
  -backend-config=key=envs/prod/terraform.tfstate
```

9. For a fresh environment, configure the GitHub Secrets
   `AUTOGRAPHS_ADMIN_PASSWORD_HASH` and `ORACLE_DB_WALLET_PASSWORD`, then dispatch
   `deploy.yml` from `main` with `operation=bootstrap_secrets`. Approve the
   `secret-audit` environment review. The workflow targets only the Vault,
   key, and three runtime secrets; it rejects ADB state and non-create changes.
   OCI generates the database password as version 1. The workflow replaces only
   the exact wallet-password and admin-hash version-1 placeholders from GitHub
   and verifies the new CURRENT values. Never overwrite the generated database
   password or pass secret contents through Terraform or committed files.

10. Run the normal release deployment after bootstrap. It compares both manual
    GitHub Secrets against OCI Vault and fails before planning on a mismatch;
    it never silently rotates them. Terraform uses the managed, generated
    password secret OCID to order ADB creation. It checks secret version
    readiness before returning the controller's secret IDs. The workflow runs
    a guarded second plan to attach scheduled rotation after ADB exists.
    A future wallet/admin rotation must be explicit and update both stores.

## Optional GitHub deploy key registration

For a new tenancy using API key authentication, generate the deploy signing
key before the first tenancy plan. Use the existing operator credential to
plan and apply. This creates the deploy user and registers the public half
against that user.

```bash
oci_deploy_key_dir="$(mktemp -d)"
chmod 700 "$oci_deploy_key_dir"
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 \
  -out "$oci_deploy_key_dir/deploy.pem"
chmod 600 "$oci_deploy_key_dir/deploy.pem"
openssl pkey -in "$oci_deploy_key_dir/deploy.pem" -pubout \
  -out "$oci_deploy_key_dir/deploy-public.pem"
```

Add `-var="deploy_user_api_public_key=$(cat "$oci_deploy_key_dir/deploy-public.pem")"`
to the tenancy plan and apply in step 4. Keep the public PEM in your local
tenancy inputs for subsequent plans. After tenancy bootstrap, use the outputs
and private file to configure GitHub Secrets for the new deployment:

```bash
(
  set -euo pipefail
  oci_deploy_user_id="$(terraform -chdir=infra/terraform/tenancy output -raw deploy_user_id)"
  oci_deploy_fingerprint="$(terraform -chdir=infra/terraform/tenancy output -raw deploy_api_key_fingerprint)"
  test -n "$oci_deploy_user_id" && test -n "$oci_deploy_fingerprint" || exit 1
  printf '%s' "$oci_deploy_user_id" | gh secret set OCI_CLI_USER_OCID
  printf '%s' "$oci_deploy_fingerprint" | gh secret set OCI_FINGERPRINT
  gh secret set OCI_PRIVATE_KEY_PEM < "$oci_deploy_key_dir/deploy.pem"
)
```

Set `OCI_TENANCY_OCID` from the same tenancy inputs. The Terraform-managed key
registration contains the public PEM and exposes its fingerprint. OCI can list
registered public keys and fingerprints. The caller's signing key remains in
its secret store. For an existing deployment, confirm who manages the active
key and verify a replacement credential before retiring the old one.

## Rotate the GitHub deploy API signing key

The operator uses tenancy admin credentials for the tenancy stack. GitHub uses
the deploy user for the regional stack, including the production Object Storage
backend and Vault checks. The deployed controller and its SSH access use
separate credentials.

1. Confirm the existing key registration in OCI and tenancy state. For a
   Terraform-managed key at `module.iam.oci_identity_api_key.deploy[0]`, the
   migration plan should move it to `module.iam.oci_identity_api_key.deploy["a"]`
   without replacing it. If the key was registered outside Terraform, reconcile
   or import the registration before managing it in a slot. Stop if the
   enrollment plan deletes or replaces the current key, deploy user, or state
   bucket.
2. Generate a new RSA key pair using the preceding procedure. Add its public
   PEM to the inactive slot in the local tenancy input
   `deploy_user_api_public_keys`, then plan and apply with the operator identity.
   Use `terraform output -json deploy_api_key_fingerprints` in the tenancy root
   to read the slot fingerprints. Keep the old registration active.
3. In GitHub's `production` environment, set `OCI_DEPLOY_USER_OCID` to the
   `deploy_user_id` output. For the inactive slot, upload its private PEM and
   OCI fingerprint as a single JSON secret. For example, for slot B, supply
   the path to the newly generated private key and its fingerprint to:

   ```bash
   jq -n --rawfile private_key_pem "$candidate_private_key_path" \
     --arg fingerprint "$candidate_fingerprint" \
     '{fingerprint: $fingerprint, private_key_pem: $private_key_pem}' \
     | gh secret set OCI_DEPLOY_KEY_B --env production
   ```

   Leave `OCI_TENANCY_OCID` configured. While `legacy` is selected, keep the
   existing `OCI_CLI_USER_OCID`, `OCI_FINGERPRINT`, and `OCI_PRIVATE_KEY_PEM`
   secrets available.
4. In GitHub Actions, open **Rotate OCI deploy signing key** and run it from
   `main` as the repository owner. Supply the staged slot and its OCI
   fingerprint. Leave **Switch the active signer after verification** off
   to test the candidate without switching. The runner verifies the RSA
   fingerprint, reads existing regional state, runs a refresh-only Terraform
   plan, and reads a CURRENT Vault bundle without printing its contents.
5. To switch, dispatch the same workflow with **Switch the active signer
   after verification** on. It repeats every check and records the slot and
   fingerprint in the GitHub deployment history for `oci-deploy-signer`
   only after they pass. All regional OCI jobs read the latest successful
   selection. Each job verifies its configured key against the recorded
   fingerprint when it starts. Before the first successful switch, jobs use
   the legacy credentials.
6. Verify the selected signer in a fresh normal Terraform CI or production
   workflow. Wait for old-key jobs to finish, then use a separate tenancy
   admin plan and apply to remove only the retired registration. Remove the
   retired GitHub secret after that. To retire a legacy slot A registration,
   remove the old `deploy_user_api_public_key` input as well as any `a` entry
   from the map. Run retries from the current `main` workflow after retirement.

To roll back, run the workflow again with activation enabled and select the
previously verified slot and its fingerprint. Use `legacy` if the outgoing
key still uses the three legacy repo secrets. Keep that credential staged
and its public key registered until the incoming signer is verified. The
tenancy administrator can restore a missing public-key registration in the
OCI Console while retaining access to the tenancy stack.

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
