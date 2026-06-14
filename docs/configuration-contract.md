# Configuration Contract

The project uses one explicit contract for local work, GitHub validation, and GitHub deployment. Sensitive values live in repo-level GitHub Secrets. Non-sensitive coordinates live in committed examples or repo-level GitHub Variables. Optional GitHub Environments can add approval gates later, but they are optional and additive, not required for the baseline delivery path.

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

These are repo-level GitHub Secrets for the deployment baseline and Phase 2 data services.

| Secret | Used By | Purpose |
|--------|---------|---------|
| `OCI_CLI_USER_OCID` | deploy workflow | OCI API signing identity user OCID |
| `OCI_TENANCY_OCID` | deploy workflow | OCI tenancy OCID |
| `OCI_FINGERPRINT` | deploy workflow | OCI API signing key fingerprint |
| `OCI_PRIVATE_KEY_PEM` | deploy workflow | OCI API signing private key PEM |
| `DEPLOY_SSH_PRIVATE_KEY` | deploy workflow | SSH private key for the OCI runtime VM |
| `ADB_ADMIN_PASSWORD` | deploy workflow / Terraform | Initial Oracle Autonomous Database ADMIN password when database creation is enabled |
| `ORACLE_DB_PASSWORD` | deploy workflow / app runtime | Runtime database password passed to the Next.js container |
| `ORACLE_DB_WALLET_ZIP_BASE64` | deploy workflow / app runtime | Base64-encoded ADB wallet zip used for mTLS connections |
| `ORACLE_DB_WALLET_PASSWORD` | deploy workflow / app runtime | Optional wallet password for node-oracledb Thin mode mTLS connections |
| `AUTOGRAPHS_OPERATOR_API_TOKEN` | app runtime | Temporary operator token for guarded smoke/mutation endpoints until Phase 5 Rust private controller/static admin seed path replaces or retires the bridge |
| `AUTOGRAPHS_ADMIN_PASSWORD_HASH` | Rust controller runtime | Argon2 hash for the single-admin browser login |

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
| `OCI_CREATE_AUTONOMOUS_DATABASE` | Optional toggle for creating the Oracle Autonomous Database; defaults to `false` until credentials are ready |
| `OCI_AUTONOMOUS_DATABASE_NAME` | Short Oracle DB name used by wallet aliases and connection strings; defaults to `autographsdb` |
| `OCI_AUTONOMOUS_DATABASE_DISPLAY_NAME` | Display name for the Oracle Autonomous Database; defaults to `autographs-prod-adb` |
| `OCI_AUTONOMOUS_DATABASE_IS_MTLS_CONNECTION_REQUIRED` | Optional ADB mTLS toggle; defaults to `true` for the wallet-based production path |
| `OCI_CREATE_MEDIA_BUCKET` | Optional toggle for creating the private media Object Storage bucket; defaults to `false` until the namespace is confirmed |
| `OCI_MEDIA_BUCKET_NAME` | Private Object Storage bucket for autograph images; defaults to `autographs-media-prod` |
| `OCI_MEDIA_NAMESPACE` | Object Storage namespace for the private media bucket; usually matches `OCI_OBJECT_STORAGE_NAMESPACE` |
| `ORACLE_DB_USER` | Runtime database user for the app container; defaults to `ADMIN` for the first bootstrap path |
| `ORACLE_DB_CONNECT_STRING` | Runtime Oracle connect alias or descriptor; use the wallet alias such as `autographsdb_medium` for mTLS |
| `ORACLE_DB_WALLET_DIR` | Runtime wallet directory inside the app container; defaults to `/opt/autographs/wallet` in deploy |
| `AUTOGRAPHS_MEDIA_STORAGE_PROVIDER` | Media adapter mode; `oci` in production, `local` for local smoke work without OCI credentials |
| `AUTOGRAPHS_SMOKE_BASE_URL` | Optional local/operator value used by the data/media smoke script to verify a deployed app-mediated image route |
| `VM_PUBLIC_IP` | Runtime VM public IP; Terraform output can replace this when available |
| `DEPLOY_SSH_USER` | SSH user for deploys, usually `opc` |
| `DEPLOY_PATH` | Directory on the VM that stores Ansible-managed app env, Caddy config, wallet, and secret files |
| `DEPLOY_SSH_READY_TIMEOUT_SECONDS` | Maximum time deploy waits for SSH after VM creation or replacement; defaults to `900` |
| `DEPLOY_SSH_READY_INTERVAL_SECONDS` | Poll interval while waiting for SSH readiness; defaults to `10` |
| `AUTOGRAPHS_DOMAIN` | Public hostname served by Caddy with automatic TLS; defaults to `autographs.jetsaredim.net` |
| `GHCR_IMAGE_REPOSITORY` | Published app image path, for example `ghcr.io/jetsaredim/autographs/app` |
| `GHCR_CLEANUP_RETAIN_TAGGED` | Number of newest GHCR app image versions to retain during scheduled/manual cleanup; defaults to `10` |
| `GHCR_CLEANUP_MIN_AGE_DAYS` | Minimum image age before GHCR cleanup can delete it; defaults to `7` |
| `GHCR_CLEANUP_PROTECTED_TAGS` | Optional comma-separated immutable tags that both GHCR and VM-local cleanup must preserve |
| `AUTOGRAPHS_LOCAL_IMAGE_RETAIN_COUNT` | Number of newest local app/tools images to retain on the runtime VM during scheduled/manual cleanup; defaults to `3` |

