# Configuration Contract

The project uses one explicit contract for local work, GitHub validation, and GitHub deployment. Sensitive values live in repo-level GitHub Secrets. Non-sensitive coordinates live in committed examples or repo-level GitHub Variables. Optional GitHub Environments can add approval gates later, but they are optional and additive, not required for the baseline delivery path.

## Committed Examples

| File | Purpose |
|------|---------|
| `.env.example` | Local operator and Rust controller reference values |
| `.github/.env.github.example` | Repo-level GitHub Secrets and GitHub Variables checklist |
| `infra/terraform/tenancy/environments/prod/terraform.tfvars.example` | Tenancy/bootstrap Terraform variable shape for local operator runs |
| `infra/terraform/environments/prod/terraform.tfvars.example` | Runtime/app Terraform variable shape for local operator runs |
| `infra/terraform/bootstrap/backend.hcl.example` | OCI remote-state backend coordinate shape |

Never put real private keys, API signing material, SSH private keys, or Terraform state in committed files.


## GitHub Secrets

These are repo-level GitHub Secrets for the deployment baseline and data services.

| Secret | Used By | Purpose |
|--------|---------|---------|
| `OCI_CLI_USER_OCID` | deploy workflow | OCI API signing identity user OCID |
| `OCI_TENANCY_OCID` | deploy workflow | OCI tenancy OCID |
| `OCI_FINGERPRINT` | deploy workflow | OCI API signing key fingerprint |
| `OCI_PRIVATE_KEY_PEM` | deploy workflow | OCI API signing private key PEM |
| `DEPLOY_SSH_PRIVATE_KEY` | deploy workflow | SSH private key for the OCI runtime VM |
| `GHCR_TOKEN` | deploy workflow | Pull access for the controller image |
| `ORACLE_DB_WALLET_ZIP_BASE64` | deploy workflow / runtime | Base64-encoded ADB wallet zip used for mTLS connections |
| `ORACLE_DB_WALLET_PASSWORD` | approved Vault bootstrap and deploy verification | Wallet password source value; must match the OCI Vault CURRENT bundle and wallet zip |
| `AUTOGRAPHS_ADMIN_PASSWORD_HASH` | approved Vault bootstrap and deploy verification | Argon2 admin hash source value; must match the OCI Vault CURRENT bundle |
| `AUTOGRAPHS_OPERATOR_API_TOKEN` | runtime | Optional compatibility token for non-management diagnostics; not a collection-management credential |
| `CLOUDFLARE_API_TOKEN` | Terraform CI/deploy and controlled Cloudflare apply | Scoped token for the active Cloudflare zone, records, TLS setting, cache rules, and edge certificate read checks |
| `PORKBUN_API_KEY`, `PORKBUN_SECRET_KEY` | Terraform CI/deploy | Porkbun DNS provider credentials |
| `RELEASE_PLEASE_APP_PRIVATE_KEY` | release workflow | GitHub App installation token signing key |

The current OCI authentication path uses OCI API signing keys because that is the initial locked decision. Treat this as a replaceable auth adapter: the workflow isolates these inputs so a future OIDC or other short-lived auth path can replace OCI API signing keys without redesigning the controller image build, Terraform, or VM deployment steps.

Production obtains the Oracle database password, wallet password, and admin
hash from OCI Vault secret IDs output by Terraform. GitHub stores copies of
only the wallet password and admin hash to seed exact Terraform placeholders
during an approved `bootstrap_secrets` dispatch and detect later drift.
Normal releases never overwrite a mismatched Vault value. A rotation must
update both stores through an explicit reviewed process. Do not store the
OCI-generated database password in GitHub. Local Ansible may still pass direct
values as a transition fallback.

## GitHub Variables

This table documents supported GitHub Variables, including optional inputs
that may be unset. The current stored-name inventory is in
[iac-operational-review-findings.md](iac-operational-review-findings.md).

The production Terraform root discovers the project compartment by its
`${project_name}-${environment}-cmp` name under the tenancy OCID. Its `compartment_ocid` output
remains available to downstream steps; GitHub no longer supplies a separate
`OCI_COMPARTMENT_OCID` input.
Terraform derives the database name, database display name, private media
bucket name, and public subdomain from `project_name` and `environment`.
The deploy workflow reads the database and media bucket names from Terraform
outputs and passes them to Ansible.

