# Phase 1: Delivery Spine and OCI Bootstrap - Research

**Researched:** 2026-04-18
**Domain:** OCI bootstrap, Terraform remote state, self-hosted Next.js delivery spine
**Confidence:** MEDIUM

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### OCI Bootstrap Ownership
- **D-01:** Manual OCI work should be kept to the absolute minimum required to bootstrap Terraform.
- **D-02:** The long-term operating model is that compartments, IAM, policies, and the rest of the OCI footprint are managed through Terraform.
- **D-03:** If any OCI resources must be created manually to get Terraform running, they should be imported into Terraform state so code becomes the source of truth afterward.

### Repository Organization
- **D-04:** Keep the project in a single repository rather than splitting infra and app/runtime into separate repos.
- **D-05:** Separate concerns by directories inside the repo, such as Terraform, app code, deploy/runtime assets, workflows, and docs.
- **D-06:** Only split repositories later if the infrastructure grows into a broader reusable OCI foundation beyond this project.

### Runtime Deployment Shape
- **D-07:** The Phase 1 runtime should be fully containerized on a single OCI VM.
- **D-08:** The host runtime shape is one `nginx` container proxying to one `Next.js` app container.
- **D-09:** OCI Object Storage and Oracle remain external managed services; `nginx` is only the incoming reverse proxy and does not access Object Storage directly.

### CI/CD Authentication and Secret Model
- **D-10:** Phase 1 should start with OCI API signing keys in GitHub Secrets for deployment authentication.
- **D-11:** Workflows, Terraform, and documentation must treat GitHub-to-OCI authentication as replaceable so the project can move to short-lived or federated auth later without redesigning the delivery spine.
- **D-12:** Sensitive credentials belong in GitHub Secrets, while non-sensitive deployment configuration should live in versioned repo-managed files and GitHub Variables rather than being hidden in secrets.

### Terraform State Strategy
- **D-13:** Terraform state should live in OCI Object Storage using the Terraform `oci` backend, not in GitHub Secrets.
- **D-14:** The remote state bucket should have versioning enabled.
- **D-15:** Local state is acceptable only as a short bootstrap step until the remote backend exists and state is migrated.

### Phase 1 Done Boundary
- **D-16:** Phase 1 should deliver more than documentation and bootstrap code alone; it must prove the deployment path end to end.
- **D-17:** Phase 1 is complete when the OCI baseline, remote Terraform state, repo organization, config contract, GitHub validation/deploy path, and the containerized runtime are in place, plus a minimal deployed `Next.js` proof-of-life app or health page is running through `nginx` on OCI.