The deploy workflow intentionally codifies the single-region tenancy defaults: runtime region and home region are both `us-ashburn-1`, the Terraform state bucket is `autographs-tf-state`, and the runtime state object key is `envs/prod/terraform.tfstate`. `GHCR_IMAGE_REPOSITORY`, cleanup retention/protection settings, `OCI_COMPARTMENT_OCID`, `OCI_AVAILABILITY_DOMAIN`, `OCI_RUNTIME_IMAGE_OCID`, `OCI_RUNTIME_SHAPE`, `OCI_RUNTIME_OCPUS`, `OCI_RUNTIME_MEMORY_GBS`, `OCI_RUNTIME_SSH_PUBLIC_KEYS`, `OCI_OBJECT_STORAGE_NAMESPACE`, data-service toggles, data-service names, SSH readiness timing, and `VM_PUBLIC_IP` are intentionally non-secret deployment coordinates. Keep them visible as GitHub Variables so deploy behavior can be audited without opening secrets.

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
- Autonomous Database and private media bucket toggles, names, and Object Storage namespace
- Object Storage namespace, bucket, and key when initializing the remote backend

GitHub Actions uses equivalent `TF_VAR_*` environment variables and writes `OCI_PRIVATE_KEY_PEM` to a temporary private key file for Terraform. During VM deploy, the same secret is copied to `${DEPLOY_PATH}/secrets/oci_api_key.pem`, mounted read-only into the app container, and exposed to the app as `OCI_PRIVATE_KEY_PATH=/opt/autographs/secrets/oci_api_key.pem`. The multiline PEM is intentionally not written into the Compose `.env` file.

## Phase 2 Data Services

Terraform defines the end-state Oracle Autonomous Database Free metadata store and the private OCI Object Storage media bucket. Both are guarded by explicit creation toggles so the live deployment does not accidentally request paid or tenancy-specific resources before the operator has supplied the correct namespace, ADMIN password, and runtime connection values.

The runtime container receives database and media coordinates through an Ansible-managed environment file consumed by Podman quadlets, not committed files. The deploy workflow writes a VM-local `app.env` under `${DEPLOY_PATH}/env`; keep wallet material, wallet passwords, real database passwords, operator tokens, and API signing material out of git. Multiline API signing keys are delivered as protected VM files rather than flattened environment values. The Phase 2 media adapter supports `AUTOGRAPHS_MEDIA_STORAGE_PROVIDER=local` for local smoke work and `oci` for production Object Storage.

## Optional GitHub Environments

GitHub Environments may be added later for manual approval, deployment history, or environment-specific variable overrides. They are optional GitHub Environments for this baseline. The repo-level contract above remains sufficient for Phase 1 and avoids making the deploy path depend on environment availability or repository plan features.

## Runtime Image Contract

Deployments publish prebuilt app, tools, and Rust controller images to `ghcr.io`
and set their immutable digest references on the VM. The VM does not build
application code. Ansible pulls the exact images published by GitHub Actions,
installs systemd quadlets for the app, private controller, shared static volume,
and Caddy containers on a dedicated Podman network, restarts affected services,
and checks the Caddy-fronted `/health` route before the workflow succeeds.
Scheduled/manual image cleanup handles old GHCR versions and unused VM-local
Podman images while preserving `latest`, protected tags, and the configured
newest image count.

