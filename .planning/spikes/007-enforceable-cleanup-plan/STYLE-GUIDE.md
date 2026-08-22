# Controller Ecosystem Style Guide

This guide records the proposed enforceable conventions from Spikes 003-006.
It is a design artifact until the cleanup plan promotes each blocking rule into
the repository validator and CI.

## Configuration Ownership

- `ControllerConfig` owns production process-environment parsing. Use typed
  sub-configs for Oracle, OCI, authentication, publishing, and runtime paths.
- Route, persistence, and media constructors receive typed configuration. They
  do not read environment variables directly.
- Keep one durable controller env file containing non-secret coordinates,
  toggles, release metadata, and OCI Vault secret OCIDs.
- Test-only and one-shot live-smoke gates stay in their test or binary owner and
  must be named and documented as non-production configuration.
- Every variable is defined once in the configuration contract with an owner,
  classification, default, consumers, and retirement condition.

## Secrets and Runtime Files

- Passwords, bearer tokens, private keys, password hashes, and wallet contents
  must not persist in controller env files.
- The production controller uses its instance principal to retrieve narrowly
  authorized OCI Vault secrets. A secret OCID is configuration, not a secret.
- File-required wallet material is materialized mode `0400` into container
  tmpfs and removed with the container. Scalar secrets remain in process memory
  and are not logged.
- Secret retrieval fails closed. Do not keep a plaintext filesystem fallback
  after the Vault cutover.
- Runtime OCI user API keys are forbidden. Deployment automation may retain its
  separate credential path until OCI workload identity replaces it.
- Inventory and cleanup tools may emit names, paths, owners, permissions,
  hashes, and resource metadata, but never secret values.

## Rust Structure

- Keep `cargo fmt`, default Clippy with warnings denied, tests, and production
  feature checking as the baseline semantic gates.
- Do not enable all pedantic or restriction lints. Promote low-noise lints one
  at a time with a documented reason.
- Production code must not contain `todo!`, `unimplemented!`, or `dbg!`.
- Split large modules along current domain responsibilities when touched. A
  line threshold is a review signal, not an automatic rewrite mandate.
- Introduce typed errors at durable boundaries where callers classify,
  redact, retry, or map failures. Do not create a single project-wide error
  enum merely to eliminate `Result<T, String>`.
- Add abstractions only when they remove demonstrated duplication, clarify an
  ownership boundary, or encode a contract that CI can verify.

## Oracle SQL

- Inline a complete one-off statement beside its query or execute call.
- Use a module-level `*_SQL` constant when the full statement is reused,
  composed from a stable shared projection, or contract tested directly.
- Keep schema definitions and migrations in `.sql` files. Do not create a
  global catalog for ordinary application queries.
- Positional binds are allowed when each value appears exactly once and bind
  arguments follow placeholder occurrence order.
- Named binds are required when a value repeats, argument order differs from
  placeholder order, or statement composition makes occurrence order unclear.
- Never assume that a repeated numeric placeholder consumes one positional
  argument. CI must reject repeated numeric bind placeholders in application
  SQL.
- Named and positional styles may coexist because the rule follows statement
  semantics, not a cosmetic repository-wide preference. Do not mix the two
  styles within one statement.

## Async and Performance

- Blocking database calls run in `spawn_blocking`; async network clients remain
  async.
- Measure synchronous filesystem or CPU image work in async paths before
  moving it. If contention is demonstrated, use bounded blocking work rather
  than unbounded task creation.
- A performance change records its baseline, workload, environment, and result.
  Style-only arguments are not evidence of improved efficiency.
- Preserve the separate CI semantic-check and release-image gates unless build
  measurements justify a topology change.

## Operations and Cruft

- Ansible owns durable production files. A file absent from the declared
  runtime contract receives an explicit Remove, Consolidate, Move to Vault,
  Keep and document, Guard in CI, or Defer disposition before deletion.
- VM cleanup starts with the redacted inventory collector. Never infer that a
  file is stale from its name alone.
- One-shot smoke env files are temporary operator artifacts. Consolidate shared
  non-secret values, document their lifetime, and remove copied credentials
  after the smoke.
- Build caches, old images, failed releases, scan artifacts, and temporary
  inventories have documented retention owners. Cleanup refuses to remove an
  active or deployed artifact.

## Enforcement Levels

| Level | Meaning | Examples |
|-------|---------|----------|
| Block | Deterministic safety or contract violation | New distributed env read, persistent secret sink, unsafe repeated numeric bind, production debug macro |
| Warn | Structural review signal | Module exceeds baseline, new stringly typed boundary error, synchronous work added to async publish path |
| Measure | Efficiency decision needs workload evidence | Connection pool, image worker changes, CI topology, base image change |
| Document | Intentional variation remains supported | Named versus positional binds, inline SQL versus justified constants |
