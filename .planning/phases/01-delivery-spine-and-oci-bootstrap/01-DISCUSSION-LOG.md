# Phase 1: Delivery Spine and OCI Bootstrap - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `01-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-04-18
**Phase:** 01-Delivery Spine and OCI Bootstrap
**Areas discussed:** OCI bootstrap ownership, repository organization, runtime deployment shape, CI/CD authentication and config, Terraform state, Phase 1 done boundary

---

## OCI Bootstrap Ownership

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal manual bootstrap, then codify everything | Console/manual setup only to unblock Terraform; import or codify all long-term resources afterward | ✓ |
| Keep some IAM/compartment setup manual long term | Faster short term, but not code-owned | |

**User's choice:** Keep manual work to the absolute minimum; long-term all OCI resources should be managed through Terraform, and anything created manually to bootstrap should be imported into state.
**Notes:** User explicitly wants compartments and "literally everything" managed through code once Terraform is operational.

---

## Repository Organization

| Option | Description | Selected |
|--------|-------------|----------|
| Single repo with directory separation | Keep infra, app, deploy/runtime assets, workflows, and docs together in one repository | ✓ |
| Split infra and app into separate repos | Separate release and ownership boundaries at the repo level | |

**User's choice:** Keep the project in a single repository and separate concerns by directories instead of multiple repos.
**Notes:** Split repos should only be reconsidered if the OCI infrastructure later becomes a reusable/shared foundation beyond this project.

---

## Runtime Deployment Shape

| Option | Description | Selected |
|--------|-------------|----------|
| `nginx` on host + one `Next.js` container | Hybrid host/container setup | |
| Fully containerized two-container runtime | One `nginx` container proxying to one `Next.js` app container on a single OCI VM | ✓ |
| Host-managed app process | No Docker boundary for the app runtime | |

**User's choice:** Fully containerized runtime on one OCI VM with one `nginx` container proxying to one `Next.js` app container.
**Notes:** Clarified that `nginx` is the front door for incoming traffic, not the component that accesses Object Storage.

---

## CI/CD Authentication and Config Contract

| Option | Description | Selected |
|--------|-------------|----------|
| Static OCI signing keys as the long-term model | Simplest immediate path, but easy to ossify | |
| Federated/short-lived auth from day one | Stronger security posture, higher bootstrap complexity | |
| Start with signing keys, preserve a migration path | Practical Phase 1 bootstrap with an intentional future auth seam | ✓ |

**User's choice:** Start with OCI API signing keys in GitHub Secrets, but structure the workflows, Terraform, and documentation so auth can move later to a federated or short-lived model.
**Notes:** Best-practice follow-up established that sensitive values belong in GitHub Secrets, while non-sensitive deploy configuration should live in repo-managed files and GitHub Variables.

---

## Terraform State

| Option | Description | Selected |
|--------|-------------|----------|
| Store state in GitHub Secrets or similar | Treat state as another CI secret blob | |
| Remote state in OCI Object Storage | Use Terraform remote backend in the tenancy | ✓ |
| Keep local state longer term | Simplest bootstrap, poor long-term fit | |

**User's choice:** Store Terraform state in OCI Object Storage using the `oci` backend, with bucket versioning enabled.
**Notes:** Local state is only acceptable as a temporary bootstrap mechanism until the remote backend exists and state has been migrated.

---

## Phase 1 Done Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Strict spine only | Infrastructure, contracts, and deploy path documented/wired without a live app proof | |
| Spine plus proof-of-life | Includes a minimal deployed `Next.js` shell or health page running through `nginx` on OCI | ✓ |
| Larger vertical slice | Pulls later app functionality into Phase 1 | |

**User's choice:** Phase 1 should include the spine plus a deployed proof-of-life app path.
**Notes:** This gives end-to-end validation without pulling Oracle catalog functionality or collection features into Phase 1.

---

## the agent's Discretion

- Exact directory names
- Exact secret and variable names
- Exact proof-of-life route/page shape
- Exact container orchestration implementation details on the VM

## Deferred Ideas

None.
