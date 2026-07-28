# Phase 10: Advisory AI-Assisted Ingest - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Original date:** 2026-07-20
**Realigned:** 2026-07-28
**Phase:** 10-Advisory AI-Assisted Ingest
**Areas discussed:** AI suggestion flow, image selection, result UI, persistence, confidence, history

---

## AI Suggestion Flow

Selected direction:

- Use image/OCR analysis in the first AI pass.
- Trigger suggestions explicitly from admin, not as an automatic background
  save or publish path.
- Analyze the primary image by default, with optional alternate image
  selection.

## Result UI

Selected direction:

- Combine a grouped review panel with inline accept/ignore controls near fields.
- Group existing matches, new candidate values, warnings/consistency issues,
  and OCR text.
- Show compact confidence and rationale where available.

## Persistence And History

Selected direction:

- Accepted suggestions stage values in the existing form until Save.
- New signer suggestions use the existing signer row flow.
- Accepted AI-assisted values receive lightweight edit-history traceability.
- Manual entry must remain fully functional when OCR/AI is unavailable,
  inaccurate, unconfigured, slow, or ignored.

## Realignment

On 2026-07-28, advisory OCR/AI ingest moved from the old combined Phase 8 into
Phase 10. Phase 8 now repairs operations and admin media review foundations;
Phase 9 owns taxonomy media cues.

---

*Phase: 10-Advisory AI-Assisted Ingest*
*Discussion captured: 2026-07-20; realigned: 2026-07-28*
