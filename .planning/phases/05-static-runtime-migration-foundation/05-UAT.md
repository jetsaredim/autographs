---
status: complete
phase: 05-static-runtime-migration-foundation
source:
  - 05-01-SUMMARY.md
  - 05-02-SUMMARY.md
  - 05-03-SUMMARY.md
  - 05-04-SUMMARY.md
  - 05-05-SUMMARY.md
  - 05-06-SUMMARY.md
  - 05-07-SUMMARY.md
started: 2026-06-20T13:22:08Z
updated: 2026-06-20T13:28:33Z
---

## Current Test

[testing complete]

## Tests

### 1. Public Static Runtime Cutover
expected: The public hostname serves the generated static collection through Caddy, and retired public API/operator routes are not reachable from the public edge.
result: pass

### 2. Live Seed-To-Static Publish
expected: A live static smoke run can create a minimal item through the private controller, upload a private original image, publish a static release, and fetch the generated item page and public derivatives through Caddy.
result: pass

### 3. Live Smoke Cleanup
expected: After the live static smoke finishes, its temporary item is unpublished from generated output, Oracle smoke rows are absent, and the uploaded private Object Storage object is gone.
result: pass

### 4. Private Controller And Admin Boundary
expected: The Rust controller remains private on the Podman network, admin mutation routes stay behind the private `/admin/api/*` boundary, and public Caddy no longer exposes the retired Next.js catalog/operator API paths.
result: pass

### 5. Secret Mount And Smoke Wallet Isolation
expected: The deployed controller uses private `:ro,Z` wallet and secret mounts, and one-shot smoke containers use a copied wallet directory with their own `:ro,Z` mount without breaking controller Oracle connectivity.
result: pass

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[none yet]