| Variable | Purpose |
|----------|---------|
| `OCI_AVAILABILITY_DOMAIN` | Availability domain for the runtime VM |
| `OCI_RUNTIME_IMAGE_OCID` | Oracle Linux image OCID for the runtime VM |
| `OCI_RUNTIME_SSH_PUBLIC_KEYS` | JSON list of SSH public keys injected into the VM |
| `OCI_OBJECT_STORAGE_NAMESPACE` | OCI Object Storage namespace for Terraform remote state |
| `CLOUDFLARE_ACCOUNT_ID` | Non-secret account ID for the Terraform-managed `jetsaredim.net` zone |
| `CLOUDFLARE_MODE` | Defaults to `proxied` in Terraform and workflows; leave the GitHub variable unset in normal production. Set `dns_only` only for a reviewed proxy rollback, or explicitly set `off` for a fresh bootstrap before the Cloudflare zone exists. Set any override before Release PR CI and review its plan; ordinary deployment rejects DNS changes that differ from that successful CI record. |
| `ORACLE_DB_USER` | Runtime database user for the Rust controller container; defaults to `ADMIN` for the first bootstrap path |
| `ORACLE_DB_CONNECT_STRING` | Runtime Oracle connect alias or descriptor; use the wallet alias such as `autographsdb_medium` for mTLS |
| `ORACLE_DB_WALLET_DIR` | Runtime wallet directory inside the Rust controller container; defaults to `/opt/autographs/wallet` in deploy |
| `TNS_ADMIN` | Runtime Oracle network configuration directory retained for tool compatibility; deployed controller and one-shot smoke containers set this to `/opt/autographs/wallet` |
| `RUST_LOG` | Runtime controller log filter; defaults to `autographs_controller=info,tower_http=info` and can be raised temporarily for debugging |
| `VM_PUBLIC_IP` | Runtime VM public IP; Terraform output can replace this when available |
| `DEPLOY_SSH_USER` | SSH user for deploys, usually `opc` |
| `DEPLOY_PATH` | Optional GitHub variable overriding Ansible's `/opt/autographs` host deployment root across freeze, deploy, unfreeze, rollback, and image cleanup. Leave unset for normal production; a path change on an existing VM needs a coordinated migration. |
| `DEPLOY_SSH_READY_TIMEOUT_SECONDS` | Maximum time deploy waits for SSH after VM creation or replacement; defaults to `900` |
| `DEPLOY_SSH_READY_INTERVAL_SECONDS` | Poll interval while waiting for SSH readiness; defaults to `10` |
| `AUTOGRAPHS_DOMAIN` | Public hostname served by Caddy with automatic TLS; defaults to `autographs.jetsaredim.net` |
| `GHCR_CONTROLLER_IMAGE_REPOSITORY` | GHCR controller image path, for example `ghcr.io/jetsaredim/autographs/controller` |
| `GHCR_CLEANUP_RETAIN_TAGGED` | Number of newest GHCR controller image versions to retain during scheduled/manual cleanup; defaults to `10` |
| `GHCR_CLEANUP_RETAIN_SEMVER_TAGGED` | Number of newest semver-tagged GHCR controller image versions to retain during scheduled/manual cleanup; defaults to `GHCR_CLEANUP_RETAIN_TAGGED` |
| `GHCR_CLEANUP_MIN_AGE_DAYS` | Minimum image age before GHCR cleanup can delete it; defaults to `7` |
| `GHCR_CLEANUP_PROTECTED_TAGS` | Optional comma-separated immutable tags that both GHCR and VM-local cleanup must preserve |
| `AUTOGRAPHS_LOCAL_IMAGE_RETAIN_COUNT` | Number of newest local controller images to retain on the runtime VM during scheduled/manual cleanup; defaults to `3` |
| `AUTOGRAPHS_CONTROLLER_DB_PROVIDER` | Deploy-time value must be `oracle`; `local` is only for direct local controller runs |
| `AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS` | Seconds between production Oracle heartbeat SQL commands; defaults to `86400`; set `0` to disable |
| `AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER` | Deploy-time value must be `oci-instance-principal`; `local` is only for direct local controller runs |

