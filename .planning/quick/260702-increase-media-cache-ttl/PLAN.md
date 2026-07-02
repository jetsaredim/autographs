---
slug: increase-media-cache-ttl
status: complete
created: 2026-07-02
---

# Increase Media Cache TTL

Update the Phase 6 static delivery cache posture so generated public media under
`/media/*` gets a longer cache lifetime suitable for CDN/browser image caching,
while keeping admin routes uncached and HTML/JSON rollback-friendly.

## Tasks

- Split `/media/*` from the existing static asset Caddy matcher.
- Set `/media/*` to `Cache-Control: public, max-age=86400`.
- Keep `/assets/*`, icons, and architecture SVG at `max-age=3600`.
- Keep public HTML/JSON/manifest at `max-age=60, must-revalidate`.
- Update docs and route contract tests.

## Verification

- `cargo test --manifest-path controller/Cargo.toml --test caddy_static_routes -- --nocapture`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook -i deploy/ansible/inventory/ci.ini deploy/ansible/playbooks/deploy.yml --syntax-check`
- `rg -n "@staticMedia|/media/\*|max-age=86400|Cache-Control" deploy/ansible/roles/autographs_deploy/files/Caddyfile controller/tests/caddy_static_routes.rs docs/deployment-runbook.md docs/static-runtime-runbook.md docs/dns-runbook.md .planning/codebase/INTEGRATIONS.md`
