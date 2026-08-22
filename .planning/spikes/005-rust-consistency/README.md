---
spike: 005
name: rust-consistency
type: standard
validates: "Given the current Rust controller, when compiler linting and structural metrics are combined, then actionable consistency improvements can be separated from cosmetic churn and organized around existing domain boundaries."
verdict: VALIDATED
related: [003, 004]
tags: [rust, clippy, architecture, sql, configuration, maintainability]
---

# Spike 005: Rust Consistency

## What This Validates

Given the current Rust controller, when compiler linting and structural metrics
are combined, then actionable consistency improvements can be separated from
cosmetic churn and organized around existing domain boundaries.

## Research

Rust's default Clippy groups already cover correctness, suspicious constructs,
style, complexity, and performance. The official guidance warns that the whole
pedantic group is intentionally aggressive and can produce false positives;
projects should cherry-pick useful restrictions instead of enabling every
restriction lint. [Clippy lint groups](https://doc.rust-lang.org/stable/clippy/index.html),
[Clippy configuration](https://doc.rust-lang.org/clippy/configuration.html)

| Approach | Pros | Cons | Status |
|----------|------|------|--------|
| Formatting plus default Clippy | Low noise; already enforced | Does not expose architecture or project-specific contracts | Keep |
| Enable all pedantic/restriction lints | Broad signal | Large documentation/numeric-style backlog and intentional false positives | Rejected |
| Default Clippy plus selected lints and repository audit | Keeps semantic compiler checks while enforcing local boundaries | Requires a small maintained project check | Chosen |

## How to Run

```bash
cargo clippy --manifest-path controller/Cargo.toml \
  --all-targets --features production-persistence -- -D warnings
cargo clippy --manifest-path controller/Cargo.toml \
  --all-targets --features production-persistence -- -W clippy::pedantic
python3 .planning/spikes/005-rust-consistency/test_rust_audit.py
python3 .planning/spikes/005-rust-consistency/rust_audit.py \
  --root controller \
  --output .planning/spikes/005-rust-consistency/rust-audit.json
```

## What to Expect

- The existing default Clippy gate passes with warnings denied.
- Pedantic mode reports warnings for evaluation, but is not a proposed gate.
- The structural report lists module sizes, direct env reads, error shape,
  blocking-I/O review candidates, clone counts, and SQL call style counts.

## Investigation Trail

1. The existing all-target production-persistence Clippy gate passed cleanly.
   There is no evidence of a broad correctness or ordinary Clippy-performance
   backlog.
2. Pedantic mode emitted 179 library warnings. Most are missing documentation,
   numeric casts in image geometry, `must_use`, or style suggestions. Enabling
   the group wholesale would obscure higher-value findings.
3. Four source modules contain 9,030 of 12,617 source lines: `publisher.rs`
   (2,584), `oracle_catalog.rs` (2,568), `catalog.rs` (1,977), and `routes.rs`
   (1,901). They account for roughly 72% of production source.
4. Runtime configuration reads occur outside `config.rs` in `routes.rs`,
   `oci_media.rs`, `oracle_connection.rs`, and `oracle_heartbeat.rs`. This makes
   validation and the proposed Vault boundary harder to reason about.
5. The project consistently uses `Result<T, String>` across 229 production
   signatures. That is simple, but error classification sometimes parses those
   strings, such as publisher error-kind selection. Typed boundary errors would
   make behavior and redaction less brittle.
6. Oracle blocking calls are correctly isolated with `spawn_blocking` in the
   repository and heartbeat. Publisher image work mixes async media reads with
   synchronous image transforms and filesystem work; this is an efficiency
   measurement target, not an automatic async rewrite.
7. Oracle SQL currently uses five `*_SQL` constants, three named-bind calls, and
   many positional calls. Variation is not itself a defect; the repeated-bind
   migration failure shows that bind semantics need an enforceable rule.

## Results

**Verdict: VALIDATED.** The controller is functional and passes its current
semantic lint gate. The useful cleanup is architectural and contract-oriented,
not a blanket idiom rewrite.

### High-Value Improvements

1. **Centralize runtime configuration.** Introduce typed `OracleConfig`,
   `OciConfig`, `AuthConfig`, and `PublishConfig` owned by `ControllerConfig`.
   Constructors receive values; adapters and routes do not read the process
   environment.
2. **Split by existing responsibilities.** Move memory repository code out of
   domain models; divide Oracle item/media, signer/taxonomy, history/publish, and
   row-mapping concerns; split route handlers by API resource; separate
   publisher cache/render/validation/promotion modules. Do this incrementally as
   those areas are changed, with characterization tests first.
3. **Use typed errors at durable boundaries.** Start with publisher and OCI
   adapters where safe public/log error kinds currently depend on string
   content. Do not introduce one repository-wide mega-enum.
4. **Keep SQL local and intentional.** Inline one-off statements beside the
   call. Use a module `*_SQL` constant only when the complete statement is
   reused, composed from a stable shared projection, or directly contract
   tested. Keep schema and migrations in `.sql` files; do not move ordinary
   application queries into a global query catalog.
5. **Bind by semantics.** Positional binds are acceptable when every value
   appears exactly once in left-to-right argument order. Named binds are
   required when a value repeats, placeholder order differs from argument
   order, or statement composition makes occurrence order non-obvious. Never
   rely on a repeated numeric placeholder to mean one positional argument.
6. **Cherry-pick lint enforcement.** Keep default Clippy denied. Add targeted
   bans for `todo!`, `unimplemented!`, and debug macros in production, plus a
   project-specific check for distributed config reads and unsafe Oracle bind
   patterns. Do not gate all pedantic warnings.

### Intentional Variation

- Named and positional Oracle binds may coexist under the semantic rule above.
- Sync filesystem helpers may remain in isolated publish operations until the
  efficiency spike proves request-runtime contention.
- `expect` in tests and invariant-protected lock acquisition is not a cleanup
  target merely because it exists.
- Large test fixtures can remain large when splitting would obscure one
  end-to-end scenario; source-module boundaries have higher priority.
