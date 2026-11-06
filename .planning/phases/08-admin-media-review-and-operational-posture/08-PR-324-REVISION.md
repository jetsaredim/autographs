---
phase: 08-admin-media-review-and-operational-posture
pr: 324
status: proposed_for_review
depends_on:
  - 08-PR-324-CONVERGENCE.md
files_modified:
  - docs/static-runtime-runbook.md
  - docs/cdn-cache-contract.md
  - docs/deployment-runbook.md
  - docs/dns-runbook.md
  - docs/security-review.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-08-PLAN.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-SOURCE-COVERAGE.md
---

# PR #324 revised Wave 8 documentation plan

## Contract

The production CDN sequence is: (1) verify origin headers and an old/new adjusted-media URL pair; (2) choose a supported DNS topology and configure bypass/cache rules; (3) enable Cloudflare proxying; (4) collect authenticated admin bypass, media HIT, public freshness, and rollback evidence. Before step 3, only origin behavior can be observed. The static admin shell is anonymous static content with `no-store`; private API data and actions require a session.

## One coherent correction

1. Inspect every `Cloudflare`, `edge`, `before enabling`, `admin shell`, and `authenticated` claim in the listed files. Remove any statement that places edge observations before proxy activation or calls the shell itself session protected.
2. In `docs/static-runtime-runbook.md`, say the candidate/origin adjusted-publish proof happens before proxying and the edge cache observations happen only after proxying. Keep the credential-gated automated smoke, owner UAT, and current URL/hash evidence distinct.
3. In `docs/cdn-cache-contract.md`, describe `/admin/*` as an anonymously served, no-store static shell and `/admin/api/*` as session protected. Preserve bypass for both. Align the plan and security review language with that boundary; align DNS/deployment/source coverage only if their existing wording implies the wrong sequence or auth model.
4. Verify no other direct consumer contradicts the corrected contract. Run `python3 scripts/validate_repo_hygiene.py`, a focused read of the affected wording, and `git diff --check`. Commit the correction as one scoped change, push it to PR #324, post a fix-lineage comment, then request independent review. Do not run another point-fix round if a Critical or Warning remains; return to the convergence artifact for a broader revision.

## Acceptance

- No documentation asks for Cloudflare edge observations before proxy activation.
- No documentation claims the static `/admin/*` shell is itself session protected.
- Every relevant document retains admin/API bypass, authenticated 200 evidence, an old/new media URL pair, and a repeated media HIT as separate gates.
- The local checks above pass and PR #324 contains both the finding comments and fix evidence.

## Non-goals

- Do not enable Cloudflare or change DNS in this documentation correction.
- Do not modify controller, Caddy, workflow, Terraform, or static UI implementation.
- Do not mark 08-08 or Phase 8 complete before the human CDN checkpoint.
