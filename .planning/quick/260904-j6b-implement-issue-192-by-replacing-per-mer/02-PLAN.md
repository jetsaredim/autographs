---
phase: quick-260904-j6b-release-please-production-model
plan: 02
type: execute
wave: 2
depends_on:
  - "01"
autonomous: true
requirements:
  - ISSUE-192
files_modified:
  - .github/workflows/deploy.yml
  - .github/workflows/ci.yml
  - .github/docker-bake.hcl
  - renovate.json
  - scripts/test_release_workflow.py
  - scripts/test-fixtures/release-workflow/cases.json
  - deploy/ansible/playbooks/controller-rollback.yml
  - deploy/ansible/roles/autographs_deploy/defaults/main.yml
  - deploy/ansible/roles/autographs_deploy/tasks/main.yml
  - deploy/ansible/roles/autographs_deploy/templates/app.env.j2
user_setup:
  - service: GitHub
    why: "Release-please-authored Release PR updates must emit ordinary pull_request events so cumulative CI is required before merge."
    env_vars:
      - name: RELEASE_PLEASE_TOKEN
        source: "Repository Actions secret containing a fine-grained PAT restricted to this repository with Contents read/write and Pull requests read/write."
must_haves:
  truths:
    - "Normal pushes fail preflight on unresolved drafts or run only release-please; they cannot reach build/deploy jobs unless that same action invocation emits release_created=true (D-01-D-03, D-20-D-21)."
    - "Release-please uses RELEASE_PLEASE_TOKEN and the exact v5.0.0 SHA, and any action failure stops the workflow (D-18-D-20)."
    - "Automatic release and retry use the exact target tag as deployment source; controller-only rollback uses current main automation, skips Terraform apply, and takes only the controller mapping from a published manifest (D-09, D-23)."
    - "Controller tags are never overwritten or falsely created for infrastructure-only releases, and the live tag digest is checked immediately before any Terraform/Ansible production mutation (D-04-D-07, D-17, D-24)."
    - "Finalization is deployment/health or repo-only validation, then idempotent status commit, then Release publication last; retry reconciles every partial prefix and manifest conflicts fail closed (D-22)."
  artifacts:
    - path: ".github/workflows/deploy.yml"
      provides: "Release PR, automatic release, retry, and controller-only rollback graph"
    - path: "scripts/test_release_workflow.py"
      provides: "Fixture-backed workflow state/ordering contract"
    - path: "deploy/ansible/playbooks/controller-rollback.yml"
      provides: "Current-main controller-only rollback without Terraform or full source rollback"
  key_links:
    - from: ".github/workflows/deploy.yml"
      to: "release-please-config.json"
      via: "RELEASE_PLEASE_TOKEN-authenticated manifest action"
      pattern: "RELEASE_PLEASE_TOKEN|release_created|tag_name"
    - from: ".github/workflows/deploy.yml"
      to: "scripts/release.py"
      via: "draft preflight, full-range plan, manifest comparison, digest validation, and status transition"
      pattern: "scripts/release.py"
    - from: ".github/workflows/deploy.yml"
      to: "deploy/ansible/playbooks/controller-rollback.yml"
      via: "rollback-only branch using current main and no Terraform apply"
      pattern: "controller-rollback.yml"
---

<objective>
Replace the current per-merge version/tag/deploy workflow with a release-please-gated production workflow whose automatic, retry, and controller-only rollback paths are executable contracts.

Purpose: Make the GitHub Release the production ledger and deployment gate while preserving exact source/artifact identity, cumulative PR validation, failure recovery, and current-main rollback safety.

Output: Rewritten deployment/CI workflow, semantic-only image publication, focused controller rollback playbook, runtime image metadata, and fixture-backed workflow regression tests.
</objective>

<execution_context>
@/home/jgreenwa/.codex/gsd-core/workflows/execute-plan.md
@/home/jgreenwa/.codex/gsd-core/templates/summary.md
</execution_context>

