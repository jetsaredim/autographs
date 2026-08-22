# Ecosystem Cleanup Plan

## Phase 8 Placement

The original Phase 8 repository cleanup was Plan 08-02 and is complete. It
corrected known repository posture findings and added a stale-document hygiene
gate, but it did not inventory the live VM, consolidate runtime configuration,
remove long-lived credentials, or define broad Rust conventions.

Remaining planned Phase 8 work is:

| Wave | Existing Scope | Cleanup Integration |
|------|----------------|---------------------|
| 5 / Plan 08-05 | Persist image-adjustment metadata in Oracle | Apply SQL/bind rules; split Oracle responsibilities only where characterization tests protect the touched behavior. |
| 6 / Plan 08-06 | Private admin preview and adjustment API/UI | Move route-owned runtime reads into typed config while the route boundary is touched; keep this out if it threatens the media deliverable. |
| 7 / Plan 08-07 | Adjustment-aware publishing and cache invalidation | Add publish-stage timing; use typed error kinds at the touched validation/promotion boundary; do not change execution model without evidence. |
| 8 / Plan 08-08 | Production CDN verification and phase closeout | Verify no secret files or stale smoke artifacts remain and record the runtime inventory disposition ledger. |

The broad cleanup should be a parallel Phase 8 posture workstream with small
PRs, not an unbounded rewrite hidden inside media feature commits. Overlapping
module cleanup is folded into Waves 5-7 only when it directly supports the
touched feature and has characterization coverage.

## Ordered Pull Requests

### C1: Standards and New-Debt Guardrails

**When:** before or alongside Wave 5. **Risk:** low.

- Promote this style guide to `docs/` and extend the existing repository
  validator.
- Block new direct production env reads outside typed config, repeated numeric
  SQL binds, production placeholder/debug macros, and new persistent secret
  sinks.
- Baseline existing violations as debt; CI blocks increases while later PRs
  reduce the baseline to zero.
- Report module size, `Result<T, String>` boundary use, and sync work in async
  modules as review signals rather than blockers.

**Exit:** fixture tests prove each rule; current CI passes; every exception has
an owner and removal condition.

### C2: Runtime Configuration and Credential Consolidation

**When:** before the next production deploy that changes runtime config.
**Risk:** medium.

- Add typed Oracle, OCI, auth, publish, and path configuration under
  `ControllerConfig`; inject it into adapters and routes.
- Consolidate durable non-secret values into one Ansible-owned
  `/opt/autographs/env/controller.env`.
- Remove controller delivery and mounting of the OCI user API private key,
  fingerprint, user OCID, and tenancy OCID. The media adapter already uses the
  instance principal.
- Keep deployment-workflow OCI credentials separate until a future workload
  identity migration is deliberately designed.
- Update the configuration contract from observed consumers, not copied env
  files.

**Exit:** production starts with instance-principal media and Oracle providers;
the live publish smoke passes; no user API key is mounted into the controller.

### C3: OCI Vault Retrieval Proof

**When:** after C2; may run parallel to Waves 5-7. **Risk:** medium.

- Provision one disposable software-protected Vault key and test secret, or use
  an existing approved project Vault.
- Grant the runtime dynamic group read access only to exact secret OCIDs.
- Extract the existing OCI instance-principal signer/client from the media
  adapter and retrieve the disposable secret directly from Rust.
- Record retrieval, restart, denial, redaction, and rotation behavior. Destroy
  the disposable secret after the proof.
- Do not put secret content in Terraform state or GitHub logs.

**Exit:** Spike 004 changes from PARTIAL to VALIDATED, including a negative IAM
test and audit-log evidence.

### C4: Production Secret Cutover

**When:** after C3 and before Phase 8 closeout. **Risk:** high.

- Store the database password, wallet password, wallet PEM, and Oracle network
  files as independently rotatable OCI Secrets. Store the admin password hash
  and retained operator token if still required.
- Replace plaintext env values with secret OCIDs. Fetch scalar secrets directly
  at startup and materialize file-required wallet contents mode `0400` on
  container tmpfs.
- Fail closed without a filesystem fallback. Validate restart and rollback
  using the previously deployed immutable image while old Vault versions remain
  available for the bounded rollback window.
- Run health, Oracle heartbeat, persistence, login, and full static-publish
  smokes before retiring old files.

**Exit:** no password, token, private key, hash, or wallet content persists in
the controller env/secrets directories; live smokes pass after reboot.

### C5: VM Cruft Reconciliation

**When:** inventory before C2; deletion after C4. **Risk:** medium.

- Run Spike 003's redacted collector on the VM and compare it with the declared
  Ansible/runtime contract.
- Give every env file, secret file, wallet directory, smoke image, Podman image,
  systemd unit, static release, scan artifact, and temporary file one manifest
  disposition.
- Encode safe removals and retention in Ansible or existing cleanup tooling.
  Keep manual-only deletion for artifacts whose ownership cannot be proved.
- Re-run the collector after cleanup and attach the redacted before/after
  comparison to the PR or operator record.

**Exit:** no unowned production artifact remains; future deployment recreates
the declared state without resurrecting removed credentials.

## Efficiency Backlog

These items are explicitly outside the cleanup PRs until their trigger fires:

- Oracle pooling: only after concurrent production p95 shows connection setup
  is material.
- Publisher blocking-worker changes: decide from Wave 7 stage timings.
- Base-image replacement: require a supported and measurably better candidate.
- CI build topology changes: require measured cost or feedback-latency evidence.
- Blanket module splits, SQL constant extraction, named-bind conversion, or
  pedantic Clippy cleanup: reject as cosmetic churn without a touched boundary.

## Required Review and Evidence

- Each cleanup PR is ready for review, passes CI, and receives an independent
  reviewer-agent pass before merge.
- Every actionable review finding is posted to the PR, fixed by a coder cycle,
  and re-reviewed. A clean review is also recorded on the PR.
- Live changes carry redacted before/after evidence and explicit rollback
  instructions. No artifact or PR comment contains secret values.
