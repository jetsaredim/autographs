# Release management

Ordinary PR merges accumulate in a ready release-please Release PR. Merge that PR when you want to cut a release and deploy its production changes. GitHub Releases are the repository release ledger; GHCR stores runnable controller images.

## Initial setup

Create a fine-grained GitHub personal access token restricted to this repository, with Contents and Pull requests read/write permissions. Save it as the Actions secret `RELEASE_PLEASE_TOKEN` using `gh secret set RELEASE_PLEASE_TOKEN` and its interactive prompt. Do not paste the token into a command argument. Renew it before expiration. It needs no OCI permissions. Release-please uses this token so its PR updates trigger ordinary PR CI; the built-in `GITHUB_TOKEN` suppresses those events.

Release-please v5.0.0 is pinned to a reviewed commit. Its configuration uses a single root package, `version.txt`, `.release-please-manifest.json`, and `CHANGELOG.md`. The bootstrap follows repository v0.1.8, with controller v0.1.4 recorded as deployed. Configure the token before merging the implementation PR.

## Cut a release

1. Merge ordinary changes with conventional commit titles: `fix(deploy): ...` for a deployment repair, `feat(controller): ...` for a controller feature. Use `!` or a `BREAKING CHANGE` footer for incompatible changes. Documentation/chore-only commits can wait for the next release.
2. Review the accumulating Release PR and its cumulative CI, including Terraform planning and controller/image checks. `always-update` keeps the candidate current with main.
3. Merge the Release PR. The same workflow invocation creates its semantic tag and draft GitHub Release, then builds/selects the controller and deploys. An ordinary merge does not enter these deployment jobs.
4. Check workflow completion and the published Release's `release-manifest.json`. Deployment and health verification precede the production-status commit; publication is last. A failed operation leaves the draft for retry and blocks further release advancement.
5. Check admin health and logs, then perform an incremental publish to exercise Oracle, Object Storage, and static publication.

For optional operator instructions, include `Autographs-Release-Warning: ...` or `Autographs-Migration-Note: ...` trailers in a merged commit. These notes are public release metadata; never include credentials or private catalog identifiers.

## Controller identity

Deployment references a semantic image tag such as `ghcr.io/jetsaredim/autographs/controller:v0.2.0`. The workflow records its SHA-256 digest and verifies that the tag still resolves to it before production changes. Release tags must never be overwritten. Mutable `latest` and `production` aliases are no longer published.

Impact is computed across the full previous-release-to-target-release Git range. A controller change builds the target release's controller tag. A Terraform/Ansible-only release reuses the active controller's existing tag and digest, without creating a new controller tag. Repository-only releases do not mutate the VM.

The manifest records `schemaVersion`, `repositoryVersion`, `previousRelease`, `sourceRevision`, `impact`, controller `tag`/`digest`/`reused`, `controllerChanged`, `terraformChanged`, `ansibleOrDeployChanged`, `publicSchemaVersion`, `migrationNotes`, and `operatorWarnings`. Existing manifests are accepted only when consistent; conflicting assets are never overwritten.

`.release-status.json` distinguishes the latest repository release, deployed repository release/source, active controller tag/digest, and previous controller tag/digest. Successful operations update this status idempotently; ordinary merges generate no status commits. A controller-only rollback preserves the deployed repository/source fields.

## Retry a failed release

Use the exact tag named by the failed draft:

```bash
gh workflow run deploy.yml --ref main -f release_tag=v0.2.0 -f operation=retry
```

Retry reconciles an absent manifest, an already-built image, an already-recorded status, or a draft awaiting publication. Existing image tags must match the expected source revision and digest. Conflicting manifest bytes require investigation, not asset replacement. Do not use retry to deploy an older published release over newer production state.

VM recreation is available through the retry input `recreate_runtime_instance=true`; it remains a deliberate operation that replaces the VM. Review release migration notes and persistence requirements before selecting it.

## Roll back the controller

Choose a published Release whose manifest refers to a retained, compatible controller image:

```bash
gh workflow run deploy.yml --ref main -f release_tag=v0.2.0 -f operation=rollback
```

Rollback uses current-main automation and changes only the controller image/version/digest and previous-controller metadata. It does not apply Terraform or historical Ansible definitions. Current repository/source status stays intact. Database schema or public-contract incompatibilities may make an old controller unsuitable: read migration warnings before choosing it. Full infrastructure/source rollback requires a reviewed change and a subsequent release.

Legacy tags predating manifests are not automatically eligible for this dispatch path. Inspect admin health, credential-refresh logs, and run an incremental publish after rollback.

## Retention

VM image cleanup remains scheduled. It preserves the active controller, previous controller, container-referenced images, explicitly protected tags, and the configured newest-image window. Other unused release-tagged images may be removed. Manual local cleanup defaults to dry-run.

Remote GHCR cleanup is inventory-only on schedules. Run an inventory first:

```bash
gh workflow run image-cleanup.yml --ref main -f dry_run=true -f remote_operation=inventory
```

Review the logged deployed and previous controller mappings, keep reasons, and deletion candidates. Explicitly request remote deletion only after reviewing that inventory:

```bash
gh workflow run image-cleanup.yml --ref main -f dry_run=true -f remote_operation=delete
```

Here `dry_run` controls VM-local deletion; `remote_operation` controls GHCR independently. Untagged manifests are retained conservatively because they may be platform children of a retained image index.

GitHub Releases and source tags are not permanent retention requirements. Before manually retiring them, choose a rollback floor, list Releases and tags, and inspect each retained manifest's controller mapping. Protect the currently deployed repository release and active/previous controller artifacts, including images reused by newer repository releases. Removing a Release also removes its manifest; removing its source tag breaks source-based retry.

```bash
gh release list --limit 100
git tag --list 'v*' --sort=-version:refname
gh release view v0.2.0 --json tagName,isDraft,assets
```

Release/tag deletion is a separate manual decision after that inventory. GHCR deletion does not delete the corresponding GitHub Release or source tag, and deleting a Release does not reclaim VM disk space.