The deploy workflow intentionally codifies the single-region tenancy defaults: runtime region and home region are both `us-ashburn-1`, the Terraform state bucket is `autographs-tf-state`, and the runtime state object key is `envs/prod/terraform.tfstate`. Keep operator-supplied non-secret deployment coordinates visible as GitHub Variables so deploy behavior can be audited without opening secrets. Terraform-owned Vault secret OCIDs are discovered by the runtime root and passed directly to Ansible rather than duplicated in GitHub Variables.

## Local Operator Values

For local runs, copy `.env.example` to an untracked file such as `.env.local`, then copy both Terraform examples:

- `infra/terraform/tenancy/environments/prod/terraform.tfvars.example` to `infra/terraform/tenancy/environments/prod/terraform.tfvars`
- `infra/terraform/environments/prod/terraform.tfvars.example` to `infra/terraform/environments/prod/terraform.tfvars`

Local Terraform uses OCI tenancy identity, compartment, runtime VM, database, media bucket, Object Storage backend, DNS, and SSH inputs. GitHub Actions uses equivalent `TF_VAR_*` environment variables and writes `OCI_PRIVATE_KEY_PEM` to a temporary private key file for Terraform. The deployment key is not a controller runtime credential; the running controller uses instance-principal authentication for OCI media access.

## Data Services

Terraform manages the private OCI Object Storage media bucket and Oracle Autonomous Database Free without resource-creation switches. Supply the real `media_bucket_namespace` before the first full apply. The explicit `bootstrap_secrets` workflow operation makes a targeted first apply of only the Vault, key, and three secrets before ADB exists.

Fresh environments first run `deploy.yml` with `operation=bootstrap_secrets`
from `main` after configuring the reviewer-gated `secret-audit` environment and
both manual GitHub Secrets. The targeted apply creates the Vault resources
without ADB. OCI generates the Oracle database password
with the `DBAAS_DEFAULT_PASSWORD` passphrase template and a length of 30, so its
generated version 1 is immediately usable. Only the Oracle wallet password and
admin password hash receive intentionally invalid Terraform bootstrap content;
the bootstrap operation replaces only those exact version-1 placeholders from
the two GitHub Secrets, then verifies their `CURRENT` versions advanced to at
least 2. Any other mismatch fails without overwriting Vault. The generated database
password must never be populated or overwritten out of band. Terraform derives
readiness from non-secret Vault version metadata: until the generated password
exists and both manual secrets have advanced, an ADB request fails its Terraform
precondition. Normal deployment compares both manual Vault bundles with GitHub
before planning and fails on mismatch. Secret contents do not enter Terraform
inputs or state.

The runtime Terraform root owns the regional Vault, software key, and controller
secrets and passes the managed Oracle database password secret OCID to the
Autonomous Database resource as `secret_id`. ADB obtains that OCID through an
exact-name lookup of the already-created secret. The managed secret discovers
the existing ADB by its exact display and DB names for its OCI rotation target;
the deploy workflow runs a guarded second plan after initial ADB creation to
attach scheduled rotation. Only
the database password has automatic generation and scheduled `P90D` rotation;
the wallet password and admin hash remain operator-managed. Neither the
pull-request plan nor the merge-triggered apply workflow receives the plaintext
database ADMIN password. This control-plane use is distinct from the running controller:
the controller receives the same secret OCID and retrieves the current secret
bundle with its VM instance principal during startup. A successful Terraform
plan proves the lookup and provider planning path only; a controlled apply and
fresh controller connection are required to prove that OCI can consume the
secret for the database update. The manually applied tenancy root grants the
deploy identity compartment-scoped management of vaults, keys, and the secret
family. This permission is limited to the dedicated Autographs project
compartment rather than the tenancy and lets the deployment root own the full
lifecycle of those regional resources.

Runtime containers receive database and media coordinates through Ansible-managed environment files consumed by Podman quadlets, not committed files. Ansible writes VM-local env files under its deployment root (`/opt/autographs/env` by default); the optional `DEPLOY_PATH` GitHub variable overrides that host root. Container-internal wallet paths remain `/opt/autographs/wallet`. Keep wallet material, wallet passwords, real database passwords, operator tokens, and API signing material out of git. Multiline API signing keys are delivered as protected VM files rather than flattened environment values.

