---
phase: quick-260904-j6b-release-please-production-model
plan: 03
type: execute
wave: 3
depends_on:
  - "02"
autonomous: true
requirements:
  - ISSUE-192
files_modified:
  - .github/workflows/image-cleanup.yml
  - deploy/ansible/roles/autographs_system_cleanup/defaults/main.yml
  - deploy/ansible/roles/autographs_system_cleanup/tasks/main.yml
  - deploy/ansible/roles/autographs_system_cleanup/files/select-removable-images.py
  - scripts/cleanup-ghcr-images.py
  - scripts/test_cleanup_ghcr_images.py
  - scripts/test_select_removable_images.py
  - .github/.env.github.example
  - README.md
  - docs/release-management.md
  - docs/deployment-runbook.md
  - docs/configuration-contract.md
  - docs/dependency-updates.md
  - docs/public-readiness.md
must_haves:
  truths:
    - "VM-local cleanup can reclaim unused release-tagged images but cannot select the active or previous-known-good controller tag/digest (D-13)."
    - "Scheduled cleanup cannot delete remote GHCR versions; an operator must inspect the dry-run inventory and explicitly select remote deletion (D-14)."
    - "Documentation makes the Release PR, PAT, fail-closed draft, manifest, finalization, retry, controller-only rollback, and retention contracts executable for one operator (D-01-D-24)."
    - "Initial implementation is pushed as a ready PR and passes cumulative CI before any review agent runs; all review findings and clean confirmation remain on the PR."
  artifacts:
    - path: ".github/workflows/image-cleanup.yml"
      provides: "Scheduled local cleanup plus inventory-only remote posture and explicit manual deletion"
    - path: "scripts/cleanup-ghcr-images.py"
      provides: "Active/previous tag-and-digest-aware GHCR selection"
    - path: "scripts/test_select_removable_images.py"
      provides: "VM image selection regressions"
    - path: "docs/release-management.md"
      provides: "Complete release/retry/rollback/retention operator procedure"
  key_links:
    - from: ".github/workflows/image-cleanup.yml"
      to: ".release-status.json"
      via: "active and previous tag/digest keep reasons"
      pattern: "deployedController|previousController"
    - from: "deploy/ansible/roles/autographs_system_cleanup/tasks/main.yml"
      to: "deploy/ansible/roles/autographs_deploy/templates/app.env.j2"
      via: "active and previous image values written by deployment"
      pattern: "AUTOGRAPHS_(PREVIOUS_)?CONTROLLER_IMAGE"
---

<objective>
Align local/remote retention and operator documentation with the new release ledger and rollback model.

Purpose: Reclaim runtime disk safely, prevent unattended remote history deletion, and make every release/failure/recovery boundary understandable and repeatable.

Output: Rollback-aware cleanup selectors/workflow/tests plus a canonical release-management runbook and corrected public/configuration documentation.
</objective>

<execution_context>
@/home/jgreenwa/.codex/gsd-core/workflows/execute-plan.md
@/home/jgreenwa/.codex/gsd-core/templates/summary.md
</execution_context>