<context>
@AGENTS.md
@.planning/quick/260904-j6b-implement-issue-192-by-replacing-per-mer/CONTEXT.md
@.planning/quick/260904-j6b-implement-issue-192-by-replacing-per-mer/01-SUMMARY.md
@release-please-config.json
@.release-please-manifest.json
@scripts/release.py
@scripts/test_release.py
@.github/workflows/deploy.yml
@.github/workflows/ci.yml
@.github/docker-bake.hcl
@deploy/ansible/roles/autographs_deploy/tasks/main.yml
@deploy/ansible/roles/autographs_deploy/templates/app.env.j2
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Specify and implement the release workflow state graph</name>
  <files>.github/workflows/deploy.yml, .github/workflows/ci.yml, .github/docker-bake.hcl, renovate.json, scripts/test_release_workflow.py, scripts/test-fixtures/release-workflow/cases.json</files>
  <behavior>
    - A normal push with no draft can execute preflight and release-please but all production/build jobs remain gated when `release_created` is false; a normal push with an unresolved draft stops before release-please (D-01, D-03, D-21, D-24).
    - Automatic release uses `release_created`/`tag_name`, checks out that exact tag, and follows manifest -> digest check -> deploy/health -> status -> publish ordering (D-02-D-04, D-22, D-24).
    - Retry locates the exact selected draft, uses its tag checkout, treats identical existing manifest bytes as complete, refuses different bytes, and can repeat deployment/status before publishing (D-09, D-22, D-24).
    - Rollback requires a published manifest, checks out current main rather than the historical release, invokes only controller rollback, contains no Terraform apply path, and preserves repository deployment metadata (D-23, D-24).
    - The final tag-to-digest verification step is ordered immediately before the first production mutation for automatic, retry, and rollback cases (D-04, D-17, D-24).
  </behavior>
  <action>
    Write the fixture cases and `test_release_workflow.py` first. The test must inspect the actual workflow plus fixture-mode expected transitions, and must fail against the old workflow. Cover all nine D-24 cases by name: normal push gating, automatic release, exact automatic/retry tag checkout, unresolved draft block, retry draft lookup, published-manifest rollback, identical-versus-conflicting manifest assets, current-main/no-Terraform controller rollback, and immediate pre-mutation tag/digest verification. Validate both job conditions and step order; do not settle for substring presence alone.

    Rewrite `deploy.yml` with push-to-main and manual-dispatch entry points. Before invoking release-please on a push, list Releases and call the Plan 01 draft preflight; when a semantic draft exists, fail with the exact `workflow_dispatch` retry command/tag and do not update a later Release PR (D-21). Invoke `googleapis/release-please-action` at `45996ed1f6d02564a971a2fa1b5860e934307cf7 # v5.0.0`, using `secrets.RELEASE_PLEASE_TOKEN`, manifest config, and no `continue-on-error`; any action failure stops regardless of outputs (D-18-D-20). Gate automatic downstream jobs only on the same step's root `release_created == 'true'` and `tag_name`, not a `release` event (D-01-D-03). Keep workflow/job permissions least-privilege: the release job needs contents/pull-requests write; image publication needs packages write; production status/finalization needs contents write.

    Replace manual force-build behavior with required `release_tag` and `operation` (`retry` or `rollback`) inputs plus optional runtime recreation for retry only. Automatic/retry must validate and check out the exact semantic tag at full depth, resolve the previous reachable tag, classify that full range, and create the deterministic manifest. Retry must look up the selected draft (or recognize an already-published final state), safely reuse a controller image only if its OCI revision label matches the tag SHA, and reconcile absent/same manifest state; upload without `--clobber` and fail on conflict. Rollback must require a published Release and existing manifest, use its controller tag/digest only, and explicitly select current `main` for workflow scripts and Ansible (D-09, D-22-D-24).

    For controller impact, probe `GHCR_CONTROLLER_IMAGE_REPOSITORY:vX.Y.Z`, build it only when absent, and on retry validate an existing image's source revision before reuse. Change docker-bake to publish only the semantic release tag, removing mutable `production`/`latest` aliases. For infrastructure-only impact, select the status file's active controller tag/digest and never synthesize the new repository tag on old bytes. Resolve the sha256 registry digest after build/reuse and compare it again in the serialized production job immediately before its first mutation. Use one production job with `environment: production` and `concurrency: {group: deploy-production, cancel-in-progress: false}` to guarantee the exact order: full deploy and health, or repo-only manifest/status validation, then one idempotent status commit to current main with GITHUB_TOKEN, then publish the draft Release last. A retry repeats or skips completed stages based on verified state; an existing asset with different bytes always stops (D-04-D-10, D-17, D-22).

    Update CI to run `test_release.py` and `test_release_workflow.py` instead of the removed version tests. The dedicated PAT ensures the release-please Release PR raises the normal pull_request event; branch checks therefore validate the accumulated candidate, including the live Terraform plan. Add release-please to Renovate's explicit-review/no-automerge rule for privileged SHA-pinned actions.
  </action>
  <verify>
    <automated>python3 -m unittest scripts/test_release.py scripts/test_release_workflow.py &amp;&amp; actionlint .github/workflows/deploy.yml .github/workflows/ci.yml</automated>
  </verify>
  <done>The workflow tests prove all automatic/retry/rollback gates and ordering, release-please uses the dedicated PAT and exact v5 SHA with fail-stop behavior, and no normal push can deploy without same-run release creation.</done>
</task>

