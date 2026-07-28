# Phase 8: Admin Media Review and Operational Posture - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-07-28
**Phase:** 8-Admin Media Review and Operational Posture
**Areas discussed:** Uploaded image alignment, admin preview gap, phase realignment, repo posture, production security patching

---

## Image Alignment And Admin Review

The user reported that uploaded item images are slightly misaligned or askew.
The practical correction workflow requires the admin to see image previews first;
the current admin page only shows image metadata and actions.

Selected direction:

- Add authenticated admin image previews before adjustment tools.
- Store adjustment metadata instead of modifying private originals.
- Start with manual fine rotation, crop/zoom, and pan.
- Apply saved adjustments during public derivative generation.
- Keep automatic deskew and perspective correction as later tiers unless
  planning proves they are small and reliable.

## Phase Realignment

The image adjustment work is larger than a spike because it touches admin UI,
private controller routes, image metadata, Oracle persistence, derivative
publishing, cache invalidation, tests, docs, and security review.

Selected direction:

- Make Phase 8 `Admin Media Review and Operational Posture`.
- Move taxonomy media cue work to Phase 9.
- Move advisory OCR/AI-assisted ingest to Phase 10.

## Security Patching Workflow

The user clarified that the security scanner issue is not syntax. The likely
problem is that the Ansible process takes too long to gather results from the
runtime instance.

Selected direction:

- Diagnose the post-IP-resolution Ansible scan failure.
- Review what information the role gathers from the instance.
- Reduce host-side collection to the smallest approval/drift inventory needed.
- Use authoritative external Oracle Linux advisory sources for richer issue
  detail where practical instead of making the instance gather everything.

---

*Phase: 8-Admin Media Review and Operational Posture*
*Discussion captured: 2026-07-28*
