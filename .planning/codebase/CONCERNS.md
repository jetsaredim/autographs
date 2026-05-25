# Codebase Concerns

**Analysis Date:** 2026-05-25

## Current Risks

**Codebase-map drift can mislead future agents**
- Issue: Some codebase intelligence docs lagged behind Phases 1-3 and described the repo as planning-only.
- Impact: Future planning/execution agents could re-scaffold existing app, infra, or CI surfaces.
- Mitigation: This reconciliation refreshes the stale maps. Keep `.planning/codebase/*` updated after substantial implementation shifts.

**Temporary operator API must not become accidental admin UX**
- Issue: `/api/operator/catalog/*` supports real mutation workflows for production data entry before Phase 4.
- Impact: If exposed through public routing or copied into UI, it could bypass the intended single-admin design.
- Mitigation: Keep routes token-guarded, blocked at the public Caddy edge, tunnel/procedure-only for production use, and covered by public-surface tests until Phase 4 replaces the bridge.

**Phase 4 owns durable admin guarantees**
- Issue: Multi-image upload/delete support exists through service/operator paths, but real admin auth, edit history, and end-to-end create/edit/publish UX remain pending.
- Impact: Marking Phase 4 requirements complete too early would hide v1 product risk.
- Mitigation: Keep DATA-03, MEDIA-04, and ADMIN-01 through ADMIN-04 pending until planned Phase 4 work lands with tests.

**Live OCI verification depends on real secrets and tenancy state**
- Issue: Routine local tests avoid live OCI credentials, while data/media smoke requires real ADB and Object Storage configuration.
- Impact: CI/local green does not automatically prove deployed data/media behavior.
- Mitigation: Use the manual Data Smoke workflow and deployment runbooks when real credential-backed verification is needed.

**Public-readiness polish is intentionally deferred**
- Issue: Root README, badges, dependency automation review, final security audit, and public showcase framing are not complete.
- Impact: The repository is operationally meaningful but not yet ready as a public showcase.
- Mitigation: Keep these items in Phase 6 and avoid spending Phase 4/5 capacity on broad showcase polish unless it unblocks delivery.

## Security Considerations

- Public image delivery must remain app-mediated and must not expose Object Storage object keys, bucket names, namespaces, signed URLs, or credentials.
- Operator token values, OCI keys, Oracle wallet material, ADB passwords, GHCR credentials, and Terraform tfvars/state must stay out of git.
- Phase 4 admin auth should provide exactly one admin path and no public account system.
- Review GitHub Actions, Terraform IAM boundaries, runtime secrets, and operator routes during Phase 6 hardening.

## Fragile Areas

- Operator API request parsing and media cleanup flows deserve focused Phase 4 tests before becoming admin UX.
- Oracle-backed repository behavior is harder to verify without live integration smoke; keep local tests plus explicit smoke workflows.
- App-mediated image delivery is a privacy-sensitive path and a potential performance hotspot.
- Planning configuration currently has Nyquist validation disabled, so validation artifacts will not be generated until that setting changes.

## Near-Term Recommendations

1. Plan Phase 4 from the existing app/service/media boundaries.
2. Decide the single-admin authentication mechanism before building admin UI.
3. Add edit-history schema and service behavior before exposing final edit/publish screens.
4. Preserve public-surface privacy regression tests as required gates for any admin or media-route changes.
5. Re-run codebase mapping after Phase 4 so the maps stay useful.

---

*Concerns refreshed: 2026-05-25 after repo-state reconciliation*
