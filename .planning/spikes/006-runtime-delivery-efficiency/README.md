---
spike: 006
name: runtime-delivery-efficiency
type: standard
validates: "Given the current controller build, image, smoke timings, and CI topology, when costs are measured rather than inferred from code shape, then useful efficiency work can be separated from harmless duplication and local cache growth."
verdict: VALIDATED
related: [003, 005]
tags: [rust, ci, container, performance, build, operations]
---

# Spike 006: Runtime and Delivery Efficiency

## What This Validates

Given the current controller build, image, smoke timings, and CI topology, when
costs are measured rather than inferred from code shape, then useful efficiency
work can be separated from harmless duplication and local cache growth.

## Measurements

Measurements were taken on 2026-08-22 from the current repository and the
locally retained production-candidate image. Production timings come from the
completed `oracledb` persistence and full static-publish smokes.

| Surface | Evidence | Interpretation |
|---------|----------|----------------|
| Warm full test suite | 137 passed, 2 credential-gated tests ignored, 17.02 seconds, 204 MB maximum RSS | The ordinary controller test loop is already compact. |
| Cold production release output | 15 MB controller binary; 656 MB isolated Cargo target including dependencies and timings | The shipped artifact is small; dependency compilation is build-cache work. |
| Controller image | 50,635,613 bytes compressed content; 16 MB binary layer; 115 KB static-admin layer | Image size is not a current operational constraint. |
| Local developer target | 22 GB, entirely under `controller/target/debug` | This is disposable local cache cruft, not runtime image bloat. |
| Live Oracle/OCI persistence | 2.07 seconds total; 268 ms connect, 3 ms item, 1,408 ms upload, 4 ms image row, 57 ms verify, 180 ms cleanup | Object upload dominated database operations; connection pooling is not justified by this low-volume smoke. |
| Full static publish smoke | 44.09 seconds, including seed, upload, publish verification, draft/republish, and cleanup | This is the baseline to compare after media-adjustment publishing lands. |

## CI Build Topology

The apparent duplicate controller builds have different purposes:

- `controller-checks` compiles debug/test artifacts for formatting, tests,
  production-feature checking, and Clippy.
- `image-build` exercises the exact multi-stage release Docker build that will
  ship, including the runtime layer and production feature set.
- The pull-request image and post-merge deploy image are built from different
  commits and trust contexts. Docker BuildKit's GitHub Actions cache reuses
  layers while the immutable merge image remains independently reproducible.
- The two pull-request jobs run in parallel. Serializing the image build behind
  controller checks would reduce compute overlap but increase feedback latency.

**Decision:** keep both gates and their parallel execution. Do not replace the
release image proof with a host release build or pass an untrusted pull-request
image into deployment.

## Runtime Review

- Oracle operations are blocking and already run through `spawn_blocking`.
  Each high-level catalog operation opens a connection. For a single-admin
  controller, the live timings do not support adding a pool yet.
- Publisher filesystem and image-transformation work is synchronous inside
  async publishing flows. Existing stage timing logs provide a starting point,
  but there is no evidence of request-worker starvation. Phase 8 Wave 7 should
  record transform and filesystem stage timings before changing execution
  models.
- The Oracle Linux 10 slim runtime installs only CA certificates and carries a
  small application layer. Removing Oracle Instant Client did not create a
  measurable reason to change the supported production base in this cleanup.

## Dispositions

| Candidate | Disposition | Trigger to Revisit |
|-----------|-------------|--------------------|
| Merge CI test and image jobs | Keep and document | Revisit only if measured CI cost or latency becomes material. |
| Reuse PR image for deployment | Reject | Trust model changes and immutable merge provenance can still be proved. |
| Add Oracle connection pool | Defer | Concurrent admin demand or production p95 shows connection setup dominates. |
| Move image transforms to bounded blocking workers | Defer with Wave 7 owner | Stage timings or runtime telemetry show async worker contention. |
| Change OL10 runtime base | Defer | A supported smaller base materially improves patching, size, or compatibility. |
| Automatically clean Cargo target | Keep operator-controlled | Disk pressure recurs; document `cargo clean` or remove selected target profiles after confirming no active build. |
| Track publish performance | Guard with baseline | Compare the 44.09-second live smoke after adjustment-aware derivatives land. |

## Verdict

**VALIDATED.** The current delivery topology is deliberate and the runtime
artifact is modest. The strongest immediate efficiency finding is operational:
local build caches need observable cleanup, while runtime optimizations should
wait for Wave 7 measurements. Broad code cleanup should prioritize secret and
configuration boundaries, module ownership, and error contracts rather than
speculative pooling or async rewrites.

