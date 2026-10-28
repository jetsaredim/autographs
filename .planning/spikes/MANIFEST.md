# Spike Manifest

## Ideas

### ecosystem-cleanup

Audit and simplify the Autographs controller ecosystem before Phase 8 Wave 5.
Inventory repository and VM cruft, consolidate environment/configuration
contracts, move persistent plaintext secrets toward OCI Vault, assess Rust and
SQL consistency with measured efficiency evidence, and produce enforceable
standards plus a prioritized cleanup plan.

**Requirements:**

- Inventory repository, VM, Podman, systemd, deployment, OCI, and GitHub
  configuration surfaces before recommending deletion or consolidation.
- Inventory outputs may contain variable names, paths, permissions, ownership,
  and resource names, but must never collect secret values.
- Persistent production files may retain non-secret configuration; passwords,
  tokens, private keys, and equivalent secrets should move to OCI Vault where
  feasible.
- Wallet material that a driver must read as files should be treated as secret
  credential material and materialized with restrictive, preferably ephemeral,
  storage.
- Every cleanup candidate must receive one disposition: Remove, Consolidate,
  Move to Vault, Keep and document, Guard in CI, or Defer with owner and reason.
- Efficiency recommendations require evidence; stylistic preferences alone do
  not justify a broad rewrite.

### oracle-driver

Validate Oracle's new pure-Rust `oracledb` driver against the existing Oracle
Autonomous Database through a hand-carried, one-shot container. The controller
continues to use `oracle` until this spike has a recorded live verdict.

**Requirements:**

- Build the spike image locally and copy it to the OCI VM for execution, matching the established smoke-test workflow.
- Start with a read-only Oracle interaction using the existing ADB wallet and database credentials.
- Only after a successful read-only result may an explicit write gate create a temporary item and image metadata; it must verify and remove them.
- The image, logs, and committed artifacts must not contain the wallet, database password, connection descriptor, catalog content, or Object Storage credentials.
- A validated result triggers a follow-up controller migration/image update; a failed or partial result records evidence and leaves the controller unchanged.

### outer-boundary-detection

Determine whether a bounded, deterministic Rust detector can distinguish a true
low-contrast outer item boundary from a high-contrast internal card across the
two exact controls and the full 116-image held-out corpus before another
production implementation round is authorized.

**Requirements:**

- Run on the current PR branch and preserve the investigation trail on PR #276.
- Keep private originals and generated diagnostic images outside Git.
- Compare actual component contours with local-gradient/Hough boundary evidence;
  do not tune a scalar threshold to one private image.
- Require the supplied dark-card WEBP to retain accurate outer-card corners and
  `20240415_225421.jpg` to reject the recorded internal quadrilateral.
- Screen all 116 held-out originals before recommending a production algorithm.
- Do not retain production detector changes during the spikes.

### shared-image-processing

The deterministic image adjustment, detection, transformation, and derivative
pipeline was extracted into one project-local Rust crate. Production continues
to consume that crate, and the now-retired Spike 011 workbench validated it as a
real second consumer without copying behavior.

**Requirements:**

- Production and any experimental tools must call the same image-processing crate;
  copied detector or transform implementations are not acceptable.
- The deployed controller's adjustment JSON, cache-key, detector,
  transformation, derivative, route, and persistence behavior must remain
  stable.
- Keep HTTP, authentication, catalog, publishing, Oracle, OCI, and environment
  concerns outside the shared crate.
- The extraction was deployed and validated before the Spike 011 workbench
  experiment began.
- Spike 011 used private images and generated outputs outside Git and did not
  require the admin environment.
- Other crate candidates may be documented, but only capabilities with a real
  second consumer and a coherent deterministic contract should be extracted.
- The Spike 011 workbench bound to loopback, treated source images as immutable,
  and kept adjustment fixtures and generated output in an explicit directory
  outside Git.
- The Spike 011 workbench exercised the shared crate directly without requiring
  the controller, admin authentication, catalog persistence, Oracle, or OCI.

### ansible-ci-separation

Separate fast Ansible syntax/lint validation from publish integration and security
scenario validation while preserving every existing PR check and manual execution.

**Requirements:**

- Preserve all 27 syntax targets, all 17 functional invocations, and full-directory lint.
- Keep deep validation on ordinary and Release PRs and available through manual dispatch.
- Make fast, publish, and security/runtime results independently visible.
- Implement the approved split in a follow-up PR, separate from Phase 8 Wave 7.

