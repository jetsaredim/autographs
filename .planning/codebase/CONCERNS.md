# Codebase Concerns

**Analysis Date:** 2026-05-28

## Current Risks

**Codebase-map drift can mislead future agents**
- Issue: Some codebase intelligence docs lagged behind implemented app, infra, CI, and Phase 4 hardening/showcase work.
- Impact: Future planning/execution agents could re-scaffold existing surfaces or treat completed hardening as still pending.
- Mitigation: This reconciliation refreshes the stale maps. Keep `.planning/codebase/*` updated after substantial implementation shifts.

**Temporary operator API must not become accidental admin UX**
- Issue: `/api/operator/catalog/*` supports real mutation workflows for production data entry before the Phase 5 Rust private controller/static admin seed path.
- Impact: If exposed through public routing or copied into UI, it could bypass the intended single-admin design.
- Mitigation: Keep routes token-guarded, blocked at the public Caddy edge, tunnel/procedure-only for production use, and covered by public-surface tests until Phase 5 replaces or retires the bridge.

**Phase 5 owns the static runtime migration foundation**
- Issue: The current public runtime still depends on live Next.js catalog APIs and a temporary Node operator bridge.
- Impact: Jumping straight to polished admin CRUD would preserve the runtime shape Phase 5 is meant to retire.
- Mitigation: Plan and test the static public artifact contract, Rust private controller, minimal static admin seed/publish path, generated derivatives, Caddy cutover, and operator-bridge retirement before Phase 6 admin polish.

**Live OCI verification depends on real secrets and tenancy state**
- Issue: Routine local tests avoid live OCI credentials, while data/media smoke requires real ADB and Object Storage configuration.
- Impact: CI/local green does not automatically prove deployed data/media behavior.
- Mitigation: Use the manual Data Smoke workflow and deployment runbooks when real credential-backed verification is needed.

## Security Considerations

- Public image delivery must remain app-mediated and must not expose Object Storage object keys, bucket names, namespaces, signed URLs, or credentials.
- Operator token values, OCI keys, Oracle wallet material, ADB passwords, GHCR credentials, and Terraform tfvars/state must stay out of git.
- Phase 5 private admin/controller access should provide exactly one admin path and no public account system.
- GitHub Actions permissions, Terraform IAM boundaries, runtime secrets, and operator routes have current-surface Phase 4 review coverage; re-check new static/publisher/admin-controller surfaces in Phase 5, polished admin surfaces in Phase 6, and AI surfaces in Phase 7.

## Fragile Areas

- Static publisher validation, derivative generation, Caddy route cutover, and Rust access to Oracle/Object Storage are high-risk Phase 5 areas.
- Operator API request parsing and media cleanup flows deserve focused Phase 5 replacement/retirement tests before Phase 6 builds polished admin UX.
- Oracle-backed repository behavior is harder to verify without live integration smoke; keep local tests plus explicit smoke workflows.
- App-mediated image delivery is a privacy-sensitive path and a potential performance hotspot.
- Planning configuration currently has Nyquist validation disabled, so validation artifacts will not be generated until that setting changes.

## Near-Term Recommendations

1. Formally plan Phase 5 from the gathered static-runtime context before implementation starts.
2. Preserve public-surface privacy regression tests as required gates for any static-output or media-route changes.
3. Validate Rust Oracle/Object Storage, derivative generation, and Caddy static/media/admin routing early in Phase 5.
4. Keep polished admin UX, edit-history browsing, and advanced media cleanup ergonomics in Phase 6 unless Phase 5 needs a minimal foundation hook.
5. Re-run codebase mapping after major Phase 5/6/7 implementation shifts so the maps stay useful.

---

*Concerns refreshed: 2026-05-28 after Phase 5 static-runtime context gathering*