When production persistence uses Oracle, the controller starts a read-only heartbeat that connects and runs `select 1 from dual` once after startup, then repeats on the configured interval. This keeps Always Free Autonomous Database inactivity tracking fresh without writing synthetic catalog records and gives operators an immediate success or failure log after a deploy or reboot.

## Runtime Image Contract

Release-please manages repository versions in `version.txt` and its manifest. Merging its Release PR deploys production changes; ordinary merges only accumulate changes. A repository-installed GitHub App mints the short-lived installation token used to create/update Release PRs, tags, and Releases; its Client ID is stored in `RELEASE_PLEASE_APP_CLIENT_ID` and its PEM private key in `RELEASE_PLEASE_APP_PRIVATE_KEY`. Controller-changing releases publish their semantic image tag to GHCR. Infrastructure-only releases reuse the active controller tag. Each Release records the controller digest, which is checked before deploying by tag. Ansible records repository/controller versions in the VM environment and checks runtime health. Configure the GitHub `production` environment deployment-branch rule to allow only `main`; the workflow independently rejects manual production dispatches from any other ref. See [release management](release-management.md) for App setup, recovery, and retention.

Scheduled VM-local cleanup removes unused Podman images while preserving the active and previous controller images, container-referenced images, protected tags, and the configured newest image count. Scheduled GHCR cleanup is inventory-only; remote deletion requires an explicit manual workflow operation after review and preserves the active and previous controller tags and digests. Repository `v*` tags may exist without a matching GHCR image when a release did not change controller image inputs.

The Ansible deploy role also maintains a 2 GiB `/.swapfile` with
`vm.swappiness=20`, installs runtime packages, opens HTTP/HTTPS, and masks
unnecessary systemd services. This gives the Always Free VM enough headroom for
deploy churn and one-off smoke/admin scripts without changing the compute
shape. The current swap file remains plaintext until the separately invoked C4
encrypted boot-only-key swap conversion is implemented and proven; this
core/Kdump slice does not mutate swap.

