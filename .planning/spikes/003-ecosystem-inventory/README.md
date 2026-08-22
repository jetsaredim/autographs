---
spike: 003
name: ecosystem-inventory
type: standard
validates: "Given the repository and production VM, when a bounded redacted inventory runs, then configuration drift, persistent secrets, stale runtime artifacts, and cleanup candidates are visible without collecting secret values."
verdict: VALIDATED
related: [001, 002]
tags: [inventory, configuration, secrets, podman, systemd, hygiene]
---

# Spike 003: Ecosystem Inventory

## What This Validates

Given the repository and production VM, when a bounded redacted inventory runs,
then configuration drift, persistent secrets, stale runtime artifacts, and
cleanup candidates are visible without collecting secret values.

The repository and VM collectors have both run successfully. Production
metadata remains in local redacted artifacts rather than committed evidence;
the checked-in repository inventory contains no VM data.

## Research

| Approach | Pros | Cons | Status |
|----------|------|------|--------|
| Ad hoc SSH commands | Fast for one question | Easy to miss surfaces; difficult to prove redaction or repeat | Rejected |
| Extend Ansible facts | Runs through existing automation | Couples investigation to deployment and can collect much more host data than needed | Deferred |
| Bounded standalone collector | Repeatable, testable, copyable to the VM, explicit output schema | Requires one manual copy/run/return cycle | Chosen |

The collector uses Python standard-library APIs because Python is already part
of the Ansible-managed host workflow, and the existing repository hygiene tools
use the same low-dependency pattern. It does not use `podman inspect`, dump
process environments, or emit assignment values.

## How to Run

Repository inventory and tests:

```bash
python3 .planning/spikes/003-ecosystem-inventory/test_inventory.py
python3 .planning/spikes/003-ecosystem-inventory/inventory.py repo \
  --root . \
  --output /tmp/autographs-repository-inventory.json
```

Output creation is fail-closed: the collector refuses to follow or replace an
existing path and creates new files mode `0600`. Choose a path that does not
already exist. Review a newly generated snapshot before deliberately replacing
the committed `repository-inventory.json` evidence.

Copy only the collector to the VM and run it as root so file ownership and
Podman/systemd listings are complete:

```bash
scp .planning/spikes/003-ecosystem-inventory/inventory.py \
  opc@autographs:~/autographs-ecosystem-inventory.py
ssh opc@autographs \
  'test ! -e /home/opc/autographs-vm-inventory.json && sudo python3 ~/autographs-ecosystem-inventory.py vm --output /home/opc/autographs-vm-inventory.json && sudo chown opc:opc /home/opc/autographs-vm-inventory.json'
scp opc@autographs:/home/opc/autographs-vm-inventory.json /tmp/
```

The fixed handoff path is safe because the root collector opens it exclusively
with no-follow semantics and forces mode `0600`; a pre-existing file or symlink
causes the run to fail instead of being overwritten. Remove the returned VM
artifact after review so a later inventory can create a fresh file.

Review the JSON before retaining it, then compare it with the repository:

```bash
python3 .planning/spikes/003-ecosystem-inventory/inventory.py compare \
  --repo .planning/spikes/003-ecosystem-inventory/repository-inventory.json \
  --vm /tmp/autographs-vm-inventory.json \
  --output /tmp/autographs-ecosystem-comparison.md
```

## What to Expect

- Repository variable names, classifications, source locations, and execution
  boundaries: VM runtime, VM smoke, build, deployment, CI, maintenance,
  infrastructure, local/test, or documentation-only.
- VM env-file paths, permissions, ownership, and key names without values.
- Bounded wallet, secret-directory, quadlet, static-release, and temporary-file
  metadata.
- Podman container/image/volume/network/secret names and systemd unit names.
- Findings for persistent secret-like env keys and overly broad permissions.
- Like-for-like runtime-template drift, controller variables using code
  defaults, smoke-only review, and actual pairwise env-key intersections.

## Observability

Both inventory modes emit versioned JSON with a `redaction_contract`. Command
failures record only the command name, return code, and error class. The VM
collector uses fixed listing formats rather than general inspect output.

## Investigation Trail

1. The first repository traversal used `Path.rglob`, which still descended into
   ignored dependency trees before filtering results. It was replaced with a
   pruned `os.walk`; the full repository inventory now completes in under one
   second locally.
2. Prefix-only extraction missed workflow references such as
   `PORKBUN_API_KEY`. Explicit `${{ secrets.NAME }}` and `${{ vars.NAME }}`
   extraction was added and tested.
3. The collector excludes `.planning/spikes/` so its fixture keys and generated
   evidence cannot feed back into later reports.
4. Repository comparison initially treated every variable-shaped token as a
   contract declaration. It now derives the contract only from canonical
   configuration docs, examples, deployment sources, workflows, and Terraform,
   while reporting Rust/prose/historical mentions separately.
5. The VM writer initially used ordinary `Path.write_text` at a predictable
   root-written path. It now refuses existing paths and symlinks and creates
   output mode `0600`; regression coverage proves a symlink cannot redirect the
   write.
6. Static review found that the controller quadlet loads both `app.env` and
   `controller.env`. The split is historical rather than a documented ownership
   boundary.
7. `controller.env` selects `OCI_AUTH_MODE=instance_principal`, but `app.env`
   still declares tenancy/user/fingerprint/private-key-path fields and Ansible
   still copies `OCI_PRIVATE_KEY_PEM` into the controller-mounted secrets
   directory. The private key is needed by GitHub/Terraform deployment, but the
   deployed controller has no demonstrated runtime need for it.
8. The first comparison flattened every authoritative repository key into a VM
   expectation. Schema 2 derives multi-boundary classifications from consumers
   and handoff paths, compares only runtime/smoke keys to corresponding VM env
   files, and labels controller-consumed but unset values as possible defaults
   rather than drift.
9. Root-level generated comparison output could feed its variable list back
   into a later repository scan. Known inventory artifact names are now skipped,
   and test-prefixed settings embedded in Rust source are classified local/test.

## Results

**Verdict: VALIDATED.** The redaction, boundary classification, schema-1
compatibility, private-output, and comparison contracts are validated by eleven
tests. The production collector completed without values, and its schema-1 VM
output renders through the schema-2 boundary-aware comparison.

Repository evidence:

- 409 relevant text files scanned.
- 164 distinct configuration names found.
- 81 names declared by authoritative contract sources and 83 incidental or
  historical mentions retained separately for review.
- 13 names classified as secret scalars.
- Five secret-like values are rendered into the persistent production env
  template: Oracle DB password, Oracle wallet password, admin password, admin
  password hash, and the legacy operator token.
- Runtime-template comparison found no missing or unknown production VM keys.
  `OCI_REALM_DOMAIN` is the only controller-consumed setting not materialized by
  deploy and intentionally uses the code default `oraclecloud.com`.
- Build, deployment, CI, maintenance, infrastructure, local/test, and smoke
  variables are now listed without being treated as production-runtime drift.
- Runtime OCI API-key material appears redundant with instance-principal media
  authentication and should be treated as a high-priority removal candidate.

The committed `repository-inventory.json` contains names and source locations,
not values. No production data or credentials were used.