## Spikes

| # | Idea | Name | Type | Validates | Verdict | Tags |
|---|------|------|------|-----------|---------|------|
| 001 | oracle-driver | oracledb-container-smoke | standard | Given the existing ADB wallet and a locally built image copied to the VM, when explicit read-only then gated temporary-write probes run, then `oracledb` can connect, bind, commit, query, and clean up against the live catalog without Oracle Instant Client. | VALIDATED | oracle, rust, container, smoke, adb |
| 002 | oracle-driver | oracledb-oci-persistence-smoke | standard | Given the same VM, wallet, instance principal, bucket, and 16-byte payload as the existing persistence smoke, when `oracledb` drives the database lifecycle and the existing OCI adapter handles the original, then the item and object both round-trip and are deleted with per-phase timings. | VALIDATED | oracle, rust, oci, container, smoke, adb |
| 003 | ecosystem-cleanup | ecosystem-inventory | standard | Given the repository and production VM, when a bounded redacted inventory runs, then configuration drift, persistent secrets, stale runtime artifacts, and cleanup candidates are visible without collecting secret values. | PARTIAL | inventory, configuration, secrets, podman, systemd, hygiene |
| 004 | ecosystem-cleanup | configuration-secret-boundary | comparison | Given the runtime's existing instance principal and file-based Oracle wallet requirement, when persistent env, startup materialization, and direct application retrieval are compared, then a lower-persistence secret boundary can be selected without adding OCI user credentials to the VM. | PARTIAL | configuration, env, oci-vault, secrets, instance-principal, podman |
| 005 | ecosystem-cleanup | rust-consistency | standard | Given the current Rust controller, when compiler linting and structural metrics are combined, then actionable consistency improvements can be separated from cosmetic churn and organized around existing domain boundaries. | VALIDATED | rust, clippy, architecture, sql, configuration, maintainability |
| 006 | ecosystem-cleanup | runtime-delivery-efficiency | standard | Given the current controller build, image, smoke timings, and CI topology, when costs are measured rather than inferred from code shape, then useful efficiency work can be separated from harmless duplication and local cache growth. | VALIDATED | rust, ci, container, performance, build, operations |
| 007 | ecosystem-cleanup | enforceable-cleanup-plan | standard | Given the inventory, secret-boundary, Rust consistency, and efficiency findings, when proposed standards are exercised by a fixture-tested contract checker and mapped to bounded Phase 8 work, then broad cleanup can prevent new debt without turning media waves into an ecosystem rewrite. | VALIDATED | style-guide, ci, cleanup, phase-08, rust, sql, configuration, operations |
| 008 | outer-boundary-detection | detector-mask-contour-diagnostics | standard | Given the exact positive and slab controls, when threshold masks, anchor components, contours, extrema chords, and edge evidence are rendered, then the failed coherence decision is observable without committing private media. | VALIDATED | rust, imageproc, diagnostics, contours, privacy |
| 009a | outer-boundary-detection | actual-contour-boundary | comparison | Given the diagnostic masks and actual component contours, when contour-derived quadrilateral candidates are scored, then the positive outer card is retained while the slab's internal card is rejected without per-image tuning. | INVALIDATED | rust, imageproc, contours, detector |
| 009b | outer-boundary-detection | local-gradient-rectangle | comparison | Given the same controls and corpus, when Canny/Hough rectangular-boundary evidence is scored, then it outperforms or falsifies the contour approach under the same acceptance gates. | VALIDATED | rust, imageproc, canny, hough, detector |
| 010 | shared-image-processing | shared-image-processing-crate | standard | Extracting the existing adjustment and derivative code into a project-local crate preserved the controller's behavior contracts through stable re-exports and enabled the now-retired Spike 011 workbench to validate the same crate as a real second consumer. | VALIDATED | rust, image-processing, crate, architecture, regression |
| 011 | shared-image-processing | local-image-workbench | standard | Given representative private images, can the shared production image-processing crate support exact adjustment fixtures and an honest operator fallback without weakening detector abstention or copying production behavior? The disposable harness was removed after acceptance. | VALIDATED | rust, image-processing, validation, fixtures, privacy |
| 012 | ansible-ci-separation | ansible-ci-separation | standard | Given the current CI workflow and two completed PR runs, when Ansible work is classified and timed, then a split can preserve every check while making fast and deep results separately visible. | PARTIAL | ci, ansible, github-actions, phase-08 |
