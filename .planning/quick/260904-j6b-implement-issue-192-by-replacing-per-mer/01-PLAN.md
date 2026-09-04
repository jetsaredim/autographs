---
phase: quick-260904-j6b-release-please-production-model
plan: 01
type: execute
wave: 1
depends_on: []
autonomous: true
requirements:
  - ISSUE-192
files_modified:
  - release-please-config.json
  - .release-please-manifest.json
  - version.txt
  - VERSION
  - CHANGELOG.md
  - .release-status.json
  - scripts/release.py
  - scripts/release-version.py
  - scripts/test_release.py
  - scripts/test_release_version.py
user_setup:
  - service: GitHub
    why: "Release PR changes must trigger ordinary pull-request CI rather than being suppressed as GITHUB_TOKEN-generated events."
    env_vars:
      - name: RELEASE_PLEASE_TOKEN
        source: "Create a fine-grained PAT restricted to the Autographs repository with Contents read/write and Pull requests read/write, then save it as a repository Actions secret."
must_haves:
  truths:
    - "Release-please is the only semantic version and Git-tag authority; custom merged-PR bump and tag-target logic no longer exists (D-01, D-02, D-15)."
    - "The manifest-mode root package starts from 0.1.3, creates ready accumulating Release PRs and draft v-prefixed Releases, and uses always-update (D-01, D-02, D-16)."
    - "Release state logic classifies the full previous-tag-to-target-tag range and produces an explicit repository-to-controller tag/digest manifest without retagging old images (D-04-D-07, D-11, D-17)."
    - "An unresolved draft is detectable before release-please runs, conflicting manifest bytes fail closed, and status transitions can be repeated after partial failure without corrupting production state (D-08, D-21, D-22)."
  artifacts:
    - path: "release-please-config.json"
      provides: "Manifest-mode release policy"
    - path: ".release-please-manifest.json"
      provides: "Existing 0.1.3 release bootstrap"
    - path: "scripts/release.py"
      provides: "Release range, draft, manifest, digest, and status state machine"
    - path: ".release-status.json"
      provides: "Latest/deployed repository plus active/previous controller mapping"
  key_links:
    - from: "scripts/release.py"
      to: ".release-status.json"
      via: "idempotent automatic-release and controller-rollback transitions"
      pattern: "deployedRepositoryVersion|deployedControllerDigest|previousControllerDigest"
    - from: "scripts/release.py"
      to: "controller/src/contracts.rs"
      via: "public schema version extraction for release-manifest.json"
      pattern: "PUBLIC_SCHEMA_VERSION"
---

<objective>
Define the release-please and release-state contracts before rewiring production automation.

Purpose: Give the workflow a tested, deterministic interface for release range impact, draft preflight, semantic tag/digest mapping, manifest reconciliation, and status transitions without owning version bumps or Git tags.

Output: Release-please manifest/config/version files, a replacement release helper with pure tests, and a backward-aware production status schema.
</objective>

<execution_context>
@/home/jgreenwa/.codex/gsd-core/workflows/execute-plan.md
@/home/jgreenwa/.codex/gsd-core/templates/summary.md
</execution_context>

