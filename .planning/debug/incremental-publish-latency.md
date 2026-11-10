---
status: verifying
trigger: "Incremental publish after image adjustment took 502405 ms on 2026-11-10"
created: 2026-11-10
updated: 2026-11-10
---

# Debug Session: Incremental Publish Latency

## Symptoms

- Expected behavior: An incremental publish that generates two derivatives should finish promptly.
- Actual behavior: Release `0e53f1b4-cc56-48e6-b997-08ddd7dacc05` took 502405 ms (8m22s).
- Error messages: None in the supplied logs; publish succeeded.
- Timeline: Observed on 2026-11-10 after an image adjustment.
- Reproduction: Adjust one image and run an incremental publish on production.

## Current Focus

- hypothesis: Repeated full-byte release and shared-media verification contributes heavily to the 85-second preflight and 395-second promotion window; exact production substep durations are still unmeasured.
- test: Deploy the measured publisher and compare the next incremental publish's reconciliation, history verification, media commit, shared-media sync, and cleanup timings.
- expecting: Integrity checks still reject conflicting receipts and corrupt bytes; duplicate shared-media reads within each history verification disappear, and logs isolate remaining latency.
- next_action: Review and merge the publisher change; collect the next production publish's new timing lines before declaring the latency fixed.

## Evidence

- timestamp: 2026-11-10
  checked: supplied production publish timestamps
  found: 84.7 seconds before loadingCatalog, 17.1 seconds for generatingDerivatives with 2 generated/240 reused, and 395.3 seconds in promotingRelease
  implication: derivative generation is a small fraction of elapsed time
- timestamp: 2026-11-10
  checked: `controller/src/publisher.rs` call graph
  found: `reconcile_history` verifies every retained release and every historical shared media entry; promotion calls it before and after pointer change, then cleanup calls `load_history` twice
  implication: verification repeats full-byte reads across releases and across stages
- timestamp: 2026-11-10
  checked: publisher unit and integration tests after shared-path deduplication and timing instrumentation
  found: 7 publisher unit tests and 49 publisher integration tests passed, including conflicting-receipt and corrupt-shared-byte checks
  implication: the local integrity and publication contracts remain intact; production speedup has not yet been measured
- timestamp: 2026-11-10
  checked: requested logging follow-up for PR #336
  found: reconciliation now has its own status stage; publish requests get a request ID; publisher logs inherit a release ID span; named promotion steps report elapsed time and success/failure; media logs report files and bytes read, copied, and pruned; a drop-guarded timing summary records reached stages on success and failure
  implication: the next production publish can distinguish history verification, candidate receipt, pointer switch, shared copy, and pruning without inferring time from gaps between timestamps
- timestamp: 2026-11-10
  checked: local validation after logging follow-up
  found: 7 publisher unit tests, 49 publisher integration tests, 3 logging contract tests, Rust formatting, and production-feature strict Clippy passed
  implication: the change is ready for PR review and CI; live timing evidence is still pending

## Eliminated

## Resolution

- root_cause: "Probable repeated full-byte media verification across five history passes; the exact distribution within promotion awaits live substage timings."
- fix: "Verify each shared media path once per history pass while comparing its digest to every receipt; show reconciliation as a publish stage; correlate request and release events; log named promotion steps, media I/O totals, and full success/failure timing summaries."
- verification: "Local publisher tests pass. Live latency comparison is pending deployment and another publish."
- files_changed: ["controller/src/publisher.rs"]