The Ansible deploy role also maintains a 2 GiB `/.swapfile` with `vm.swappiness=20`, installs runtime packages, opens HTTP/HTTPS, and masks unnecessary systemd services. This gives the Always Free VM enough headroom for deploy churn and one-off smoke/admin scripts without changing the compute shape.

## Phase 5 Private Controller Contract

The Rust controller is a private runtime service. GitHub-hosted deploys build
and deploy controller images, render the controller-only runtime environment,
and verify the deployed controller reports persistent Oracle and OCI instance-principal
providers before the workflow succeeds.

Runtime controller settings:

| Variable | Classification | Purpose |
|----------|----------------|---------|
| `AUTOGRAPHS_CONTROLLER_BIND_ADDR` | runtime coordinate | Controller listener; defaults to `0.0.0.0:8080` |
| `AUTOGRAPHS_CONTROLLER_DB_PROVIDER` | runtime coordinate | Deploy-time value must be `oracle`; `local` is only for direct local controller runs |
| `AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER` | runtime coordinate | Deploy-time value must be `oci-instance-principal`; `local` is only for direct local controller runs |
| `AUTOGRAPHS_CONTROLLER_LOCAL_MEDIA_ROOT` | local/staged coordinate | Local private-media path used only when the controller media provider is `local` |
| `AUTOGRAPHS_PUBLIC_ORIGIN` | runtime coordinate | Canonical HTTPS origin used for browser mutation checks |
| `AUTOGRAPHS_ADMIN_SECURE_COOKIES` | runtime coordinate | Keep `true` in deployment; `false` is an explicit local HTTP exception |
| `AUTOGRAPHS_ADMIN_PASSWORD_HASH` | runtime secret | Argon2 single-admin password hash |
| `AUTOGRAPHS_ADMIN_PASSWORD` | local-development secret only | Optional local plaintext shortcut; never deploy it |
| `AUTOGRAPHS_OPERATOR_API_TOKEN` | runtime/operator secret | CLI-friendly bearer token for tunnel and maintenance calls |
| `AUTOGRAPHS_STATIC_RELEASE_ROOT` | runtime coordinate | Static root containing `releases/`, `failed/`, and the active `current` pointer |
| `AUTOGRAPHS_STATIC_CURRENT_LINK` | runtime coordinate | Active static release pointer |
| `AUTOGRAPHS_STATIC_FAILED_CANDIDATE_RETAIN_COUNT` | runtime coordinate | Number of failed candidates retained for diagnostics |
| `AUTOGRAPHS_PUBLISH_MODE` | runtime coordinate | Defaults to incremental publishing |
| `OCI_AUTH_MODE` | runtime coordinate | Controller value must be `instance_principal` when media provider is `oci-instance-principal` |
| `OCI_MEDIA_NAMESPACE` | runtime coordinate | Object Storage namespace containing the private media bucket |
| `OCI_MEDIA_BUCKET_NAME` | runtime coordinate | Private media bucket name |

The runtime dynamic group matches compute instances in the project compartment,
which keeps tenancy bootstrap independent of runtime instance IDs. Its IAM
policy grants bucket discovery and media-bucket-scoped object access so the
private controller can use OCI instance principals for Object Storage without
long-lived S3 Customer Secret credentials.

The controller media adapter uses native OCI Object Storage requests signed with
runtime instance-principal credentials. Do not create new Terraform-managed IAM
users, Vault secrets, or Customer Secret keys for controller media access.

The static release root and current pointer live on the runtime VM. Public
artifacts are generated inside the OCI boundary from Oracle metadata and
private originals. GitHub-hosted jobs may receive deploy secrets needed to
render the private controller environment, but must not publish generated
static release content outside the VM.

The operator-run live static publish smoke also uses these VM-local values:

| Variable | Classification | Purpose |
|----------|----------------|---------|
| `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE` | operator gate | Must be exactly `true` before the credential-gated smoke mutates live data |
| `AUTOGRAPHS_CONTROLLER_BASE_URL` | private runtime coordinate | Controller URL reachable from the one-shot smoke container |
| `AUTOGRAPHS_STATIC_PREVIEW_BASE_URL` | private runtime coordinate | Caddy preview prefix, normally `http://autographs-caddy:8081/current` |

Until the public static cutover passes, the Next.js app, `/api/catalog/*`,
app-mediated image streaming, the Node `/api/operator/*` bridge, and the old
data smoke remain current runtime paths. Retire them together through the
checklist in [static-runtime-runbook.md](static-runtime-runbook.md), not as
independent ad hoc removals.
