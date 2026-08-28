---
quick_id: 260827-vd8
status: complete
implementation_commits:
  - 3c24e0d
  - f960dba
  - 49052e0
completed: 2026-08-28
---

# Quick Task 260827-vd8 Summary: C4 Core and Kdump Persistence Gate

Implemented the first bounded C4 slice. The repository now converges controller and host core-dump policy and disables Kdump, while leaving encrypted swap, production reboot proof, wallet tmpfs, and OLED reclamation as explicit later work.

## Changes

- Added `LimitCORE=0` to the generated controller Quadlet.
- Added an Ansible-managed systemd-coredump drop-in with `Storage=none` and `ProcessSizeMax=0`.
- Added fail-closed Kdump service disablement and conditional `crashkernel` removal from installed boot entries, with no automatic reboot and no historical-dump deletion.
- Added a Rust source-contract test and CI/runtime validation entry point covering task wiring, policy values, failure handling, reboot boundaries, and absence of swap/OLED storage mutations.
- Added exact operator checks for the staged configuration and post-reboot proof.
- Recorded C3 as completed while keeping Spike 004 partial until the remaining C4 gates finish.
- Recorded PCP and the 20 GiB `/var/oled` XFS logical volume as a separate C5 decision and opt-in maintenance operation.

## Validation

- `cargo test --manifest-path controller/Cargo.toml --test runtime_kernel_persistence` — 3 passed.
- `bash scripts/validate-runtime.sh` — passed.
- Deploy playbook Ansible syntax check — passed during execution.
- `ansible-lint deploy/ansible/ --profile production` — passed during execution.
- Documentation boundary searches and negative OLED/LVM mutation search — passed.
- `git diff --check` — passed.

## Remaining Checkpoints

- Merge/deploy this slice, obtain reboot approval, then collect the documented live core/Kdump proof. The deploy itself does not reboot.
- Implement encrypted swap with a random boot-only key in the next C4 slice. Its opt-in cutover must repeat the memory preflight after stopping Autographs services, immediately before `swapoff`.
- Complete wallet tmpfs and secret cutover/rollback evidence after the kernel and swap gates are proven.
- Decide in C5 whether PCP should be restored as useful monitoring or retired before any separate `/var/oled` LVM reclamation.
