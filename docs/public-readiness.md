# Public Readiness Checklist

Use this checklist before making the repository public, before merging a hardening PR, and whenever a later phase adds a new public, admin, deployment, or AI surface.

## Required Gates

- Confirm the current branch is not `main` or `master` before committing changes.
- Open a normal pull request, not a draft PR.
- Let GitHub Actions validate the PR. Treat CI as the authoritative validation gate for lint, typecheck, tests, build, workflow checks, and any configured repository scans.
- Confirm the PR review feedback is written back to the PR as GitHub comments.
- Confirm repository badges in `README.md` map to real workflows or clearly documented static signals.
- Confirm `README.md` separates current implementation from Phase 5 static-runtime foundation, Phase 6 admin, and Phase 7 AI plans.
- Confirm `docs/security-review.md` records current-surface security findings as fixed, accepted, deferred, or tracked.
- Confirm `docs/dependency-updates.md` explains Renovate scope, the Dependency Dashboard issue, review expectations, and manual verification before merging dependency updates.
- Confirm `renovate.json` is present and configured for package, workflow, Docker/Corepack, Terraform, and runtime image update surfaces.
- Confirm temporary operator routes remain token-guarded, blocked at the public Caddy edge, and documented as tunnel/procedure-only until Phase 5.
- Confirm public image delivery remains app-mediated and does not expose direct Object Storage URLs, bucket names, namespaces, object keys, signed URLs, or credentials.
- Confirm data smoke status is explicit: local/CI checks can validate local-mode behavior, while live Oracle/Object Storage proof requires the manual Data Smoke workflow with real secrets.
- Confirm the image cleanup workflow and Ansible cleanup role are still covered by the documented cleanup-job behavior.
- Confirm ignored local files are not committed, especially `.env*`, Terraform tfvars/state, Oracle wallet material, OCI keys, local node/build output, and local Ansible/cache files.
- Run a credible secret scan when available, such as `gitleaks detect --redact`, and record the result in the readiness notes. If the scanner is unavailable, record that CI or a follow-up PR must provide equivalent coverage.

## Deferred Scope Rules

- Phase 5 may defer only static-runtime/private-controller foundation items: generated public artifacts, Rust controller access, minimal static admin seed/publish path, generated derivatives, Caddy cutover, and retirement of the temporary operator bridge.
- Phase 6 may defer only polished admin-workflow items: full daily-use admin UX, edit history, richer media cleanup ergonomics, and admin workflow hardening beyond the Phase 5 foundation.
- Phase 7 may defer only advisory OCR/AI ingest items: provider selection, prompts, metadata suggestions, privacy review, and failure-mode handling.
- Do not defer a current public-gallery, media-delivery, repository-secret, workflow-permission, or operator-exposure issue into Phase 5 or Phase 6. Track or fix it before public release.

## Manual Checks

- Verify the public deployment route after merge if the Deploy workflow ran.
- Run the manual Data Smoke workflow when real Oracle/Object Storage confidence is needed.
- Inspect the PR checks page rather than relying on local-only validation.
- Review the public README and architecture diagram from the perspective of a hiring manager or technical lead: the project should read as production-lean, lifecycle-aware, and honest about what is shipped versus planned.
