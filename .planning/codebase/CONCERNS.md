# Codebase Concerns

**Analysis Date:** 2026-06-19

## Current Risks

**Planning-doc drift can mislead future agents**
- Issue: Several GSD maps still described the retired Next.js runtime and Phase
  5 as future work after the Rust/static implementation had landed.
- Impact: Future work could accidentally recreate removed app surfaces or defer
  current static/controller issues into already-complete Phase 5 scope.
- Mitigation: This reconciliation refreshes the planning maps and high-level
  state. Keep `.planning/codebase/*` updated after major implementation shifts.

**Static public artifacts are the privacy boundary**
- Issue: Public output is generated ahead of time, so any leaked private key,
  object identifier, unpublished record, Oracle detail, or image UUID can become
  durable public content.
- Impact: A publish bug could expose private media/source metadata.
- Mitigation: Keep static contract/privacy validation mandatory and generate
  public derivatives/paths only through publisher-controlled code.

**Rust controller is now the private mutation boundary**
- Issue: Admin/publish behavior is concentrated behind `/admin` and
  `/admin/api/*`.
- Impact: Auth, route exposure, persistence, and media operations now carry the
  production mutation risk formerly held by temporary operator APIs.
- Mitigation: Treat Phase 6 admin expansion as security-sensitive and keep
  retired operator APIs blocked at Caddy.

**Production security patching can affect the live VM**
- Issue: Applying `approved-production-update` runs Ansible against production
  and can update OS packages.
- Impact: Supply-chain, approval, drift-check, SSH, or `dnf` failures can
  affect live operations.
- Mitigation: Keep action SHA pins updateable/reviewed, preserve approver
  allowlist checks, refuse drifted package sets, remove stale approval labels on
  failure, and document the workflow in `docs/security-patching.md`.

**Live OCI verification depends on real secrets and tenancy state**
- Issue: Routine local tests avoid live OCI credentials.
- Impact: CI green does not automatically prove deployed Oracle/Object Storage
  behavior.
- Mitigation: Use `docs/static-runtime-runbook.md`, deployment checks, and live
  smoke tests when real credential-backed verification is needed.

## Security Considerations

- Public static output must not expose Object Storage object keys, bucket names,
  namespaces, signed URLs, Oracle internals, image UUIDs, or credentials.
- Operator/admin tokens, OCI keys, Oracle wallet material, ADB passwords, GHCR
  credentials, deploy SSH keys, and Terraform tfvars/state must stay out of git.
- Production-sensitive GitHub Actions should use least privilege and reviewed
  immutable action references where practical.
- Phase 6 and Phase 7 must add fresh security review for the new admin and AI
  surfaces they introduce.

## Fragile Areas

- Static publisher validation and release promotion.
- Derivative generation and privacy stripping.
- Rust Oracle/Object Storage production adapters.
- Caddy route boundaries between public static, admin shell, and controller API.
- Production security patching issue metadata, approval labels, cleanup, and
  drift checks.
- Planning state after out-of-band implementation progress.

## Near-Term Recommendations

1. Finish reconciling `.planning/PROJECT.md`, `.planning/ROADMAP.md`,
   `.planning/REQUIREMENTS.md`, and `.planning/STATE.md` with the implemented
   static runtime.
2. Keep README, public readiness, dependency update, security, and patching docs
   aligned with the Rust/static runtime.
3. Treat Phase 6 as admin workflow polish on the current controller foundation.
4. Re-run codebase mapping after major Phase 6/7 implementation shifts.

---

*Concerns refreshed: 2026-06-19 after Phase 5 static runtime implementation and PR 129 production security patching merge*
