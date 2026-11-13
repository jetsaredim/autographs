---
status: complete
created: 2026-11-13
branch: docs/architecture-vault-cloudflare
---

# Document current architecture and serialize Terraform state use

## Goal

Show the current Cloudflare and OCI Vault interactions in the editable and published architecture diagram. Prevent PR Terraform plans from overlapping and make stateful operations wait for the OCI backend lock across PR, deploy, and Cloudflare workflows.

## Tasks

- Update the draw.io source, public SVG, and architecture page to match the current deployed design.
- Queue PR Terraform jobs across pull requests and retain every pending job.
- Set a bounded OCI state lock wait for CI plans, deployment plans and applies, Cloudflare plans and applies, and deployment taint.
- Validate diagram XML, workflow YAML and locking configuration, architecture publisher tests, and existing workflow tests.