<task type="auto">
  <name>Task 2: Preserve rollback metadata and implement controller-only rollback</name>
  <files>deploy/ansible/playbooks/controller-rollback.yml, deploy/ansible/roles/autographs_deploy/defaults/main.yml, deploy/ansible/roles/autographs_deploy/tasks/main.yml, deploy/ansible/roles/autographs_deploy/templates/app.env.j2, .github/workflows/deploy.yml, scripts/test_release_workflow.py</files>
  <action>
    Extend the full deploy role's release inputs and rendered `app.env` with the verified active controller digest plus previous-known-good controller image/version/digest. Before overwriting `app.env`, parse its existing release metadata: when deploying a different controller, shift the old active controller mapping to previous; when redeploying the same controller for infrastructure-only impact or retry, keep the existing previous mapping. Keep repository version/source metadata driven by the target release for automatic/retry full deploys.

    Add a focused `controller-rollback.yml` that runs from the current-main checkout and changes controller state only. It must read the existing protected `app.env`, authenticate to GHCR, pull the manifest-recorded semantic controller tag, verify the pulled digest equals the already rechecked workflow digest, atomically shift the current active image/version/digest to previous fields, update only controller image/version/digest lines and the controller quadlet image, reload/restart the controller, verify controller/Caddy health, and log out. It must not run `terraform apply`, deploy historical Ansible code, rewrite repository version/source fields, rewrite unrelated environment/secrets, or claim a source/infrastructure rollback (D-23). Resolve the runtime host with the repository's current-main runtime-IP action/state-read path. Full source or infrastructure rollback remains a separately reviewed forward change.

    Pin the production `dawidd6/action-ansible-playbook` invocation to the already documented `126642a1c6ce512da255ef2b41e8ee90f0077474 # v9` SHA. Extend the workflow contract test to prove the rollback job uses current main, calls this focused playbook, skips Terraform apply/recreation, and leaves deployedRepositoryVersion/sourceRevision untouched (D-22-D-24).
  </action>
  <verify>
    <automated>python3 -m unittest scripts/test_release_workflow.py &amp;&amp; actionlint .github/workflows/deploy.yml &amp;&amp; ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/controller-rollback.yml</automated>
  </verify>
  <done>Full deployments preserve a previous-known-good controller mapping, and manual rollback uses only current-main controller mechanics to switch verified controller bytes while repository and infrastructure state remain unchanged.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Fine-grained PAT to GitHub | The release action can write Release PRs, tags, and Releases. |
| Workflow input/Release metadata to production | Manual tags and remote assets select production behavior. |
| Semantic tag to runtime | Registry tag mutability could substitute different controller bytes. |
| Historical release to current infrastructure | Old source must not silently roll Terraform/Ansible backward. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-J6B-06 | Elevation of privilege | RELEASE_PLEASE_TOKEN | mitigate | Repository-scoped fine-grained PAT with only Contents and Pull requests read/write; no OCI/package scope; pin the consuming action to the reviewed v5 SHA. |
| T-J6B-07 | Tampering / Repudiation | Release asset reconciliation | mitigate | Deterministic manifest, identical-byte idempotency, conflict refusal, no clobber, and publish-last ordering. |
| T-J6B-08 | Spoofing / Tampering | manual release_tag | mitigate | Require exact semantic tag/Release state/source match; retry requires draft reconciliation and rollback requires a published manifest. |
| T-J6B-09 | Tampering | GHCR tag | mitigate | Refuse overwrite, validate OCI revision on retry, and re-resolve/compare digest immediately before mutation. |
| T-J6B-10 | Elevation of privilege / Tampering | historical rollback source | mitigate | Checkout current main, skip Terraform apply, and run a focused controller-only playbook that preserves repository/source/infrastructure metadata. |
| T-J6B-11 | Denial of service | partial deployment/finalization | mitigate | Serialized production job, idempotent deploy/status steps, durable draft manifest, and publish last; manual retry reconciles any completed prefix. |
</threat_model>

<verification>
Run the two Python release suites, actionlint, Ansible syntax checks for full deploy and controller rollback, and `git diff --check`. The ready PR's normal CI is the cumulative Terraform plan/controller/image/Ansible gate.
</verification>

<success_criteria>
- Release PR updates trigger normal CI using the narrowly scoped PAT.
- Any release-please failure or prior unresolved draft blocks release advancement.
- Automatic release, retry, and controller-only rollback are fixture-tested with exact checkout, state, and ordering expectations.
- Production uses semantic controller tags and rechecks their immutable digest immediately before mutation.
- Status is committed before Release publication, and retry safely reconciles partial state without clobbering assets.
- Controller rollback uses current-main automation, performs no Terraform/source rollback, and preserves deployed repository metadata.
</success_criteria>

<output>
Create `.planning/quick/260904-j6b-implement-issue-192-by-replacing-per-mer/02-SUMMARY.md` when done.
</output>
