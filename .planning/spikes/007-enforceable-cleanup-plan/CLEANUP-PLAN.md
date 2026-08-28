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

**Implementation contract:** promote the maintained standard to `docs/`; use
Cargo/Clippy and measured LLVM coverage for Rust semantics, fixture-tested
ast-grep rules for structural source boundaries, Gitleaks for committed values,
and Rust/Ansible contract tests for Oracle and runtime secret behavior. The
Python quality checker remains historical spike evidence and is not a CI gate.

- Promote this style guide to `docs/` and extend the existing repository
  validator.
- Block new direct production env reads outside typed config, numeric SQL binds
  that do not appear exactly as `:1..:N` in occurrence order, production
  placeholder/debug macros, and persistent password/token/private-key/wallet/
  secret-key/API-key env sinks. Secret OCID references remain allowed.
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

**Status:** completed 2026-08-24 with a live production-runtime
instance-principal read of the Terraform-generated proof secret. Spike 004
remains PARTIAL until C4's kernel-persistence, encrypted-swap, wallet, secret
cutover, reboot, and rollback gates are complete.

### C4: Production Secret Cutover

**When:** after C3 and before Phase 8 closeout. **Risk:** high.

- Store the database password, wallet password, wallet PEM, and Oracle network
  files as independently rotatable OCI Secrets. Store the admin password hash
  and retained operator token if still required.
- Replace plaintext env values with secret OCIDs. Fetch scalar secrets directly
  at startup and materialize file-required wallet contents mode `0400` on
  container tmpfs.
- Before cutover, disable controller core dumps, review kernel crash-dump
  behavior, and replace persistent plaintext swap with either no swap or
  encrypted swap using a random boot-only key that is not stored durably. Prove
  those settings after reboot; tmpfs alone is not a no-disk guarantee.
- The controller core limit, systemd-coredump policy, and Kdump desired state are
  implemented as the first C4 slice. They remain PENDING live proof until an
  ordinary deploy plus an operator-approved reboot confirms the running limits,
  `kexec_crash_loaded=0`, and no running `crashkernel` argument.
- The next C4 slice owns encrypted boot-only-key swap. Its opt-in cutover must
  repeat the memory-safety preflight after stopping the Autographs services,
  immediately before `swapoff`; production conversion and reboot evidence
  follow that implementation.
- Fail the current controller closed without an automatic filesystem fallback.
  Keep the previous image digest, its exact Quadlet, and pinned legacy Vault
  versions for a bounded rollback window.
- Ship a small, separately invoked rollback materializer before cutover. Using
  the VM instance principal, it fetches the pinned legacy secret versions into
  `/run/autographs-rollback/legacy.env` (mode `0600`) and
  `/run/autographs-rollback/wallet/` (files mode `0400`) on the host's runtime
  tmpfs. It never writes beneath `/opt/autographs/env`,
  `/opt/autographs/secrets`, or `/opt/autographs/wallet`.
- Rehearse the old-image rollback exactly: stop the controller; run the helper;
  install the saved Quadlet with the previous image digest, the durable
  non-secret `controller.env`, `EnvironmentFile=/run/autographs-rollback/legacy.env`,
  and `Volume=/run/autographs-rollback/wallet:/opt/autographs/wallet:ro,Z`;
  run `systemctl daemon-reload`; start the service; then pass health, Oracle
  heartbeat, login, persistence, and full static-publish smokes.
- C4 must turn that contract into an operator-tested command sequence using
  `/usr/local/libexec/autographs-vault-rollback-materialize`, a non-secret
  manifest at `/opt/autographs/rollback/pre-vault.json`, the saved Quadlet at
  `/opt/autographs/rollback/autographs-controller.container`, and the active
  Quadlet path `/etc/containers/systemd/autographs-controller.container`:

  ```bash
  sudo systemctl stop autographs-controller.service
  sudo /usr/local/libexec/autographs-vault-rollback-materialize \
    --manifest /opt/autographs/rollback/pre-vault.json \
    --output-root /run/autographs-rollback
  sudo install -o root -g root -m 0644 \
    /opt/autographs/rollback/autographs-controller.container \
    /etc/containers/systemd/autographs-controller.container
  sudo systemctl daemon-reload
  sudo systemctl start autographs-controller.service
  ```

  The saved Quadlet pins the previous image digest and contains the two
  `EnvironmentFile`/`Volume` directives above. The helper and saved Quadlet are
  deliverables of C4, not commands that exist in the current runtime.
- Return to the Vault-capable image, remove `/run/autographs-rollback`, and
  verify service readiness. Retire the helper, saved Quadlet, and pinned secret
  versions only after the rollback rehearsal succeeds and the rollback window
  closes.
- Run health, Oracle heartbeat, persistence, login, and full static-publish
  smokes before retiring old files.

**Exit:** no password, token, private key, hash, or wallet content persists in
the controller env/secrets directories; swap/core gates and live smokes pass
after reboot; the documented old-image rollback has been executed successfully.

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
- Treat PCP and `/var/oled` as a separate C5 decision. Production has a 20 GiB
  XFS OLED logical volume with only an empty crash directory and one stale PCP
  archive; the application does not consume PCP, and the deploy role masks the
  PCP archive timers. Decide explicitly whether to restore useful monitoring or
  retire PCP. Because XFS cannot shrink, reclaiming the OLED allocation requires
  an opt-in maintenance operation with a boot-volume backup and OCI serial
  console access: stop the selected services, verify allowlisted contents,
  remove the mount/fstab and logical-volume ownership safely, extend root, and
  grow XFS. Do not combine it with encrypted-swap conversion, and do not add
  those destructive commands to the ordinary deploy role.

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
