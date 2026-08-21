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
