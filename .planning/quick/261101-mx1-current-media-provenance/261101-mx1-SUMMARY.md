---
quick: 261101-mx1
status: complete
implementation_commit: f57b0eb
---

# Current-media provenance correction

The failed `v0.5.0` retry reached the first-cutover audit after the publish gate passed and the old controller stopped. The protected audit report listed 242 unresolved paths. Every path was named by the active `current` release and the retained release manifests. The old audit reintroduced all 242 paths as unresolved because they appeared in unproven retained manifests, despite having verified them through `current`.

The audit now removes exact paths already proven by the active release or a trusted archive from the retained unresolved set. Paths unique to unproven retained releases still block. The production retry workflow has a `v0.5.0`-only, hash-pinned helper override from current main so a retry of the existing immutable tag can use this correction without changing the tagged controller or deployment files.

## Verification

- Focused active/retained overlap, unique retained path, pruned release wait, trusted archive, and workflow order tests passed.
- Ansible preflight syntax check and lint passed.
- The previous production-shaped Caddy/Ansible fixture passed before this focused audit correction; the new overlap case is exercised by the local audit fixture.
- Live retry remains required to establish production success.
