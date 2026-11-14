---
status: complete
quick_id: 261113-w9g
---

# PR #345 Rust dependency compatibility

The local fix is appropriate for the Renovate update: it adopts RSA 0.10's `PrivateKey` and `SysRng` APIs, moves SHA-1 and SHA-2 to their compatible 0.11 versions, and preserves the publisher's lowercase SHA-256 hex output. The lockfile removes superseded crypto dependency versions. No additional source changes were required.

Validation passed:

- `cargo fmt --check`
- `cargo test --locked`
- `cargo check --locked --features production-persistence`
- `cargo clippy --locked --all-targets --features production-persistence -- -D warnings`
- `cargo test --locked --features production-persistence`
- `git diff --check`

Live OCI smoke tests remain ignored by the normal test command because they require production credentials.