### Claude's Discretion
- Exact directory names inside the single repo, as long as app, infra, deploy/runtime assets, workflows, and docs are clearly separated.
- Exact naming of GitHub secrets and variables, as long as the contract is explicit and consistent.
- Whether the app proof-of-life is a dedicated health route, a minimal landing page, or both.
- Exact container orchestration mechanism on the VM, as long as it preserves the locked two-container runtime shape.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PLAT-01 | Operator can provision the documented OCI baseline for the app using committed infrastructure code plus clearly documented one-time manual bootstrap steps. | Bootstrap sequencing, Terraform `oci` backend setup, import guidance, least-privilege compartment/policy guidance, and artifact list define the exact plan shape. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] [CITED: https://developer.hashicorp.com/terraform/cli/import/usage] [CITED: https://docs.oracle.com/en-us/iaas/Content/Identity/Concepts/policysyntax.htm] [CITED: https://docs.oracle.com/en/cloud/foundation/cloud_architecture/governance/compartments.html] |
| PLAT-02 | GitHub Actions validates the repository on pull requests and can deploy approved infrastructure and application changes on merge to `main`. | GitHub secrets/variables/environment guidance, future OIDC seam guidance, and recommended workflow split define the CI and deploy spine. [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets] [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-variables] [CITED: https://docs.github.com/en/actions/concepts/security/openid-connect] |
| PLAT-03 | Operator can configure the application using an explicit committed environment and secret contract for local and GitHub-based deployment. | Partial backend configuration, repo-managed variable contract, and explicit local-vs-GitHub secret split define the configuration contract the planner should create. [CITED: https://developer.hashicorp.com/terraform/language/backend] [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets] [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-variables] |
</phase_requirements>

## Summary

Phase 1 should be planned as a bootstrap-and-proof phase, not an application feature phase. The fastest safe sequence is: create the minimum OCI identity and state prerequisites manually, migrate immediately to Terraform ownership, stand up one compartment-scoped OCI baseline, scaffold one self-hosted `Next.js` proof-of-life app, and wire GitHub Actions so pull requests validate while merges to `main` can apply infrastructure and roll the VM forward. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] [CITED: https://developer.hashicorp.com/terraform/cli/import/usage] [CITED: https://nextjs.org/docs/app/guides/self-hosting]

The biggest planning mistake would be treating this as “set up everything OCI-related.” The official docs make the key constraints pretty clear: the Terraform backend bucket must already exist, bucket versioning is strongly recommended, OCI policies should be compartment-scoped instead of root-scoped, Next.js self-hosting wants a reverse proxy in front of the app server, and GitHub’s modern direction is short-lived OIDC tokens even if this phase starts on API keys. That means the plan should explicitly model a bootstrap paradox, a future auth seam, and a production-like reverse-proxy runtime from day one. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] [CITED: https://docs.oracle.com/en-us/iaas/Content/Object/Tasks/usingversioning.htm] [CITED: https://docs.oracle.com/en/cloud/foundation/cloud_architecture/governance/compartments.html] [CITED: https://nextjs.org/docs/app/guides/self-hosting] [CITED: https://docs.github.com/en/actions/concepts/security/openid-connect]

**Primary recommendation:** Plan Phase 1 around a two-step Terraform bootstrap (`bootstrap` then `main`), a two-container OCI VM runtime (`nginx` -> `Next.js`), and a two-workflow GitHub spine (`pr.yml` and `deploy.yml`) where GitHub Actions publishes the app image to `ghcr.io`, repo-level secrets and variables form the baseline config contract, and a live proof-of-life deployment is the exit gate. [ASSUMED]

## Standard Stack

### Core
| Library / Tool | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `next` | `16.2.4` published 2026-04-15 [VERIFIED: npm registry] | Proof-of-life web app and long-term full-stack app shell | Self-hosted Node.js and Docker deployment are first-class, and the docs recommend a reverse proxy such as `nginx` in front. [CITED: https://nextjs.org/docs/app/guides/self-hosting] |
| `react` | `19.2.5` published 2026-04-08 [VERIFIED: npm registry] | UI runtime paired with Next.js | Current stable pair for Next.js 16. [VERIFIED: npm registry] |
| `react-dom` | `19.2.5` published 2026-04-08 [VERIFIED: npm registry] | Server/client rendering runtime paired with Next.js | Current stable pair for Next.js 16. [VERIFIED: npm registry] |
| `typescript` | `6.0.3` published 2026-04-16 [VERIFIED: npm registry] | App and script language | Strong contracts matter early because this phase spans app, deploy scripts, env parsing, and Terraform-adjacent automation. [ASSUMED] |
| Terraform CLI | `1.14.x` docs track latest stable line [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] | OCI provisioning, state migration, and resource import | Required for the `oci` backend and for bringing bootstrap-created resources under code ownership. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] [CITED: https://developer.hashicorp.com/terraform/cli/import/usage] |
| OCI Terraform provider `oracle/oci` | `8.8.0` published ~14 days before research [VERIFIED: Terraform Registry search result] | OCI resource management | Official provider for OCI resources. [VERIFIED: Terraform Registry search result] |
| Docker Engine | `29.4.0` available locally [VERIFIED: local env] | Container runtime on the operator machine and target VM | Phase 1 runtime is explicitly a two-container host. [VERIFIED: local env] |
| Docker Compose | `v5.1.3` available locally [VERIFIED: local env] | Multi-container runtime definition for `nginx` and app containers | Docker docs position Compose as the declarative way to run multi-container apps from a single YAML file. [CITED: https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-docker-compose/] |

### Supporting
| Library / Tool | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `zod` | `4.3.6` published 2026-01-22 [VERIFIED: npm registry] | Parse and validate env/config contracts | Use in Phase 1 for startup config validation and to keep `.env.example` honest. [ASSUMED] |
| `pino` | `10.3.1` published 2026-02-09 [VERIFIED: npm registry] | Structured startup/deploy/runtime logging | Use for proof-of-life app logs and deploy troubleshooting on the VM. [ASSUMED] |
| `oracledb` | `6.10.0` published 2025-10-16 [VERIFIED: npm registry] | Oracle driver for later phases | Do not wire it into Phase 1 runtime yet, but plan the repo layout so Phase 2 can add it cleanly. [VERIFIED: npm registry] |
| `@playwright/test` | `4.1.4` published 2026-04-09 [VERIFIED: npm registry] | Optional browser smoke checks later | Keep out of Wave 1 if it slows proof-of-life; Phase 1 can rely on build and HTTP smoke checks first. [ASSUMED] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Docker Compose-managed two-container runtime | Ad hoc `docker run` scripts | Docker docs show Compose is the cleaner declarative fit for multi-container lifecycle, networking, and reruns. [CITED: https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-docker-compose/] |
| OCI API keys in GitHub Secrets for Phase 1 | GitHub OIDC from day one | OIDC is the stronger modern posture, but Phase 1 is explicitly locked to start with signing keys; plan a swappable auth module so later migration is easy. [CITED: https://docs.github.com/en/actions/concepts/security/openid-connect] |
| Two-step Terraform bootstrap (`local` then remote `oci` backend) | Trying to start directly on remote backend | The backend bucket must already exist, so direct remote init creates a chicken-and-egg problem. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] [ASSUMED] |

**Installation:**
```bash
npm install next react react-dom zod pino
npm install -D typescript @types/node
```

**Version verification:** Package versions above were verified from the npm registry during this session. [VERIFIED: npm registry]

## Architecture Patterns

### Recommended Project Structure
```text
apps/
  web/                    # Next.js proof-of-life app and future full-stack app
infra/
  terraform/
    bootstrap/           # Local-state stack for compartment/state prerequisites
    main/                # Remote-state stack for steady-state OCI resources
deploy/
  compose/               # compose.yaml and nginx config
  scripts/               # deploy and smoke scripts
docs/
  operations/            # bootstrap, secrets, deploy, rollback, and VM runbooks
.github/
  workflows/             # pr validation and main deploy workflows
```
This exact naming is a practical recommendation, not a locked requirement. [ASSUMED]

### Pattern 1: Bootstrap Then Migrate
**What:** Create only the minimum OCI resources needed to enable remote state and least-privilege automation, then import or migrate immediately so Terraform becomes the owner. [CITED: https://developer.hashicorp.com/terraform/cli/import/usage]  
**When to use:** At the very start of the phase. [ASSUMED]  
**Why:** The `oci` backend assumes the bucket already exists, and HashiCorp recommends bucket versioning and partial backend configuration. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] [CITED: https://developer.hashicorp.com/terraform/language/backend]  
**Example:**
```hcl
// Source: adapted from HashiCorp backend docs
terraform {
  backend "oci" {}
}
```

### Pattern 2: Split Terraform by Ownership Stage
**What:** Keep `infra/terraform/bootstrap` separate from `infra/terraform/main` so the temporary local-state bootstrap logic does not get tangled with steady-state remote-state infra. [ASSUMED]  
**When to use:** Throughout the whole phase. [ASSUMED]  
**Why:** This gives the planner a clean import/migration boundary and keeps later applies reproducible. [ASSUMED]

### Pattern 3: Two-Container Runtime, One Compose File
**What:** Run one `nginx` container and one `Next.js` app container on the OCI VM using a checked-in Compose file. [CITED: https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-docker-compose/]  
**When to use:** For local proof-of-life parity and production rollout. [ASSUMED]  
**Why:** Docker positions Compose as the declarative way to define multi-container apps, and Next.js recommends a reverse proxy in front of the app server. [CITED: https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-docker-compose/] [CITED: https://nextjs.org/docs/app/guides/self-hosting]  
**Example:**
```yaml
# Source: adapted from Docker Compose and Next.js self-hosting docs
services:
  proxy:
    image: nginx:stable
    ports:
      - "80:80"
    depends_on:
      - web
  web:
    image: ghcr.io/OWNER/REPO/web:${IMAGE_TAG}
    env_file:
      - .env.runtime
```

### Pattern 4: Separate Secrets From Variables
**What:** Put sensitive values in GitHub Secrets and non-sensitive deployment coordinates in repo files or GitHub Variables. [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets] [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-variables]  
**When to use:** For both local bootstrap docs and CI/CD design. [ASSUMED]  
**Why:** This matches the locked phase decisions and the GitHub platform model. [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets] [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-variables]

### Pattern 5: Auth Adapter Boundary
**What:** Wrap OCI auth inputs behind one documented interface used by Terraform and workflows so API keys can later be replaced by OIDC without rewriting the pipeline shape. [ASSUMED]  
**When to use:** From the first deploy workflow onward. [ASSUMED]  
**Why:** GitHub’s current cloud-auth direction is short-lived OIDC tokens, even though Phase 1 intentionally starts on API keys. [CITED: https://docs.github.com/en/actions/concepts/security/openid-connect]

### Sequencing The Planner Should Preserve
1. Create the repo skeleton, app shell, and env contract first so CI has something concrete to validate. [ASSUMED]
2. Perform the manual OCI bootstrap for user/API key, target compartment, and backend prerequisites. Do not use the root compartment for app resources. [CITED: https://docs.oracle.com/en/cloud/foundation/cloud_architecture/governance/compartments.html]
3. Run `infra/terraform/bootstrap` with local state, create the state bucket with versioning, and create the least-privilege deploy identity/policies. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] [CITED: https://docs.oracle.com/en-us/iaas/Content/Object/Tasks/usingversioning.htm]
4. Import any manually created bootstrap resources that must remain code-owned. [CITED: https://developer.hashicorp.com/terraform/cli/import/usage]
5. Migrate Terraform to the remote `oci` backend using partial backend configuration, not hardcoded credentials. [CITED: https://developer.hashicorp.com/terraform/language/backend]
6. Provision the VM baseline and deploy the two-container runtime. [ASSUMED]
7. Add PR validation, then add `main` deploy automation, then prove the live route through `nginx`. [ASSUMED]

### Anti-Patterns to Avoid
- **Root-compartment deployment:** Oracle recommends not using the root compartment for your own OCI resources. [CITED: https://docs.oracle.com/en/cloud/foundation/cloud_architecture/governance/compartments.html]
- **Backend credentials in committed HCL:** Terraform says partial backend config still lands in `.terraform/` locally, so that directory must stay ignored and secrets must stay out of VCS. [CITED: https://developer.hashicorp.com/terraform/language/backend]
- **Assuming GitHub environments always work for private repos:** environment availability depends on the repo plan and visibility. [CITED: https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments]
- **Skipping the reverse proxy:** Next.js recommends a reverse proxy such as `nginx` rather than exposing the app server directly. [CITED: https://nextjs.org/docs/app/guides/self-hosting]
- **Declaring phase complete without a live route:** the phase’s locked done boundary requires a proof-of-life deployment, not just docs and workflows. [CITED: /home/jgreenwa/dev/git/github.com/jetsaredim/autographs/.planning/phases/01-delivery-spine-and-oci-bootstrap/01-CONTEXT.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Terraform remote state coordination | Custom object upload/download scripts for state files | Terraform `oci` backend | It already provides shared remote state, workspaces, and state locking on OCI Object Storage. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] |
| Multi-container startup order and networking | Bespoke shell scripts with many `docker run` flags | Docker Compose | Docker docs position Compose as the declarative multi-container tool with a checked-in YAML source of truth. [CITED: https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-docker-compose/] |
| Secret/variable distribution rules | One giant `.env` copied everywhere | GitHub Secrets + GitHub Variables + committed `.env.example` | GitHub provides first-class separation for sensitive and non-sensitive workflow inputs. [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets] [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-variables] |
| Future auth migration | Rewriting each workflow step later | One auth adapter contract now | GitHub’s OIDC model is the future path, so the planner should isolate auth inputs from the rest of the deploy logic. [CITED: https://docs.github.com/en/actions/concepts/security/openid-connect] [ASSUMED] |
| Manual drift capture | “We’ll remember what was clicked in OCI” | `terraform import` plus docs | HashiCorp documents import specifically for bringing existing resources under Terraform state. [CITED: https://developer.hashicorp.com/terraform/cli/import/usage] |

**Key insight:** The planner should spend effort on sequence and ownership boundaries, not on inventing tooling that Terraform, Docker Compose, and GitHub Actions already provide. [ASSUMED]

## Common Pitfalls

### Pitfall 1: Remote State Bootstrap Paradox
**What goes wrong:** The plan assumes Terraform can create its own backend bucket while already using that backend. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci]  
**Why it happens:** The `oci` backend requires an existing bucket and namespace, so planning can accidentally skip the bootstrap state transition. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci]  
**How to avoid:** Treat `bootstrap` as a separate local-state step, then migrate to remote state immediately after the bucket exists. [ASSUMED]  
**Warning signs:** A single Terraform root tries to create the bucket and initialize the backend in the same first run. [ASSUMED]

### Pitfall 2: Over-Broad OCI Policies
**What goes wrong:** The deploy identity gets tenancy-wide power because it is faster to make the first apply work that way. [CITED: https://docs.oracle.com/en-us/iaas/Content/Identity/Concepts/policysyntax.htm]  
**Why it happens:** OCI policy examples often start broad, and blank-tenancy setup blurs manual admin work with steady-state deploy work. [ASSUMED]  
**How to avoid:** Put app resources in a dedicated compartment and scope policies there instead of the root compartment. [CITED: https://docs.oracle.com/en/cloud/foundation/cloud_architecture/governance/compartments.html]  
**Warning signs:** Policy statements say `manage all-resources in tenancy` for routine CI jobs. [CITED: https://docs.oracle.com/en-us/iaas/Content/Identity/Concepts/policysyntax.htm]

### Pitfall 3: Hardcoded Backend/Auth Material
**What goes wrong:** Sensitive backend or OCI credentials end up in HCL, committed config, or reusable shell history. [CITED: https://developer.hashicorp.com/terraform/language/backend]  
**Why it happens:** Partial backend configuration is optional, so teams sometimes take the shortest path and inline credentials. [CITED: https://developer.hashicorp.com/terraform/language/backend]  
**How to avoid:** Keep backend blocks partial, feed secrets at init time, ignore `.terraform/`, and document exactly which inputs are local-only versus GitHub-managed. [CITED: https://developer.hashicorp.com/terraform/language/backend]  
**Warning signs:** `private_key_path`, fingerprints, or OCIDs appear in committed backend files without placeholders. [ASSUMED]

### Pitfall 4: GitHub Environment Plan Surprises
**What goes wrong:** The plan depends on GitHub environment features that are unavailable for the repo’s plan/visibility combination. [CITED: https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments]  
**Why it happens:** GitHub environment protections and secrets have availability limits for private repositories on some plans. [CITED: https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments]  
**How to avoid:** Design the secret/variable contract so repo-level secrets and variables work even if environment features are reduced, then layer environments only if the account supports them. [ASSUMED]  
**Warning signs:** The planner assumes required reviewers or private-repo environment secrets without confirming plan support. [CITED: https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments]

### Pitfall 5: Proof-of-Life That Bypasses Production Shape
**What goes wrong:** The app runs directly on a port during development and the phase is declared complete before the `nginx` -> `Next.js` path is proven. [CITED: https://nextjs.org/docs/app/guides/self-hosting]  
**Why it happens:** `next start` is easy to run alone, so the reverse proxy gets deferred. [CITED: https://nextjs.org/docs/app/guides/self-hosting]  
**How to avoid:** Make the acceptance route run through the same Compose stack and reverse proxy that production will use. [ASSUMED]  
**Warning signs:** The deploy docs end in “curl port 3000” instead of a proxied health check. [ASSUMED]

## Code Examples

Verified patterns from official sources:

### Partial OCI Backend Configuration
```hcl
// Source: https://developer.hashicorp.com/terraform/language/backend
terraform {
  backend "oci" {}
}
```

### OCI Backend Settings File
```hcl
# Source: adapted from https://developer.hashicorp.com/terraform/language/backend/oci
bucket           = "autographs-tf-state"
namespace        = "your-namespace"
region           = "us-chicago-1"
key              = "prod/terraform.tfstate"
workspace_key_prefix = "envs"
```

### Minimal Compose Shape
```yaml
# Source: adapted from https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-docker-compose/
services:
  proxy:
    image: nginx:stable
    depends_on:
      - web
    ports:
      - "80:80"

  web:
    image: ghcr.io/OWNER/REPO/web:${IMAGE_TAG}
```

### Next.js Runtime Scripts
```json
// Source: adapted from https://nextjs.org/docs/app/getting-started/installation
{
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start"
  }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Long-lived cloud secrets as the default CI auth model | OIDC short-lived tokens are the preferred modern direction | Current GitHub Actions security docs as of 2026-04 still position OIDC as the better model. [CITED: https://docs.github.com/en/actions/concepts/security/openid-connect] | Phase 1 should keep API-key auth isolated so later replacement is cheap. [ASSUMED] |
| Ad hoc `docker run` commands for multi-container stacks | Compose-managed multi-container application definitions | Current Docker docs position Compose as the declarative multi-container path. [CITED: https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-docker-compose/] | The planner should create one checked-in Compose source of truth. [ASSUMED] |
| Single flat cloud scope for everything | Compartment-scoped policy boundaries | Current OCI governance docs emphasize compartments as the permission scope and recommend not using root for app resources. [CITED: https://docs.oracle.com/en/cloud/foundation/cloud_architecture/governance/compartments.html] | The planner should treat compartment design as a first-class bootstrap artifact. [ASSUMED] |

**Deprecated/outdated:**
- Root-compartment app deployment is outdated for this phase’s goals because Oracle explicitly recommends not using the root compartment for your own OCI resources. [CITED: https://docs.oracle.com/en/cloud/foundation/cloud_architecture/governance/compartments.html]
- Hardcoding backend credentials in committed Terraform files is outdated because HashiCorp documents partial backend configuration specifically for separating sensitive backend inputs. [CITED: https://developer.hashicorp.com/terraform/language/backend]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Docker Compose is the best concrete VM orchestration mechanism for this phase’s two-container runtime. | Architecture Patterns | Low — the user explicitly left orchestration implementation discretionary; systemd-managed `docker run` is possible but less maintainable. |
| A2 | `ghcr.io` is the selected Phase 1 registry path for the prebuilt app image deploy flow. | Architecture Patterns | Low — this is now a locked planning decision for Phase 1 rather than an unresolved branch. |
| A3 | The exact recommended directory names (`apps/web`, `infra/terraform/bootstrap`, `deploy/compose`) are appropriate for this repo. | Recommended Project Structure | Low — the user cares about separation, not these exact names. |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

## Open Questions (RESOLVED AND APPLIED TO PLANS)

1. **Will deploys pull a prebuilt image or build on the VM?**
   - Resolution: Phase 1 should use a prebuilt-image deploy path. GitHub Actions will build the app image, publish it to `ghcr.io`, and the OCI VM will pull that image during deployment.
   - Why: This keeps the VM runtime lean, makes the delivery spine more reproducible, and gives the deploy workflow a concrete artifact contract instead of a host-build fork.
   - Planning implication: The config contract, deploy workflow, VM deploy script, and runbook must all assume registry-backed pulls rather than on-host image builds.

2. **Are GitHub environments usable for this repo’s plan and visibility?**
   - Resolution: Repo-level GitHub Secrets and GitHub Variables are the baseline contract for Phase 1. GitHub Environments are optional/additive rather than required.
   - Why: This avoids blocking the bootstrap spine on repository-plan or visibility nuances while still leaving room to add environment-specific protections later.
   - Planning implication: Workflow inputs and docs should work without GitHub Environments, with environments documented only as a possible enhancement.

3. **Which OCI region and tenancy naming conventions will the operator use?**
   - Resolution: OCI region, home region, compartment OCIDs, and naming prefixes remain explicit operator-provided inputs rather than hidden assumptions.
   - Why: Always Free and home-region constraints are real, but they should be surfaced through `.tfvars`, backend config, and docs instead of being hard-coded in plans.
   - Planning implication: Terraform variables, backend examples, and bootstrap docs must make these values first-class required inputs.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `node` | Next.js app scaffold and scripts | ✓ [VERIFIED: local env] | `v22.22.2` [VERIFIED: local env] | — |
| `npm` | Package install and script execution | ✓ [VERIFIED: local env] | `10.9.7` [VERIFIED: local env] | — |
| `docker` | Local container build and runtime parity | ✓ [VERIFIED: local env] | `29.4.0` [VERIFIED: local env] | — |
| `docker compose` | Two-container runtime definition | ✓ [VERIFIED: local env] | `v5.1.3` [VERIFIED: local env] | `docker run` scripts, but not recommended. [ASSUMED] |
| `git` | CI/CD and repo workflows | ✓ [VERIFIED: local env] | `2.51.0` [VERIFIED: local env] | — |
| `gh` | Optional GitHub secret/variable setup automation | ✓ [VERIFIED: local env] | `2.46.0` [VERIFIED: local env] | Web UI. [ASSUMED] |
| `openssl` | OCI API signing key generation | ✓ [VERIFIED: local env] | `3.5.3` [VERIFIED: local env] | — |
| `ssh` | VM access and deploy command execution | ✓ [VERIFIED: local env] | `OpenSSH_10.0p2` [VERIFIED: local env] | — |
| `terraform` | OCI provisioning and state migration | ✗ [VERIFIED: local env] | — | No practical fallback if Terraform remains the ownership model. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] |
| `oci` CLI | Helpful for bootstrap inspection and some operator steps | ✗ [VERIFIED: local env] | — | Console plus Terraform can cover most needs. [ASSUMED] |

**Missing dependencies with no fallback:**
- `terraform` — this blocks actual Phase 1 execution and must be handled by an explicit prerequisite checkpoint or manual setup step before any Terraform verification runs. [VERIFIED: local env]

**Missing dependencies with fallback:**
- `oci` CLI — not required if bootstrap docs rely on the OCI Console and Terraform instead. [ASSUMED]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | OCI API signing keys in GitHub Secrets for Phase 1, with an explicit later OIDC seam. [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets] [CITED: https://docs.github.com/en/actions/concepts/security/openid-connect] |
| V3 Session Management | no | No end-user session model should be planned in this phase. [CITED: /home/jgreenwa/dev/git/github.com/jetsaredim/autographs/.planning/REQUIREMENTS.md] |
| V4 Access Control | yes | Dedicated compartment plus compartment-scoped OCI policies instead of tenancy-wide routine deploy access. [CITED: https://docs.oracle.com/en/cloud/foundation/cloud_architecture/governance/compartments.html] [CITED: https://docs.oracle.com/en-us/iaas/Content/Identity/Concepts/policysyntax.htm] |
| V5 Input Validation | yes | `zod`-validated env and deploy contracts. [VERIFIED: npm registry] [ASSUMED] |
| V6 Cryptography | yes | Use OCI/API signing keys, SSH, and OpenSSL; never invent custom crypto or token formats. [VERIFIED: local env] [ASSUMED] |

### Known Threat Patterns for This Phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Over-privileged GitHub deploy identity | Elevation of Privilege | Separate manual bootstrap admin access from steady-state deploy credentials; scope policies to the app compartment. [CITED: https://docs.oracle.com/en/cloud/foundation/cloud_architecture/governance/compartments.html] [CITED: https://docs.oracle.com/en-us/iaas/Content/Identity/Concepts/policysyntax.htm] |
| Secrets leakage through repo or workflow logs | Information Disclosure | Keep sensitive values in GitHub Secrets, keep `.terraform/` ignored, and avoid structured secret blobs when possible. [CITED: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets] [CITED: https://developer.hashicorp.com/terraform/language/backend] |
| State corruption from concurrent applies | Tampering | Use the Terraform `oci` backend with built-in state locking and bucket versioning. [CITED: https://developer.hashicorp.com/terraform/language/backend/oci] [CITED: https://docs.oracle.com/en-us/iaas/Content/Object/Tasks/usingversioning.htm] |
| Direct exposure of the app server | Denial of Service | Put `nginx` in front of `next start` as the public entrypoint. [CITED: https://nextjs.org/docs/app/guides/self-hosting] |

## Sources

### Primary (HIGH confidence)
- https://developer.hashicorp.com/terraform/language/backend/oci - OCI backend behavior, versioning warning, state locking, and required permissions
- https://developer.hashicorp.com/terraform/language/backend - partial backend configuration behavior and `.terraform/` sensitivity
- https://developer.hashicorp.com/terraform/cli/import/usage - import workflow for bringing manual bootstrap resources under Terraform
- https://nextjs.org/docs/app/getting-started/installation - minimum supported Node.js version
- https://nextjs.org/docs/app/guides/self-hosting - reverse proxy recommendation and single-instance self-hosting behavior
- https://docs.oracle.com/en-us/iaas/Content/Object/Tasks/usingversioning.htm - OCI bucket versioning behavior
- https://docs.oracle.com/en/cloud/foundation/cloud_architecture/governance/compartments.html - compartment guidance and root-compartment warning
- https://docs.oracle.com/en-us/iaas/Content/Identity/Concepts/policysyntax.htm - OCI policy syntax and compartment-scoped examples
- https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm - Always Free compute sizing constraints
- https://docs.oracle.com/en-us/iaas/autonomous-database-serverless/doc/autonomous-always-free.html - home-region-only and backup limitations for Always Free Autonomous Database
- https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets - GitHub secret handling model
- https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-variables - GitHub variables model
- https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments - environment availability constraints
- https://docs.github.com/en/actions/concepts/security/openid-connect - future auth replacement target
- https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-docker-compose/ - Compose as declarative multi-container tooling

### Secondary (MEDIUM confidence)
- https://docs.github.com/en/actions/publishing-packages/publishing-docker-images - image publishing path if the planner chooses a prebuilt-image deploy model
- Official npm registry queries run during this session for `next`, `react`, `react-dom`, `typescript`, `zod`, `pino`, `oracledb`, `@playwright/test`
- Official Terraform Registry search result for `oracle/oci` latest provider version

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - package/runtime choices and platform constraints were verified from official docs and registries.
- Architecture: MEDIUM - the bootstrap split, Compose choice, and registry recommendation are practical inferences built on official constraints.
- Pitfalls: HIGH - each critical pitfall maps directly to a documented platform constraint or a straightforward inference from it.

**Research date:** 2026-04-18
**Valid until:** 2026-05-18
