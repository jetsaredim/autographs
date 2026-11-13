---
status: complete
created: 2026-11-13
branch: docs/architecture-vault-cloudflare
---

# Architecture and Terraform state serialization

Implementation commit `e37aeb9` updated the editable draw.io diagram, public SVG, and architecture page to show Cloudflare DNS/proxy/cache behavior, Porkbun nameserver delegation, OCI Vault secret retrieval, and the current release deployment path.

The PR Terraform job now uses a shared PR plan queue with `queue: max`, avoiding pending-run replacement. OCI backend state locking remains the cross-workflow boundary; CI, deploy, and Cloudflare stateful operations now wait up to ten minutes for a held lock. Deployment and Cloudflare workflows retain their existing `release-production` concurrency group.

Validation passed: diagram XML parsing, workflow YAML parsing, Terraform serialization assertions, `git diff --check`, `python3 -m unittest scripts/test_release_workflow.py` (22 tests), and `cargo test --manifest-path controller/Cargo.toml --test publisher` (49 tests). Production concurrency behavior still needs observation after the PR merges.
