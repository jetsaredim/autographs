---
slug: increase-media-cache-ttl
status: complete
completed: 2026-07-02
---

# Increase Media Cache TTL Summary

Updated Caddy and docs so generated public media under `/media/*` now uses:

```text
Cache-Control: public, max-age=86400
```

Admin routes remain `no-store`, public HTML/JSON remain short-lived, and other
public assets remain at `max-age=3600`.

## Files Changed

- `deploy/ansible/roles/autographs_deploy/files/Caddyfile`
- `controller/tests/caddy_static_routes.rs`
- `docs/deployment-runbook.md`
- `docs/static-runtime-runbook.md`
- `docs/dns-runbook.md`
- `.planning/codebase/INTEGRATIONS.md`
- `.planning/STATE.md`

## Verification

- `cargo test --manifest-path controller/Cargo.toml --test caddy_static_routes -- --nocapture` passed.
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook -i deploy/ansible/inventory/ci.ini deploy/ansible/playbooks/deploy.yml --syntax-check` passed.
- `rg -n "@staticMedia|/media/\*|max-age=86400|Cache-Control" deploy/ansible/roles/autographs_deploy/files/Caddyfile controller/tests/caddy_static_routes.rs docs/deployment-runbook.md docs/static-runtime-runbook.md docs/dns-runbook.md .planning/codebase/INTEGRATIONS.md` passed.
