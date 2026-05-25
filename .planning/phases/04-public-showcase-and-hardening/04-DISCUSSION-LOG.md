# Phase 4: Public Showcase and Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-25
**Phase:** 04-public-showcase-and-hardening
**Areas discussed:** Public Story, Hardening Bar, Dependency Automation, Readiness Standard, Cleanup Depth

---

## Public Story

| Option | Description | Selected |
|--------|-------------|----------|
| Showcase first | Present the repo as a polished public portfolio/showcase. | ✓ |
| Technical/operator guide first | Lead with setup, deployment, and operations. | |
| Human+AI narrative first | Lead with the GSD/human+AI collaboration story. | ✓ |

**User's choice:** Combination of showcase, project constraints, and human+AI build story including GSD.
**Notes:** The README should be credible for public review while showing how planning, constraints, phase execution, and review loops shaped the work.

---

## Hardening Bar

| Option | Description | Selected |
|--------|-------------|----------|
| Minimize Surface | Make concrete low-risk changes such as headers, reduced health detail, operator-route checks, scanning, and tracked issues. | ✓ |
| Audit Mostly | Prefer documenting risks and filing issues, with minimal runtime changes. | |
| Strict Gate | Treat security/readiness findings as blockers until fixed, even if Phase 4 expands. | ✓ |

**User's choice:** A combination of concrete surface minimization and a strict credibility gate.
**Notes:** The final refinement was to block public-readiness credibility gaps too: security leaks, stale README claims, broken badges, confusing docs, missing lifecycle notes, missing dependency automation, untriaged warnings, and similar issues should be fixed unless they clearly belong to Phase 5 or Phase 6.

---

## Dependency Automation

| Option | Description | Selected |
|--------|-------------|----------|
| Dependabot First | Native GitHub setup for npm, Actions, Docker, and Terraform with low overhead. | |
| Renovate Now | Broader/flexible coverage, better for Ansible/Caddy/custom version tracking. | ✓ |
| Policy Only | Document manual dependency review for now. | |

**User's choice:** Renovate now.
**Notes:** The user wants the lifecycle story to be stronger than the simplest native option. Plans should keep the Renovate setup conservative and explain how update PRs are reviewed.

---

## Readiness Standard

| Option | Description | Selected |
|--------|-------------|----------|
| Obvious issues resolved | Fix issues that would undermine public credibility. | ✓ |
| Security surface minimized | Reduce the current attack surface and avoid private data leakage. | ✓ |
| Proactive scanning and issue filing | Scan for risks and file/track exceptions rather than hand-waving. | ✓ |

**User's choice:** All of the above.
**Notes:** The user is reviewing this with a potential hiring manager or technical lead and wants Phase 4 to demonstrate lifecycle thinking across the project, not just code changes.

---

## Cleanup Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Narrow phase-order cleanup | Fix only stale Phase 4/5/6 references. | |
| Repo-wide doc organization | Organize docs across the repo and make structure sane for current content. | ✓ |
| Full rewrite of every doc | Rewrite everything regardless of need. | |

**User's choice:** Repo-wide doc organization.
**Notes:** The root README, docs directory, runbooks, diagrams, codebase maps, and planning notes should tell a coherent story without implying unbuilt admin or AI work exists.

---

## the agent's Discretion

- Exact README section order.
- Exact Renovate grouping/schedule.
- Exact scanner/tool choice for proactive secret/security scanning.
- Exact format for tracked exceptions and public-readiness issue list.

## Deferred Ideas

- Phase 5 remains responsible for real admin workflow, admin security review, operator bridge retirement implementation, and edit-history UX.
- Phase 6 remains responsible for AI/OCR provider, prompt, privacy, eval, and configuration/security review.
