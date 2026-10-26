---
phase: 08-admin-media-review-and-operational-posture
pr: 294
review_round: 2
assumptions: 08-PR-294-ASSUMPTIONS.md
plan: 08-PR-294-CONTRACT-REVISION.md
source_review: 08-07-PR-294-REVIEW.md
status: verification_passed_after_revision
---

# PR #294 contract revision review

## ISSUES FOUND

**Verdict:** The pointer-first/shared-media/tombstone design addresses the two round-2 findings in principle, but the deployment and live-proof requirements are not yet executable as written. Keep convergence at `reassessment_required` until these properties are covered and independently rechecked.

### Findings

1. **BLOCKER — The cutover must prevent the existing publisher from making candidate media public.** The plan says to deploy the Caddy deny route and publisher rule “together,” but gives no publication freeze, deployment ordering, or readiness gate. The currently deployed `prepare_shared_media` copies candidate files into public `static/media` before `promote_candidate` (`controller/src/publisher.rs:867-881, 2035-2048`). A new Caddy tombstone route alone does not hide an untombstoned candidate. Ansible copies the Caddyfile and queues service restarts (`deploy/ansible/roles/autographs_deploy/tasks/main.yml:407-414, 550-576`), so “together” is not an atomic replacement of both processes. **Required property:** No publish can execute the old candidate-to-shared path during rollout, and the revised publisher/Caddy pair must be ready before publication resumes. **Example route:** Specify an operator publish freeze and post-deploy readiness/HTTP checks, then resume publishing.

2. **WARNING — The deployed precommit proof needs a feasible pause or failure mechanism.** Wave 8 requires observing a candidate-only URL return 404 “while promotion is deliberately paused or forced to fail,” but the existing credentialed smoke has no promotion pause/fault hook (`controller/tests/live_static_publish_smoke.rs` publishes and fetches only after completion), and the implementation step adds only a unit-test callback. **Required property:** The live acceptance gate must state a repeatable, safe way to inspect the deployed Caddy origin during the precommit interval, or explicitly make the controlled local callback plus real Caddy HTTP fixture the precommit proof and reserve the deployed smoke for postcommit behavior. **Example route:** Add a test-only pause in an isolated disposable publish environment, or revise the live gate to match the proposed deterministic tests.

3. **WARNING — The takedown operator procedure needs an explicit writeability and durability preflight.** The plan gives `sudo install` into a Podman volume after `volume inspect`, but does not require checking that the Caddy container sees the marker on its read-only mount or that the chosen host mountpoint is the same volume used by the controller/Caddy quadlets (`autographs-controller.container.j2:15`, `autographs-caddy.container.j2:14`). The subsequent origin 404 check catches failure, but the procedure calls marker creation the takedown commit before proving that serving sees it. **Required property:** The operator action must verify the exact marker is present in the mounted origin namespace before treating the URL as denied, and record both origin routes' 404 responses as the effective commit evidence. **Example route:** Include a volume identity/readback check and name the successful dual-route origin probe as completion of the origin deny step.

### Coverage assessed

The revised assumptions identify candidate generation/validation, shared copy, pointer promotion, Caddy's two routes, rollback, release/shared pruning, emergency deletion, warnings, local tests, and the credentialed smoke. The plan carries the round-2 `CR-02` fix regression and `WR-02` sibling-path miss through specific tasks: active-only precommit copy, postcommit shared copy, durable exact-path deny, tombstone-aware pruning and candidate rejection, and failure tests. It preserves old/new URL retention and the resolved WebP dimension check. The failure matrix covers stalled/failed promotion, postcommit fallback, ordinary correction/unpublish/rollback, missed retained copies, and manifest-listed deletion. Public privacy and normal no-purge behavior are explicit.

The proposed Caddy matcher syntax is supported by the official [`file` matcher](https://caddyserver.com/docs/caddyfile/matchers#file), [`route` ordering](https://caddyserver.com/docs/caddyfile/directives/route), and [`try_files` rewrite](https://caddyserver.com/docs/caddyfile/directives/try_files) references. The plan correctly requires an adapted configuration and real HTTP status matrix; source-text tests alone cannot establish it. The local origin fixture, candidate callback, promotion failure, postcommit failure, rollback, tombstoned pruning, untombstoned corruption, public/private 404, and nonretrievable marker cases are specified. The credentialed smoke and Cloudflare check remain explicit live gates, not claims of completed proof.

The retention rule, emergency exception, marker backup, exact CDN purge, no marker clearing on rollback, historical URL reconciliation, and explicit operator choice before an irreversible takedown are covered. No coder work or convergence status change is authorized by this review. The parent must record revised assumptions and a plan that resolves the findings above as evidence before setting `ready_to_resume`, then use round 3 as the mandatory convergence checkpoint.

## Revision 2 — VERIFICATION PASSED

The revised assumptions and contract plan address all three findings above. This is a pre-execution plan verdict; the named checks still must pass during implementation and deployment.

| Prior finding | Revised evidence | Verdict |
| --- | --- | --- |
| Cutover exposes candidates while the old publisher runs | Contract revision §4 requires a live Caddy publish gate before replacing either process; both exact POST paths return 503, existing jobs drain, and the Ansible `autographs_deploy_publish_frozen` switch carries the gate through controller and Caddy restarts. The gate is removed only after image-digest, running-config, dual-route, publish-status, and current-release checks. | Resolved |
| Deployed precommit pause was infeasible | Contract revision §2.2 and Wave 8 define the controlled publisher callback plus a disposable *real Caddy* HTTP fixture as precommit proof, including 404 before pointer change and 200 immediately after. Deployed credentialed smoke now proves postcommit behavior; no production pause hook is required. | Resolved |
| Marker may be on an unreadable/different volume | Contract revision §3.2 inspects both container mounts against the canonical `autographs-static` volume, reads the regular marker through Caddy's read-only mount, and requires public-origin HTTPS and loopback-preview 404 before deletion or CDN purge. Host file creation alone is explicitly insufficient. | Resolved |

The freeze is operationally feasible against the current role: `autographs-controller.container.j2` gives the controller the named static volume and no host-published 8080 port; `autographs-caddy.container.j2` binds one host Caddyfile, mounts the same volume read-only, and publishes public 80/443 plus loopback 8081. `tasks/main.yml` copies the Caddyfile at lines 407–414, then flushes restart handlers at line 566. The proposed `blockinfile` insertion occurs between those steps and is required to survive the copy before either restart. The initial manual in-place host-file edit and Caddy reload gate the *currently running* old controller before Ansible starts; the plan explicitly drains active jobs and blocks other ingress. The existing `Requires=autographs-controller.service` relationship explains why the controller is kept running under a Caddy gate rather than stopped for the rollout. The named container and volume defaults match the proposed operator commands (`defaults/main.yml:10–18`).

The remaining proof is appropriately staged: local deterministic tests plus real Caddy HTTP for the precommit boundary; deployed running-image/configuration and origin probes before unfreeze; credentialed Oracle/OCI smoke and Cloudflare edge checks later. The marker step includes volume identity, readback, dual-route denial, retained-copy deletion, exact edge purge, and persistent rollback-safe denial. These checks make the revised contract reviewable and preserve the round-2 finding lineage. The parent may record this review and the revised assumptions/plan paths as convergence evidence before changing the separate review report's status; this artifact does not change that status.
