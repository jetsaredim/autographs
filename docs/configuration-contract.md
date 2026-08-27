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
| `ADB_ADMIN_PASSWORD` | deploy workflow / Terraform | Initial Oracle Autonomous Database ADMIN password when database creation is enabled |
| `ORACLE_DB_PASSWORD` | deploy workflow / runtime | Runtime database password passed to the Rust controller container |
| `ORACLE_DB_WALLET_ZIP_BASE64` | deploy workflow / runtime | Base64-encoded ADB wallet zip used for mTLS connections |
| `ORACLE_DB_WALLET_PASSWORD` | deploy workflow / runtime | Wallet download password required by the `oracledb` driver when `ORACLE_DB_WALLET_DIR` is set |
| `AUTOGRAPHS_OPERATOR_API_TOKEN` | runtime | Optional compatibility token for non-management diagnostics; not a collection-management credential |
| `AUTOGRAPHS_ADMIN_PASSWORD_HASH` | Rust controller runtime | Argon2 hash for the single-admin browser login |

The current OCI authentication path uses OCI API signing keys because that is the initial locked decision. Treat this as a replaceable auth adapter: the workflow isolates these inputs so a future OIDC or other short-lived auth path can replace OCI API signing keys without redesigning the controller image build, Terraform, or VM deployment steps.

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
| `OCI_CREATE_AUTONOMOUS_DATABASE` | Optional toggle for creating the Oracle Autonomous Database; defaults to `false` until credentials are ready |
| `OCI_AUTONOMOUS_DATABASE_NAME` | Short Oracle DB name used by wallet aliases and connection strings; defaults to `autographsdb` |
| `OCI_AUTONOMOUS_DATABASE_DISPLAY_NAME` | Display name for the Oracle Autonomous Database; defaults to `autographs-prod-adb` |
| `OCI_AUTONOMOUS_DATABASE_IS_MTLS_CONNECTION_REQUIRED` | Optional ADB mTLS toggle; defaults to `true` for the wallet-based production path |
| `OCI_CREATE_MEDIA_BUCKET` | Optional toggle for creating the private media Object Storage bucket; defaults to `false` until the namespace is confirmed |
| `OCI_MEDIA_BUCKET_NAME` | Private Object Storage bucket for autograph images; defaults to `autographs-media-prod` |
| `OCI_MEDIA_NAMESPACE` | Object Storage namespace for the private media bucket; usually matches `OCI_OBJECT_STORAGE_NAMESPACE` |
| `ORACLE_DB_USER` | Runtime database user for the Rust controller container; defaults to `ADMIN` for the first bootstrap path |
| `ORACLE_DB_CONNECT_STRING` | Runtime Oracle connect alias or descriptor; use the wallet alias such as `autographsdb_medium` for mTLS |
| `ORACLE_DB_WALLET_DIR` | Runtime wallet directory inside the Rust controller container; defaults to `/opt/autographs/wallet` in deploy |
| `TNS_ADMIN` | Runtime Oracle network configuration directory retained for tool compatibility; deployed controller and one-shot smoke containers set this to `/opt/autographs/wallet` |
| `RUST_LOG` | Runtime controller log filter; defaults to `autographs_controller=info,tower_http=info` and can be raised temporarily for debugging |
| `VM_PUBLIC_IP` | Runtime VM public IP; Terraform output can replace this when available |
| `DEPLOY_SSH_USER` | SSH user for deploys, usually `opc` |
| `DEPLOY_PATH` | Directory on the VM that stores Ansible-managed env, Caddy config, wallet, and secret files |
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

The deploy workflow intentionally codifies the single-region tenancy defaults: runtime region and home region are both `us-ashburn-1`, the Terraform state bucket is `autographs-tf-state`, and the runtime state object key is `envs/prod/terraform.tfstate`. Keep non-secret deployment coordinates visible as GitHub Variables so deploy behavior can be audited without opening secrets.

## Local Operator Values

For local runs, copy `.env.example` to an untracked file such as `.env.local`, then copy both Terraform examples:

- `infra/terraform/tenancy/environments/prod/terraform.tfvars.example` to `infra/terraform/tenancy/environments/prod/terraform.tfvars`
- `infra/terraform/environments/prod/terraform.tfvars.example` to `infra/terraform/environments/prod/terraform.tfvars`

Local Terraform uses OCI tenancy identity, compartment, runtime VM, database, media bucket, Object Storage backend, DNS, and SSH inputs. GitHub Actions uses equivalent `TF_VAR_*` environment variables and writes `OCI_PRIVATE_KEY_PEM` to a temporary private key file for Terraform. The deployment key is not a controller runtime credential; the running controller uses instance-principal authentication for OCI media access.

## Data Services

