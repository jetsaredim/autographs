---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-09-27T10:34:26Z
depth: deep
files_reviewed: 3
files_reviewed_list:
  - .github/workflows/deploy.yml
  - scripts/test_release_workflow.py
  - scripts/test-fixtures/release-workflow/cases.json
findings:
  critical: 1
  warning: 1
  info: 0
  total: 2
status: issues_found
---

# Phase 08: Code Review Report

**Reviewed:** 2026-09-27T10:34:26Z
**Depth:** deep
**Files Reviewed:** 3
**Status:** issues_found

## Summary

The split checkout is mechanically compatible with `actions/checkout` and the pinned `dawidd6/action-ansible-playbook` implementation: the action changes into `with.directory` before resolving `playbook` and `requirements`, and the absolute `ANSIBLE_CONFIG` points at the same nested checkout. Controller builds, Terraform, manifest reconciliation, health checks, status mutation, publication, and rollback continue to use the selected release-tag workspace.

The production boundary is nevertheless unsafe. A retry imports the entire mutable `deploy/ansible` desired-state tree from whatever main commit was current when the workflow was dispatched, while the manifest, deployed environment metadata, status ledger, and release publication continue to claim only the old tag/source revision. Nothing constrains or records which newer runtime changes were applied. The new structural test encodes that mismatch instead of detecting it.

## Critical Issues

### CR-01: Retry can silently publish later runtime configuration as the older release

**File:** `/home/jgreenwa/dev/git/github.com/jetsaredim/autographs/.github/workflows/deploy.yml:186-194`

**Issue:** The shared invariant is that a retry may repair deployment orchestration without changing the immutable application/infrastructure/runtime payload attributed to the selected release, unless the second revision is explicitly reviewed, constrained, and recorded. The new sparse checkout violates that invariant by selecting all of `deploy/ansible` from `${{ github.sha }}` and then running its full `autographs_deploy` role at lines 612-650. That subtree is not merely an orchestration wrapper: it contains package/default definitions, kernel mutation tasks, `Caddyfile`, app environment and quadlet templates, credential handling, and service restart logic. If main receives any later change in those files before a retry dispatch, the retry applies that newer production desired state and can then pass health checks and publish the old release as though only its tagged `sourceRevision` had been deployed.

All direct sibling paths were inspected. The controller build still reads the tag-root `.github/docker-bake.hcl` and `controller/`; Terraform still runs from tag-root `infra/terraform`; the controller tag/digest and release source revision remain tag-derived; health checks validate only those old identities; status and GitHub Release publication record only the old manifest/source; rollback is isolated from the retry checkout. Consequently, none of those consumers detects or reports newer Ansible content. The current v0.2.4-to-PR delta happens to contain the intended boot-entry fix plus its test, but the permanent workflow has no guard against a later Caddy, environment, package, kernel, or service change being included in the same retry.

**Fix:** Make the recovery input an explicit, immutable, auditable patch boundary rather than the whole mutable Ansible tree. For example, pin a reviewed recovery revision, compare `RELEASE_TAG..RECOVERY_REVISION` under `deploy/ansible`, fail unless every changed runtime file is in a release-specific allowlist, and overlay only those approved files onto the tag checkout. Record the recovery revision and approved file hashes in a dedicated retry audit asset/status field before publication. At minimum, fail before Terraform/Ansible mutation if any non-allowlisted defaults, files, templates, handlers, playbooks, or tasks differ from the release tag.

## Warnings

### WR-01: The regression test proves path routing but not the claimed release-content isolation

**File:** `/home/jgreenwa/dev/git/github.com/jetsaredim/autographs/scripts/test_release_workflow.py:109-182`

**Issue:** The shared invariant is the same immutable-payload boundary from CR-01. The test verifies that the controller and Terraform steps do not mention `.retry-automation`, but it treats `sparse-checkout: deploy/ansible` as sufficient proof that no later release content is relabeled. It never enumerates the production-mutating files inside that checkout, compares their revision to the selected tag, requires an allowlist, or checks that the second revision is captured in an audit artifact. It therefore remains green if a later commit changes `roles/autographs_deploy/files/Caddyfile`, `defaults/main.yml`, any app/quadlet template, or the main deployment task list—the exact cases that violate the test name's claim. The sibling assertions for controller, Terraform, manifest/status, health, publication, and rollback were inspected; none covers Ansible desired-state provenance.

**Fix:** Replace the string-only isolation assertion with a two-revision fixture/integration test that introduces representative allowed and forbidden Ansible changes. Require an approved hotfix file to succeed, require changes to defaults/templates/files/general deployment tasks to fail before mutation, and assert that the accepted recovery SHA/file hashes are written to the retry audit record used by the publication gate.

---

_Reviewed: 2026-09-27T10:34:26Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