<context>
@AGENTS.md
@.planning/quick/260904-j6b-implement-issue-192-by-replacing-per-mer/CONTEXT.md
@.planning/quick/260904-j6b-implement-issue-192-by-replacing-per-mer/02-SUMMARY.md
@.release-status.json
@.github/workflows/image-cleanup.yml
@scripts/cleanup-ghcr-images.py
@scripts/test_cleanup_ghcr_images.py
@deploy/ansible/roles/autographs_system_cleanup/tasks/main.yml
@deploy/ansible/roles/autographs_system_cleanup/files/select-removable-images.py
@README.md
@docs/deployment-runbook.md
@docs/configuration-contract.md
@docs/dependency-updates.md
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Protect the explicit controller rollback window during cleanup</name>
  <files>.github/workflows/image-cleanup.yml, deploy/ansible/roles/autographs_system_cleanup/defaults/main.yml, deploy/ansible/roles/autographs_system_cleanup/tasks/main.yml, deploy/ansible/roles/autographs_system_cleanup/files/select-removable-images.py, scripts/cleanup-ghcr-images.py, scripts/test_cleanup_ghcr_images.py, scripts/test_select_removable_images.py</files>
  <behavior>
    - Local selection never emits an image ID referenced by active or previous image tag/digest and may emit older semantic-tagged, unused images outside the configured newest-image count (D-13).
    - Multi-tag Podman image records remain protected when any reference is active/previous/operator-protected; stale multi-tag records are removable with the existing force behavior.
    - GHCR selection records separate keep reasons for active and previous controller tag/digest and does not rely on mutable `latest`/`production` aliases.
    - Scheduled runs perform configured VM-local cleanup but force GHCR into inventory mode; only an explicit manual `delete` selection permits the package DELETE API (D-14).
  </behavior>
  <action>
    Add direct fixture tests for the local selector before changing it. Feed active and previous image references from both `AUTOGRAPHS_CONTROLLER_IMAGE`/digest and `AUTOGRAPHS_PREVIOUS_CONTROLLER_IMAGE`/digest lines in `app.env`; protect their image IDs regardless of age or tag multiplicity. Keep the configured newest-local count as an additional rollback window, then allow all remaining unused semantic release images to be selected so the VM can reclaim disk (D-13).

    Refactor GHCR selection into a pure tested decision function. Read the active and previous controller tags/digests from the Plan 01 status schema, preserve either mapping independently of min-age/newest-count, and print explicit keep reasons and deletion candidates. Remove automatic special treatment for mutable `latest` and `production` because Plan 02 no longer publishes them; retain `GHCR_CLEANUP_PROTECTED_TAGS` as an operator override.

    Split local and remote controls in `image-cleanup.yml`. A schedule may continue applying VM-local cleanup, but its GHCR job must always set dry-run/inventory. Manual dispatch defaults to inventory and exposes an explicit `delete` choice before remote mutation; do not infer delete from a false-like default. Log latest/deployed repository release plus active/previous controller tag/digest before selection. Pin `dawidd6/action-ansible-playbook` to `126642a1c6ce512da255ef2b41e8ee90f0077474 # v9`. Do not automate deletion of GitHub Releases or source tags; Plan 03 documentation makes those separate inventory-backed operator actions because removing either can break manifest-backed retry/rollback (D-14).
  </action>
  <verify>
    <automated>python3 -m unittest scripts/test_cleanup_ghcr_images.py scripts/test_select_removable_images.py &amp;&amp; actionlint .github/workflows/image-cleanup.yml &amp;&amp; ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/system-cleanup.yml</automated>
  </verify>
  <done>Active and previous controller bytes survive both selectors, old local release images remain reclaimable, schedules cannot delete GHCR remotely, and manual remote deletion is inventory-first.</done>
</task>

<task type="auto">
  <name>Task 2: Publish the complete release and recovery operating contract</name>
  <files>.github/.env.github.example, README.md, docs/release-management.md, docs/deployment-runbook.md, docs/configuration-contract.md, docs/dependency-updates.md, docs/public-readiness.md</files>
  <action>
    Add `docs/release-management.md` as the canonical operator procedure and link it from README/deployment docs. Describe the conventional commit contract, accumulating ready Release PR, `always-update`, cumulative CI, and exact fine-grained `RELEASE_PLEASE_TOKEN` creation/storage/rotation boundary: Autographs repository only, Contents read/write, Pull requests read/write, no OCI/package/tenancy permissions (D-01, D-12, D-15-D-20). State that action errors always stop and that any unresolved draft blocks release-please until the operator manually dispatches retry for that exact tag (D-20-D-21).

    Document automatic release state in exact order: release-please creates the semantic tag and draft Release; full-range impact selects a newly built controller tag or the existing controller mapping; workflow creates or byte-compares the manifest asset without clobbering; live tag digest is checked immediately before mutation; deployment/health or repo-only validation succeeds; the idempotent status commit lands; only then is the Release published (D-02-D-08, D-11, D-17, D-22). List every manifest field and both commit trailers. Explain retry reconciliation for absent/same/conflicting manifest, missing/already-built image, repeated deployment, already-current status, and draft/already-published final state.

    Describe rollback only as a controller rollback: choose a published manifest-bearing Release, run the current-main workflow/playbook, skip Terraform apply and historical deployment definitions, verify the recorded controller tag/digest, change the controller, and retain current deployed repository/source/infrastructure status. Explicitly say a full repository, Terraform, or Ansible rollback requires its own reviewed change and later Release rather than the rollback dispatch (D-09, D-23). Include post-operation controller health/log checks and incremental publish proof.

    Replace README's generated per-merge status block with stable links to GitHub Releases and `.release-status.json`. Correct every claim that ordinary merges tag/deploy, that digests are operator-facing deploy references, or that repo tags imply controller tags. Update configuration fields for active/previous tag/digest and separate cleanup controls. Explain local cleanup, scheduled remote inventory, explicit manual GHCR deletion, and the pre-prune GitHub Release/source-tag/GHCR inventory that names current deployed and rollback-protected artifacts. Update dependency docs for the release-please v5 SHA/PAT and the GitHub env example with `RELEASE_PLEASE_TOKEN` plus cleanup controls. Update public readiness so the initial implementation and future Release PRs are ready PRs with CI/review evidence (D-10, D-13-D-14, D-24).
  </action>
  <verify>
    <automated>python3 scripts/validate_repo_hygiene.py &amp;&amp; python3 -m unittest scripts/test_validate_repo_hygiene.py &amp;&amp; ! rg -n 'Merges to `main` run the deploy workflow|increments the repo semver in `VERSION`|prefer immutable GHCR image digests' README.md docs .github/.env.github.example</automated>
  </verify>
  <done>The repository gives one accurate release-please, failure, retry, controller-only rollback, status, and retention procedure, including the PAT's narrow permissions and the prohibition on historical infrastructure rollback.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Runtime metadata to Podman cleanup | Incorrect parsing could delete live or rollback-required local bytes. |