Terraform defines the end-state Oracle Autonomous Database Free metadata store and the private OCI Object Storage media bucket. Both are guarded by explicit creation toggles so the live deployment does not accidentally request paid or tenancy-specific resources before the operator has supplied the correct namespace, ADMIN password, and runtime connection values.

Runtime containers receive database and media coordinates through Ansible-managed environment files consumed by Podman quadlets, not committed files. The deploy workflow writes VM-local env files under `${DEPLOY_PATH}/env`; keep wallet material, wallet passwords, real database passwords, operator tokens, and API signing material out of git. Multiline API signing keys are delivered as protected VM files rather than flattened environment values.

When production persistence uses Oracle, the controller starts a read-only heartbeat that connects and runs `select 1 from dual` once after startup, then repeats on the configured interval. This keeps Always Free Autonomous Database inactivity tracking fresh without writing synthetic catalog records and gives operators an immediate success or failure log after a deploy or reboot.

## Runtime Image Contract

Deployments publish the prebuilt Rust controller image to `ghcr.io` with the repo semver tag tracked in `VERSION` and mirrored into `.release-status.json`, for example `v0.8.4`. The VM does not build application code. Ansible pulls the semver-tagged controller image published by GitHub Actions, records the repo and deployed controller versions in the VM-local environment, installs systemd quadlets for the private controller, shared static volume, and Caddy containers on a dedicated Podman network, retires the old Node service if present, restarts affected services, and checks the Caddy-fronted static release plus `/admin/api/health` before the workflow succeeds.

Scheduled/manual image cleanup handles old GHCR versions and unused VM-local Podman images while preserving the deployed controller semver tag, `latest`, protected tags, and the configured newest image counts. Git repo `v*` tags may exist without a matching GHCR image when a merge did not change controller image inputs.

The Ansible deploy role also maintains a 2 GiB `/.swapfile` with `vm.swappiness=20`, installs runtime packages, opens HTTP/HTTPS, and masks unnecessary systemd services. This gives the Always Free VM enough headroom for deploy churn and one-off smoke/admin scripts without changing the compute shape.

## Private Controller Contract

The Rust controller is a private runtime service. GitHub-hosted deploys build and deploy controller images, render the controller-only runtime environment, and verify the deployed controller reports persistent Oracle and OCI instance-principal providers before the workflow succeeds.

Runtime controller settings:

| Variable | Classification | Purpose |
|----------|----------------|---------|
| `AUTOGRAPHS_CONTROLLER_BIND_ADDR` | runtime coordinate | Controller listener; defaults to `0.0.0.0:8080` |
| `AUTOGRAPHS_REPO_VERSION` | release metadata | Latest repo semver version from `VERSION` |
| `AUTOGRAPHS_CONTROLLER_VERSION` | release metadata | Deployed controller image semver version |
| `AUTOGRAPHS_CONTROLLER_IMAGE` | release metadata | Deployed semver-tagged controller image reference |
| `AUTOGRAPHS_SOURCE_REVISION` | release metadata | Source revision that triggered the version workflow |
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

The tenancy bootstrap root also creates a default OCI Vault, a software-protected runtime secrets key, and Terraform-managed runtime secret shells with non-sensitive, intentionally invalid bootstrap content for the controller secrets that will move to Vault: Oracle database password, Oracle wallet password, and admin password hash. Terraform creates the initial manual secret versions but ignores subsequent secret-content changes so operator-managed OCI CLI rotations remain authoritative and real values never enter Terraform configuration or state. The runtime secret-bundle policy is generated from those Terraform-managed secret OCIDs, so the runtime dynamic group can read only the curated Autographs runtime secrets without passing secret IDs between Terraform stacks. Operators must replace the bootstrap versions outside Terraform before switching deploy to the Vault OCIDs; secret contents must not enter Terraform inputs, state, GitHub repository variables, or committed files.

The controller media adapter uses native OCI Object Storage requests signed with runtime instance-principal credentials. The controller startup path can also resolve `*_VAULT_SECRET_ID` environment variables through OCI Vault Secret Retrieval before normal configuration loads. When a supported Vault secret OCID is configured, deploy renders the matching direct secret value blank and the controller treats Vault as authoritative even if a direct value is accidentally present. Direct secret env vars remain supported only as a transition fallback, but production should move `ORACLE_DB_PASSWORD`, `ORACLE_DB_WALLET_PASSWORD`, and `AUTOGRAPHS_ADMIN_PASSWORD_HASH` to Vault secret OCIDs. `AUTOGRAPHS_ADMIN_PASSWORD` is not moved to Vault for production; the production path is hash-only, so operators must retain the original admin password outside this repo or rotate it by generating and storing a new Argon2 hash. Do not create new Terraform-managed IAM users or Customer Secret keys for controller media or runtime configuration access.

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