The generated controller Quadlet sets `LimitCORE=0`. Host configuration writes
`/etc/systemd/coredump.conf.d/99-autographs.conf` with `Storage=none` and
`ProcessSizeMax=0`, and the deploy role stops and masks `kdump.service`. It also
removes `crashkernel` from installed boot entries when present, but never
reboots automatically. Deployment fails unless the restarted controller's
generated service limit and live process soft/hard core limits are all zero.
The configuration contract is still not a complete running-kernel proof: after
deployment, an operator-approved reboot and the checked script in
[deployment-runbook.md](deployment-runbook.md#core-and-kdump-persistence-gate)
must confirm the running process limit, coredump policy, Kdump state,
`kexec_crash_loaded=0`, and absence of `crashkernel` from `/proc/cmdline`.

## Private Controller Contract

The Rust controller is a private runtime service. GitHub-hosted deploys build and deploy controller images, render the controller-only runtime environment, and verify the deployed controller reports persistent Oracle and OCI instance-principal providers before the workflow succeeds.

Runtime controller settings:

| Variable | Classification | Purpose |
|----------|----------------|---------|
| `AUTOGRAPHS_CONTROLLER_BIND_ADDR` | runtime coordinate | Controller listener; defaults to `0.0.0.0:8080` |
| `AUTOGRAPHS_REPO_VERSION` | release metadata | Repository release deployed to this VM |
| `AUTOGRAPHS_CONTROLLER_VERSION` | release metadata | Deployed controller image semver version |
| `AUTOGRAPHS_CONTROLLER_IMAGE` | release metadata | Deployed semver-tagged controller image reference |
| `AUTOGRAPHS_CONTROLLER_DIGEST` | release metadata | Verified active controller registry digest |
| `AUTOGRAPHS_PREVIOUS_CONTROLLER_IMAGE` | release metadata | Previous known-good controller image reference retained for rollback |
| `AUTOGRAPHS_PREVIOUS_CONTROLLER_VERSION` | release metadata | Previous controller semantic version |
| `AUTOGRAPHS_PREVIOUS_CONTROLLER_DIGEST` | release metadata | Previous controller registry digest |
| `AUTOGRAPHS_SOURCE_REVISION` | release metadata | Source revision of the deployed repository release |
| `AUTOGRAPHS_CONTROLLER_DB_PROVIDER` | runtime coordinate | Deploy-time value must be `oracle`; `local` is only for direct local controller runs |
| `AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS` | runtime coordinate | Seconds between lightweight Oracle heartbeat SQL commands in production persistence mode; defaults to `86400`; set `0` to disable |
| `AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER` | runtime coordinate | Deploy-time value must be `oci-instance-principal`; `local` is only for direct local controller runs |
| `AUTOGRAPHS_CONTROLLER_LOCAL_MEDIA_ROOT` | local/staged coordinate | Local private-media path used only when the controller media provider is `local` |
| `AUTOGRAPHS_PUBLIC_ORIGIN` | runtime coordinate | Canonical HTTPS origin used for browser mutation checks |
| `AUTOGRAPHS_ADMIN_SECURE_COOKIES` | runtime coordinate | Keep `true` in deployment; `false` is an explicit local HTTP exception |
| `AUTOGRAPHS_ADMIN_PASSWORD_HASH` | runtime secret | Argon2 single-admin password hash |
| `AUTOGRAPHS_ADMIN_PASSWORD` | local-development secret only | Optional local plaintext shortcut; never deploy it |
| `AUTOGRAPHS_OPERATOR_API_TOKEN` | runtime/operator secret | Optional CLI-friendly bearer token for non-management diagnostics such as protected auth probes; item create/edit/publish operations use `/admin/api/login` |
| `ORACLE_DB_PASSWORD_VAULT_SECRET_ID` | runtime coordinate | OCI Vault secret OCID for the Oracle runtime password |
| `ORACLE_DB_WALLET_PASSWORD_VAULT_SECRET_ID` | runtime coordinate | OCI Vault secret OCID for the Oracle wallet password |
| `AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID` | runtime coordinate | OCI Vault secret OCID for the admin password hash |
| `AUTOGRAPHS_STATIC_RELEASE_ROOT` | runtime coordinate | Static root containing `releases/`, `failed/`, and the active `current` pointer |
| `AUTOGRAPHS_STATIC_CURRENT_LINK` | runtime coordinate | Active static release pointer |
| `AUTOGRAPHS_STATIC_PROMOTED_RELEASE_RETAIN_COUNT` | runtime coordinate | Number of promoted releases retained after successful publish; defaults to `5` |
| `AUTOGRAPHS_STATIC_FAILED_CANDIDATE_RETAIN_COUNT` | runtime coordinate | Number of failed candidates retained for diagnostics; defaults to `1` |
| `AUTOGRAPHS_PUBLISH_MODE` | runtime coordinate | Defaults to incremental publishing |
| `OCI_AUTH_MODE` | runtime coordinate | Controller value must be `instance_principal` when media provider is `oci-instance-principal` |
| `OCI_MEDIA_NAMESPACE` | runtime coordinate | Object Storage namespace containing the private media bucket |
| `OCI_MEDIA_BUCKET_NAME` | runtime coordinate | Private media bucket name |

The runtime dynamic group matches compute instances in the project compartment, which keeps tenancy bootstrap independent of runtime instance IDs. Its IAM policy grants bucket discovery and media-bucket-scoped object access so the private controller can use OCI instance principals for Object Storage without long-lived S3 Customer Secret credentials.

The runtime Terraform root owns a default OCI Vault, a software-protected runtime secrets key, and three Terraform-managed controller secrets. `oracle_db_password` is generated and rotation-managed by OCI; its generated version 1 is usable and must never be populated or overwritten out of band. Only `oracle_db_wallet_password` and `admin_password_hash` contain non-sensitive, intentionally invalid version-1 Terraform bootstrap content. The approved bootstrap operation replaces those exact placeholders from GitHub Secrets with `CURRENT` versions 2 or later; Terraform ignores later content changes so real values never enter Terraform configuration or state. Normal deployment compares but never overwrites. The tenancy root retains IAM ownership and generates the runtime secret-bundle policy from deterministic secret names, so the runtime dynamic group can read only the curated Autographs runtime secrets without passing secret OCIDs between Terraform stacks. Secret contents must not enter Terraform inputs, state, GitHub repository variables, or committed files.

The controller media adapter uses native OCI Object Storage requests signed with runtime instance-principal credentials. The runtime Terraform root owns the regional Vault, key, and secrets through the deploy identity's compartment-scoped `manage vaults`, `manage keys`, and `manage secret-family` permissions. The controller startup path resolves those `*_VAULT_SECRET_ID` environment variables through OCI Vault Secret Retrieval before normal configuration loads. Each successful load logs only the safe logical secret kind, numeric OCI bundle version, and requested `CURRENT` stage. For the Oracle database password only, the shared credential provider retains the Vault OCID and numeric bundle version and, after an exact `ORA-01017` connection failure, serializes one `CURRENT`-stage Vault refresh per failed credential generation and retries that connection once. Successful refresh and concurrent-reuse messages correlate the controller-local generation with the durable Vault version. These messages are emitted only on load or refresh and never include secret contents, secret OCIDs, user-defined version names, or customer metadata. Unrelated failures, session initialization, queries, and application operations are not retried; wallet and admin-hash changes still require a controller restart. OCI generates and rotates only the database password, coordinates each rotation with the targeted ADB, and keeps the stable secret OCID unchanged; wallet and admin-hash rotation remain manual. When a supported Vault secret OCID is configured, deploy renders the matching direct secret value blank and the controller treats Vault as authoritative even if a direct value is accidentally present. Direct secret env vars remain supported only as a transition fallback, but production should use the three Vault secret OCIDs and must not supply a direct `ORACLE_DB_PASSWORD` for the OCI-generated path. `AUTOGRAPHS_ADMIN_PASSWORD` is not moved to Vault for production; the production path is hash-only, so operators must retain the original admin password outside this repo or rotate it by generating and storing a new Argon2 hash. Do not create new Terraform-managed IAM users or Customer Secret keys for controller media or runtime configuration access.

The static release root and current pointer live on the runtime VM. Public artifacts are generated inside the OCI boundary from Oracle metadata and private originals. GitHub-hosted jobs may receive deploy secrets needed to render the private controller environment, but must not publish generated static release content outside the VM.

Collection management has one supported admin authentication path: `POST /admin/api/login` validates the configured single-admin password or Argon2 hash and returns HTTP-only session cookies. Browser collection-management requests to item, image, publication, and publish routes must use that session cookie plus the configured same-origin `Origin` or `Referer`. `AUTOGRAPHS_ADMIN_PASSWORD_HASH` is the production credential source. `AUTOGRAPHS_ADMIN_PASSWORD` exists only for local development and ignored operator smoke tests; do not deploy it. `AUTOGRAPHS_OPERATOR_API_TOKEN` remains a compatibility secret for non-management diagnostics and must not be used as a create/edit/publish path.

Static release retention is controller-owned. `AUTOGRAPHS_STATIC_PROMOTED_RELEASE_RETAIN_COUNT` defaults to `5` and prunes older promoted releases after successful publishes. `AUTOGRAPHS_STATIC_FAILED_CANDIDATE_RETAIN_COUNT` defaults to `1` and keeps only the newest failed candidate for diagnosis while preserving the last valid `current` release.

The operator-run live static publish smoke also uses these VM-local values:

| Variable | Classification | Purpose |
|----------|----------------|---------|
| `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE` | operator gate | Must be exactly `true` before the credential-gated smoke mutates live data |
| `AUTOGRAPHS_CONTROLLER_BASE_URL` | private runtime coordinate | Controller URL reachable from the one-shot smoke container |
| `AUTOGRAPHS_STATIC_PREVIEW_BASE_URL` | private runtime coordinate | Caddy static root, normally `http://autographs-caddy:8081` |

The generated static release replaces the old dynamic public routes, `/api/catalog/*`, app-mediated image streaming, and `/api/operator/*`. The retired Node app source and toolchain are no longer part of the active repository tree.
