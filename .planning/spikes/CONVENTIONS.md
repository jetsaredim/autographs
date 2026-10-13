# Spike Conventions

Patterns established for this spike session.

## Stack

Use a self-contained Rust crate and a two-stage OCI-compatible container for
database-driver spikes. Pin the explored driver in the spike's own manifest and
lockfile until a live verdict warrants a controller change.

## Structure

Build a spike image with its spike directory as the Docker build context. This
keeps `.planning` excluded from normal controller image builds without special
`.dockerignore` exceptions.

## Patterns

Live database probes must be opt-in, begin read-only, emit redacted structured
evidence, and provide a separately invocable cleanup path for any temporary
rows.

## Tools & Libraries

The `oracledb 26.0.0-beta.2` pre-release driver compiled locally and completed
both the direct live ADB read/write/cleanup smoke and the representative
ADB-plus-private-OCI-object persistence smoke without Oracle Instant Client.

## Ecosystem Cleanup

Use redacted inventories before deleting production artifacts. Durable env
files may contain non-secret coordinates and secret OCIDs, but persistent
passwords, tokens, private keys, password hashes, and wallet contents should
move behind OCI Vault and ephemeral file materialization.

Enforce semantic consistency rather than visual uniformity. Centralize runtime
configuration, reject repeated numeric Oracle binds, allow named and positional
binds under explicit occurrence-order rules, and keep one-off SQL inline unless
reuse, composition, or direct contract testing justifies a module constant.

Performance cleanup requires a workload and baseline. Keep the separate CI
semantic-check and release-image proofs; defer pooling, async rewrites, and base
image changes until measurements show a material constraint.

## Image Detector Exploration

Keep private originals, rendered masks, overlays, and per-image evaluator output
under `/tmp`; commit only reusable spike code and redacted aggregate findings.
Freeze structural constants before exact-control and corpus evaluation. Require
exact positive and negative controls before a full held-out screen, and visually
inspect every proposed confidence result rather than treating counts as quality.

For outer-boundary ambiguity, use local-gradient/Hough rectangle evidence as a
conservative veto on an existing primary candidate. Do not promote the gradient
rectangle to replacement corners without separate evidence. Border-median
component contours are diagnostic/supporting evidence only when the weak mask
has not flooded the frame.
