# Quick Task Context: Release-Please Production Release Model

## Objective

Implement issue #192 by replacing the current tag-and-deploy-on-every-merge behavior with an accumulating release-please Release PR and an explicit release-gated production deployment model.

## Locked Decisions

- **D-01:** Ordinary PR merges update the release-please Release PR; they do not tag or deploy production.
- **D-02:** Merging the Release PR creates the semantic repository tag and GitHub Release that gates production deployment.
- **D-03:** Keep release creation and its downstream build/deploy jobs in one workflow graph, gated by release-please's `release_created` output, so the design does not depend on a second workflow event suppressed by `GITHUB_TOKEN`.
- **D-04:** Production deploys reference a semantic controller image tag (`vX.Y.Z`). The image digest is recorded and verified as integrity/audit metadata, not used as the operator-facing deploy reference.
- **D-05:** When controller sources changed across the full previous-release-to-current-release range, build and publish the current release's controller tag and deploy it.
- **D-06:** When only Terraform, Ansible, or deployment wiring changed, do not rebuild or retag the controller. Deploy the repository release while reusing the previously deployed controller tag and digest. The release manifest must make that mapping explicit.
- **D-07:** Do not apply a new repository release tag to an older controller image.
- **D-08:** Preserve `.release-status.json`'s ability to distinguish the latest repository version from the deployed controller version without restoring per-merge version/status commits.
- **D-09:** Keep a manual dispatch path for retrying or rolling back to an existing release tag.
- **D-10:** Production deployment uses the GitHub production environment and concurrency protection.
- **D-11:** Release impact must be classified from the entire previous release tag to the target release tag, not from only the most recently merged PR.
- **D-12:** Release PR validation must cover the cumulative release candidate.
- **D-13:** VM-local cleanup may delete old release-tagged images. It must protect the active controller image and the configured rollback window/previous known-good image.
- **D-14:** Remote GitHub Releases, source tags, and GHCR artifacts are not permanent. Any pruning is an explicit operator-approved decision with a dry-run inventory that identifies deployed and rollback-protected artifacts before candidates are deleted.
- **D-15:** Conventional commits are the release input. Production-impacting changes use scoped `fix`/`feat` commits; documentation-only changes may wait for a later releasable change.
- **D-16:** Prefer `always-update: true` so the Release PR stays current with all merged changes.
- **D-17:** The current issue wording that deploys should prefer digest references is superseded by the chosen contract: deploy semantic release tags and verify/record their immutable digest.

## Checker-Resolved Decisions

- **D-18:** Use a dedicated repository-scoped fine-grained PAT named `RELEASE_PLEASE_TOKEN`, limited to Contents read/write and Pull requests read/write, so release-please-created or updated Release PRs trigger the normal pull-request CI workflow.
- **D-19:** Pin `googleapis/release-please-action` v5.0.0 to commit `45996ed1f6d02564a971a2fa1b5860e934307cf7`.
- **D-20:** Any release-please action failure stops the workflow. Do not tolerate failures based on partially emitted action outputs.
- **D-21:** Before release-please runs on an ordinary push, fail closed when an unresolved draft release exists. An operator must use the manual retry path to reconcile that draft before a later Release PR or release can advance.
- **D-22:** Finalization order is deploy and health verification (or repository-only validation), then an idempotent production-status commit, then publishing the GitHub Release last. Retry must reconcile manifest, deployment, status, and publish partial states; release-manifest uploads are idempotent only for identical bytes and refuse conflicting assets.
- **D-23:** Rollback is controller-only. It uses the workflow and Terraform/Ansible mechanics from current `main`, never applies Terraform or deployment definitions checked out from the historical release, selects only the controller tag/digest from a published release manifest, and leaves the currently deployed repository release unchanged in status. Full source or infrastructure rollback requires a separate reviewed change.
- **D-24:** Add fixture-backed workflow contract tests for normal-push gating, automatic release, exact release-tag checkout, unresolved-draft blocking, retry draft lookup, published-manifest rollback, manifest conflict refusal, current-source controller-only rollback, and the immediate pre-mutation tag/digest check.

## Delivery Expectations

- Remove the custom per-merge release version/tag publishing loop and its bot-authored status churn.
- Configure release-please in manifest mode and pin third-party actions consistently with repository conventions.
- Produce a production release manifest containing repository version, source revision, impact classification, controller tag and digest, Terraform/Ansible impact, public schema version, and operator warnings/migration notes.
- Define clear behavior for release failure, retry, rollback, and local/remote retention.
- Update tests and operator documentation in the same PR.
- Preserve unrelated user-owned work, especially `.planning/quick/260831-jlc-configure-oci-managed-generation-and-adb/`.

## Validation and Review

- Run the repository's CI/workflow, Python, Terraform, Ansible, and relevant Cargo validations in proportion to changed files.
- Push the initial implementation and open a ready PR before the review/coder loop.
- Put every actionable review finding and the final clean confirmation on the GitHub PR; use inline comments where a finding maps to a changed line.
