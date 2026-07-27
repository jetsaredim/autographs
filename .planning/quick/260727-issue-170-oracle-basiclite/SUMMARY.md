---
status: complete
issue: 170
completed: 2026-07-27
---

# Issue 170 Oracle Instant Client Basic Lite Evaluation

## Completed Locally

- Swapped the controller runtime image from `oracle-instantclient-basic-23.26.2.0.0-2.el10` to `oracle-instantclient-basiclite-23.26.2.0.0-2.el10`.
- Swapped both one-shot live smoke Dockerfiles from `oracle-instantclient-basic` to `oracle-instantclient-basiclite`.
- Added Dockerfile contract assertions so the runtime and smoke images stay on Basic Lite intentionally.
- Updated runtime configuration and static runtime runbook docs to name Basic Lite, record the smaller-image rationale, and call out relevant limitations for this app: narrower character-set/collation support and English client-side errors.
- Fixed a live-smoke blocker in the Oracle catalog adapter: status-only publication updates no longer rewrite tags, characters, or franchises unless those child collections are present in the update payload. The old behavior caused repeated Oracle `ORA-12860` failures while publishing the temporary live static smoke item.
- Added privacy-safe derivative failure diagnostics so live publish errors identify item/image IDs and public slugs without logging private Object Storage keys or original filenames.
- Added live persistence checksum audit and repair modes. The production-like audit found legacy missing checksums and two mismatched published images; repair updated Oracle metadata from the current private Object Storage bytes, and the follow-up audit passed with zero mismatches and zero missing published checksums.
- Hardened live static publish smoke behavior after a real full-publish timeout: publish requests now use a configurable longer timeout, destructive cleanup is deferred if the controller reports a publish still running, and the controller serializes publish requests with an async publish lock.
- Added controller startup logging for repo version, deployed controller version, controller image reference, and source revision so VM logs identify the running image/tag.
- Updated the deploy workflow release-tag step to tag the release source commit when that commit is present on `origin/main`, falling back to the release-status chore commit only when needed. This keeps GitHub release-note summaries focused on the actual change commit instead of `chore: update release status to vN.N.N`.

## Image Evidence

Issue 170 recorded the previous local controller image at about `651MB`.

Built Basic Lite images:

- `autographs-controller:issue-170-basiclite` — `401MB`
- `autographs-live-persistence-smoke:issue-170-basiclite` — `387MB`
- `autographs-live-static-publish-smoke:issue-170-basiclite` — `390MB`

The controller image is about `250MB` smaller than the issue's recorded Basic-package baseline.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test caddy_static_routes -- --nocapture` — passed, 4 tests.
- `cargo test --manifest-path controller/Cargo.toml` — passed.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test publisher -- --nocapture` — passed, 35 tests after the publish lock and derivative diagnostics changes.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_static_publish_smoke --no-run` — passed after the live smoke timeout/cleanup hardening.
- `cargo check --manifest-path controller/Cargo.toml` — passed after adding startup release metadata logging.
- `python3 -m unittest scripts/test_release_version.py` — passed, 15 tests after the release-tag workflow change.
- `docker build --file controller/Dockerfile --tag autographs-controller:issue-170-basiclite .` — passed.
- `docker build --file controller/Dockerfile.smoke --tag autographs-live-persistence-smoke:issue-170-basiclite .` — passed.
- `docker build --file controller/Dockerfile.static-smoke --build-arg AUTOGRAPHS_SMOKE_IMAGE_VERSION=issue-170-basiclite --tag autographs-live-static-publish-smoke:issue-170-basiclite .` — passed.
- `docker run --rm autographs-live-persistence-smoke:issue-170-basiclite` — passed the no-credential gated start path.
- `docker run --rm autographs-live-static-publish-smoke:issue-170-basiclite` — passed the no-credential gated start path.
- Live persistence smoke on the runtime VM — passed with Basic Lite; follow-up cleanup confirmed the temporary Oracle rows were absent and the logged Object Storage key returned `404`.
- Live static publish smoke against the previously deployed controller — initially failed twice at publication update with Oracle `ORA-12860`; this was addressed by the Oracle adapter fix.
- Live static publish smoke then exposed real media checksum drift in existing production-like data. After live checksum repair, the published-image checksum audit passed.
- The patched live static publish smoke passed on the runtime VM with Basic Lite. It created item `962bb6cd-e5e7-4054-83f1-45bd501a8aeb`, published release `585437d4-02fc-4b4f-83f3-457215d37f48`, verified generated slug `live-static-smoke-e0bb312c8f94470884e89b3c85f17b57`, then drafted and republished to remove the temporary public artifacts.

## Closeout

Issue 170 is validated: Oracle Instant Client Basic Lite works for the controller runtime, live Oracle persistence, OCI Object Storage media reads/writes, static publish generation, and non-English catalog metadata in the real controller/static path while reducing the controller image by about `250MB` versus the recorded Basic-package baseline.