<context>
@AGENTS.md
@.planning/quick/260904-j6b-implement-issue-192-by-replacing-per-mer/CONTEXT.md
@.release-status.json
@VERSION
@scripts/release-version.py
@scripts/test_release_version.py
@scripts/cleanup-ghcr-images.py
@controller/src/contracts.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Establish release-please as the single version authority</name>
  <files>release-please-config.json, .release-please-manifest.json, version.txt, VERSION, CHANGELOG.md</files>
  <action>
    Add a single root package to manifest-mode release-please, seeded at `0.1.3`. Use the `simple` release strategy so `version.txt` and `CHANGELOG.md` are maintained by the Release PR; delete the custom `VERSION` file instead of retaining two authorities. Configure `include-v-in-tag: true`, no component prefix, a ready rather than draft Release PR, `draft: true` for the GitHub Release, `force-tag-creation: true`, and `always-update: true` (D-01, D-02, D-16). Configure conventional-commit changelog sections and default versioning so scoped `fix`/`feat` and breaking changes drive releases while documentation/chore-only changes do not independently cut a release (D-15). Do not rewrite or recreate existing v0.0.x/v0.1.x tags.

    Keep token selection out of these config files: Plan 02 supplies the required repository-scoped `RELEASE_PLEASE_TOKEN`. Add schema URLs where supported so the JSON files remain editor- and CI-valid.
  </action>
  <verify>
    <automated>python3 -m json.tool release-please-config.json &gt;/dev/null &amp;&amp; python3 -m json.tool .release-please-manifest.json &gt;/dev/null &amp;&amp; test "$(tr -d '\n' &lt; version.txt)" = "0.1.3" &amp;&amp; test ! -e VERSION</automated>
  </verify>
  <done>The existing release line is bootstrapped without new tags, Release PRs own future versions/changelog entries, and the old standalone version authority is gone.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement the release reconciliation state machine</name>
  <files>.release-status.json, scripts/release.py, scripts/release-version.py, scripts/test_release.py, scripts/test_release_version.py</files>
  <behavior>
    - A temporary Git history where a controller change precedes a later infrastructure commit still classifies the complete previous-tag-to-target-tag range as controller-image (D-05, D-11).
    - An infrastructure-only range resolves runtime-config and reuses the active status tag/digest; repository-only resolves repo-only and requires no production mutation (D-06, D-07).
    - The target tag must match vX.Y.Z, resolve to the declared source SHA, and select itself as controller tag only when controller inputs changed (D-04, D-05, D-17).
    - Preflight returns a blocking result for any unresolved semantic draft Release and identifies its exact tag for the manual retry instruction (D-21).
    - An absent release-manifest asset may be created, byte-identical content is an idempotent no-op, and different existing bytes are a hard conflict; no clobber mode exists (D-22).
    - Automatic/retry status records latest and deployed repository versions plus active/previous controller mappings; controller-only rollback swaps controller mappings while preserving latestRepositoryVersion, deployedRepositoryVersion, latestDeployImpactVersion, and deployed source revision (D-08, D-09, D-22, D-23).
  </behavior>
  <action>
    First replace `test_release_version.py` with tests for the new behavior, using temporary Git repositories and JSON fixtures rather than mocking a single merged PR. Then delete `release-version.py` and implement `scripts/release.py`; it must not compute semantic bumps, create tags, push commits, or edit GitHub resources.

    Give the helper explicit subcommands or equivalent pure entry points for: validating a semantic target tag and exact tag-to-source SHA; resolving the nearest previous reachable semantic release; classifying every changed path in `previous..target` into controller, Terraform, Ansible/deploy, or repository-only impact; collecting `Autographs-Release-Warning` and `Autographs-Migration-Note` commit trailers; consuming GitHub Releases JSON to fail preflight on unresolved drafts; producing a deterministic `release-manifest.json`; comparing an existing asset's bytes/hash with the generated manifest and returning create/same/conflict; checking sha256 digest syntax/equality; and applying idempotent automatic/retry/controller-rollback status transitions.

    The release manifest contains schemaVersion, repositoryVersion, previousRelease, sourceRevision, impact, controller `{tag,digest,reused}`, controller/Terraform/Ansible-or-deploy change booleans, `publicSchemaVersion` parsed from `controller/src/contracts.rs`, migration notes, and operator warnings. Evolve `.release-status.json` to store latestRepositoryVersion, deployedRepositoryVersion, latestDeployImpactVersion, deployedControllerVersion/deployedControllerDigest, previousControllerVersion/previousControllerDigest, lastDeployImpact, sourceRevision, and updatedAt. Seed v0.1.3 tag fields from current production and leave only the unknown bootstrap digests empty; the first real release must resolve them from GHCR. A rollback transition changes active/previous controller fields only and never claims that the repository source or infrastructure was rolled back (D-04-D-09, D-17, D-21-D-23).
  </action>
  <verify>
    <automated>python3 -m unittest scripts/test_release.py &amp;&amp; python3 -m json.tool .release-status.json &gt;/dev/null &amp;&amp; test ! -e scripts/release-version.py &amp;&amp; test ! -e scripts/test_release_version.py</automated>
  </verify>
  <done>The tested helper exposes all decisions needed by automatic release, retry, and controller-only rollback, refuses draft/asset/digest ambiguity, and updates status idempotently without taking GitHub or Git actions itself.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Git history to release plan | Commit paths/messages select production impact and release notes. |
| GitHub Release JSON/assets to retry or rollback | Remote draft and manifest state may be missing, stale, or conflicting. |
| Semantic image tag to digest | A friendly tag must map to the expected immutable content. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-J6B-01 | Tampering | release range | mitigate | Require exact semantic tag resolution and classify the entire previous-tag-to-target-tag range. |
| T-J6B-02 | Tampering / Repudiation | release manifest | mitigate | Generate deterministic bytes, refuse conflicts instead of clobbering, and record source/tag/digest/impact. |
| T-J6B-03 | Spoofing | draft/retry target | mitigate | Block ordinary release advancement while any semantic draft remains unresolved and identify the exact retry tag. |
| T-J6B-04 | Tampering | rollback status | mitigate | Model controller-only rollback separately and preserve deployed repository/source/infrastructure fields. |
| T-J6B-05 | Information disclosure | manifest/trailers | mitigate | Permit only revisions, tags, digests, schema values, and explicit operator-authored warning/note trailers; never ingest credentials or secret identifiers. |
</threat_model>

<verification>
Run `python3 -m unittest scripts/test_release.py`, validate all JSON with `python3 -m json.tool`, and run `git diff --check` on the plan's changes.
</verification>

<success_criteria>
- Release-please owns version/changelog/tag selection from the existing 0.1.3 baseline.
- No custom per-merge bump or Git tag implementation remains.
- Full-range impact, unresolved drafts, deterministic manifest assets, digest verification, retry, and controller-only rollback status are executable Python contracts.
- `.release-status.json` can distinguish latest/deployed repository state from active/previous controller state.
</success_criteria>

<output>
Create `.planning/quick/260904-j6b-implement-issue-192-by-replacing-per-mer/01-SUMMARY.md` when done.
</output>