| Production status to GHCR cleanup | Incorrect remote keep selection could remove a manifest-backed controller. |
| Operator inventory to destructive GitHub APIs | Releases, tags, and package versions are recoverability assets until deliberately retired. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-J6B-12 | Tampering / Denial of service | local image selector | mitigate | Parse active/previous tag and digest references, protect any matching multi-tag ID, test fixtures, and keep dry-run reporting. |
| T-J6B-13 | Tampering / Denial of service | GHCR cleanup | mitigate | Pure keep-reason selection protects active/previous mappings; schedule is inventory-only; manual delete is explicit. |
| T-J6B-14 | Repudiation | remote history pruning | mitigate | Inventory logs identify protected/candidate Releases, source tags, and GHCR artifacts before separately approved deletion. |
| T-J6B-15 | Information disclosure | operator docs/logs | mitigate | Show public tags/digests/revisions only; keep PAT values, OCI credentials, and Vault data out of commands and output. |
</threat_model>

<verification>
Run all local gates before opening the PR:

1. `python3 -m unittest scripts/test_release.py scripts/test_release_workflow.py scripts/test_cleanup_ghcr_images.py scripts/test_select_removable_images.py scripts/test_validate_repo_hygiene.py scripts/test_oracle_linux_advisory_enrichment.py scripts/test_oracle_linux_oscap_results.py scripts/test_security_patching_create_issue_tasks.py`.
2. `actionlint .github/workflows/*.yml`.
3. `terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff`, then initialized `terraform validate` for runtime and tenancy roots; never apply infrastructure as local verification.
4. `ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/controller-rollback.yml deploy/ansible/playbooks/system-cleanup.yml` and `ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/`.
5. `cargo fmt --manifest-path controller/Cargo.toml --check`, `cargo test --manifest-path controller/Cargo.toml --features production-persistence`, `cargo check --manifest-path controller/Cargo.toml --features production-persistence`, and `cargo clippy --manifest-path controller/Cargo.toml --all-targets --features production-persistence -- -D warnings`.
6. `python3 scripts/validate_repo_hygiene.py` and `git diff --check`.
7. Push the complete initial implementation and open a normal ready-for-review PR that closes #192. Wait for cumulative GitHub CI, including the authenticated Terraform plan, before review agents run. Put every finding on the PR, reply directly to line threads after fixes, and post the clean confirmation; do not create review-tracking docs.
8. After the implementation PR merge, verify it only opens/updates the release-please Release PR. Configure `RELEASE_PLEASE_TOKEN` before that merge. After Release PR CI passes, merge it and validate the expected infrastructure-only bootstrap: a new repo tag/draft appears, no new controller tag is created, manifest maps the release to v0.1.3 plus resolved digest, full production deploy/health and status commit succeed, and only then the Release is published. Run an incremental publish.
</verification>

<success_criteria>
- Cleanup tests prove active/previous tag-and-digest protection locally and remotely.
- Scheduled remote deletion is impossible; manual remote deletion requires a visible inventory and explicit choice.
- Operator docs cover PAT setup, Release PR CI, fail-closed drafts, exact finalization, partial-state retry, controller-only rollback, and full-rollback exclusion.
- All local gates and ready-PR CI pass before the PR-audited review/coder cycle.
</success_criteria>

<output>
Create `.planning/quick/260904-j6b-implement-issue-192-by-replacing-per-mer/03-SUMMARY.md` when done.
</output>
