---
status: complete
created: 2026-07-17
completed: 2026-07-17
---

# Summary

Updated public item detail signer rows so roles render inline as
`Name (role)`, moved item-specific signer context into the secondary detail
line, and reduced visible Wikipedia/IMDb profile icon badges while preserving
44px accessible hit targets. External profile links now request new tabs, and
detail fact chips list multi-signed item signers separately instead of using
the compact `A + B` summary. Removed tracked generated public item/data
fixtures, excluded generated public content/media from the Docker build
context, and kept tests focused on generated artifacts rather than checked-in
site content.

Validation:

- `cargo fmt`
- `git diff --check`
- `cargo test --test publisher`
- `cargo test --test static_contract`
