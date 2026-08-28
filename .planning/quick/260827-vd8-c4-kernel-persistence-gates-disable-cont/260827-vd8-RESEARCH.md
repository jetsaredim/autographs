# C4 Kernel Persistence Gates - Research

**Researched:** 2026-08-28
**Domain:** Oracle Linux 10, systemd/Podman Quadlet, dm-crypt swap, Ansible
**Confidence:** HIGH for systemd/coredump behavior; MEDIUM for the live swap cutover until rehearsed on the VM

## Summary

The production VM needs swap capacity: it has 946 MiB RAM and its 2 GiB `/.swapfile` currently carries about 127 MiB, so eliminating swap is not the selected first slice. The safe end state is the existing 2 GiB backing file opened as a **plain dm-crypt mapping with a fresh `/dev/urandom` key at every boot**, and only `/dev/mapper/autographs-swap` registered as swap. `[VERIFIED: production baseline supplied 2026-08-28 UTC]` `[CITED: https://www.freedesktop.org/software/systemd/man/250/crypttab.html]`

The current plaintext-to-encrypted conversion is the risky part. Do not hide it inside an ordinary merge deploy. Land the idempotent steady-state files and a separately invoked, opt-in cutover playbook; preflight memory, stop the two application services, require `swapoff` to succeed before overwriting anything, activate encrypted swap, restart and health-check, then reboot and verify. `[VERIFIED: deploy/ansible/roles/autographs_deploy/tasks/main.yml]` `[VERIFIED: deploy/ansible/roles/security_patching/tasks/reboot_cleanup.yml]`

**Primary recommendation:** implement coredump/kdump controls in the normal role, but gate the one-time live swap conversion behind an explicit operator variable and maintenance playbook; after the reboot proof, rotate the C4 secrets so current values have never existed while plaintext swap was active.

## `/var/oled` and PCP Findings

The OCI image allocated a dedicated 20 GiB XFS logical volume at
`/var/oled`. Production inspection found only about 4 MiB of files on it:
an empty `crash/` directory and a single 3.9 MiB PCP archive from 2026-05-20.
The roughly 428 MiB reported by `df` is predominantly XFS allocation and
metadata overhead, not retained application data. The volume group has no free
extents because `ocivolume-oled` owns 20 GiB and `ocivolume-root` owns the
remaining 24.5 GiB.

Oracle uses `/var/oled/crash` as the default OCI Kdump target and PCP uses
`/var/oled/pcp` for OS/network performance archives. `[CITED:
https://docs.oracle.com/en-us/iaas/oracle-linux/oci/diagnostics-kdump.htm]`
`[CITED: https://docs.oracle.com/en/learn/ol-pcp/]` The application does not
consume PCP. The repository deliberately masks all `pmlogger` and `pmie`
maintenance timers, so production has `pmcd` active but `pmlogger`/`pmie`
inactive and cannot provide a maintained historical archive. The installed
Oracle PCP configuration otherwise specifies a seven-day retention window and
100 MiB archive rotation, but those settings are inert while the timers and
logger are stopped. `[VERIFIED:
deploy/ansible/roles/autographs_deploy/defaults/main.yml]` `[VERIFIED:
production inventory 2026-08-28 UTC]`

Once C4 disables Kdump, this project has no declared consumer for the OLED
volume. XFS cannot shrink, so reclaiming the allocation requires a separate,
explicit C5 maintenance operation rather than an ordinary deploy: verify only
allowlisted PCP/empty-crash paths exist, stop and retire PCP, back up any wanted
archive, unmount `/var/oled`, remove its fstab entry and logical volume, extend
`ocivolume-root`, and run `xfs_growfs /`. This can return nearly 20 GiB to the
root filesystem, but it needs an OCI boot-volume backup and serial-console
recovery access because a stale fstab entry or interrupted LVM operation can
affect boot. Do not combine this storage migration with the encrypted-swap
conversion or wallet cutover.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| Encrypted swap lifecycle | Host OS/systemd | Ansible | `/etc/crypttab` and `/etc/fstab` are translated into boot units; Ansible owns desired state. `[CITED: https://www.freedesktop.org/software/systemd/man/250/crypttab.html]` |
| Controller core limit | Podman Quadlet-generated systemd service | Host coredump policy | `LimitCORE=0` belongs on `autographs-controller.service`; host policy prevents persistence if another process dumps. `[CITED: https://www.freedesktop.org/software/systemd/man/250/systemd-coredump.socket.html]` |
| Kernel crash dumps | Oracle Linux boot/service configuration | Ansible | Kdump captures stopped-kernel memory and can write `vmcore`; masking plus post-reboot `kexec_crash_loaded=0` closes that path. `[CITED: https://docs.oracle.com/en/operating-systems/oracle-linux/8/boot/monitoring-WorkingWithKernelDumps.html]` |
| Reboot and health proof | Operator-run Ansible playbook | Existing health checks | The repository already waits for reboot, both Quadlets, static manifest, and admin health. `[VERIFIED: deploy/ansible/roles/security_patching/tasks/reboot_cleanup.yml]` |

## Project Constraints (from AGENTS.md)

- Keep the one-Rust-controller/static-public architecture and OCI Always Free operational simplicity; do not introduce another steady-state secret/swap daemon. `[VERIFIED: AGENTS.md]`
- Put reusable behavior in Ansible roles, keep playbooks thin, and run Ansible syntax/lint plus proportionate live smoke checks. `[VERIFIED: AGENTS.md]`
- Preserve least privilege, explicit secret handling, fail-closed runtime behavior, and redacted operational evidence. `[VERIFIED: AGENTS.md]`
- Work through the active GSD quick workflow, never commit to `main`, and put substantive review findings/clean confirmation on the PR. `[VERIFIED: AGENTS.md]`

## Prescriptive Design

### 1. Encrypted boot-only-key swap

Use this exact logical contract in `/etc/crypttab`:

```text
autographs-swap /.swapfile /dev/urandom swap,cipher=aes-xts-plain64,size=512,nofail
```

and replace the raw fstab entry with:

```text
/dev/mapper/autographs-swap none swap defaults,nofail 0 0
```

`crypttab` permits a file as the backing source and `/dev/urandom` as a randomized swap key; `swap` implies plain dm-crypt and runs `mkswap` after every activation. It is destructive on every boot, which is correct only because `/.swapfile` is dedicated to swap. `[CITED: https://www.freedesktop.org/software/systemd/man/250/crypttab.html]` XTS uses two keys; a 512-bit XTS key gives two 256-bit halves. `[CITED: https://man7.org/linux/man-pages/man8/cryptsetup-luksopen.8.html]` `nofail` makes a cryptsetup fault degrade to **no swap**, not plaintext swap or an unbootable VM; the live validation must still fail until the encrypted mapper is active. `[CITED: https://www.freedesktop.org/software/systemd/man/250/crypttab.html]`

Let `systemd-cryptsetup-generator` create the mapping unit and fstab create the swap unit; do not hand-roll boot units. `[CITED: https://www.freedesktop.org/software/systemd/man/250/crypttab.html]` Install/verify the Oracle Linux `cryptsetup` package before touching swap. `[ASSUMED: target package name; verify with dnf on the VM before implementation]`

The one-time cutover must use an Ansible `block`/`rescue` sequence:

1. Assert the source is exactly `/.swapfile`, the target name is exactly `autographs-swap`, and no second raw swap exists.
2. Assert `MemAvailable >= current SwapUsed + 256 MiB`; then stop controller and Caddy and re-check. The 256 MiB margin is a project safety choice, not an OS guarantee. `[ASSUMED: selected operational margin]`
3. Back up `/etc/fstab`, `/etc/crypttab`, and `grubby --info=ALL` as non-secret rollback evidence.
4. Run `swapoff /.swapfile`; on any failure, restart services and abort without modifying the file.
5. Overwrite the full existing allocation (`dd ... conv=fsync`) before reuse. This is best-effort local media sanitization, not a claim about OCI snapshots or provider-side recovery.
6. Write the managed crypttab/fstab entries, `systemctl daemon-reload`, start `systemd-cryptsetup@autographs\x2dswap.service`, then `swapon /dev/mapper/autographs-swap`.
7. On activation failure, close the mapper, recreate raw swap, restore the two config files, enable raw swap, restart services, and abort. This rollback is allowed only **before** new rotated C4 values are loaded.
8. Restart controller/Caddy and pass current health checks; schedule an explicit reboot rather than rebooting every ordinary deploy.

An overwrite cannot prove that old plaintext pages are irrecoverable from cloud storage history. Therefore rotate the database password, wallet password, and admin hash after encrypted-swap reboot validation and before wallet-tmpfs cutover. This ensures the new current versions were never exposed to plaintext swap. `[VERIFIED: .planning/spikes/004-configuration-secret-boundary/CONFIGURATION-BOUNDARY.md]`

### 2. Userspace and kernel dump controls

Add `LimitCORE=0` to `[Service]` in `autographs-controller.container.j2`. The observed generated service currently has unlimited core size; systemd documents `LimitCORE=` for service processes. `[VERIFIED: production baseline supplied 2026-08-28 UTC]` `[CITED: https://www.freedesktop.org/software/systemd/man/250/systemd-coredump.socket.html]`

Install `/etc/systemd/coredump.conf.d/99-autographs.conf`:

```ini
[Coredump]
Storage=none
ProcessSizeMax=0
```

Systemd explicitly specifies both settings to disable coredump processing; drop-ins are read for each new dump, so masking the currently active/static socket is unnecessary. `[CITED: https://www.freedesktop.org/software/systemd/man/250/systemd-coredump.socket.html]` Existing files under `/var/lib/systemd/coredump`, `/var/crash`, or OCI's `/var/oled/crash` require a metadata-only inventory and explicit operator approval before deletion because they are historical diagnostic data. `[CITED: https://docs.oracle.com/en/operating-systems/oracle-linux/10/boot/monitoring-ConfiguringKdump.html]`

Mask and stop `kdump.service` with a non-ignored Ansible task, remove `crashkernel` from all boot entries using `grubby --update-kernel=ALL --remove-args="crashkernel"`, and reboot. Oracle Linux documents `grubby` as the persistent boot-argument interface and `/proc/cmdline` as the running-kernel proof. `[CITED: https://docs.oracle.com/en/operating-systems/oracle-linux/10/boot/boot-UsinggrubbyToManageKernels_change_kernel_command_line_boot_parameters.html]` Oracle also says kdump captures stopped-kernel memory and may store it under `/var/oled/crash` on OCI, so the currently enabled-but-failed service is not an acceptable end state. `[CITED: https://docs.oracle.com/en/operating-systems/oracle-linux/8/boot/monitoring-WorkingWithKernelDumps.html]`

## Exact Repository Changes

| File | Planned responsibility |
|---|---|
| `deploy/ansible/roles/autographs_deploy/defaults/main.yml` | Add mapper name, cipher/key-size, coredump paths, memory margin, and opt-in cutover/reboot flags. `[VERIFIED: codebase]` |
| `deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml` | New idempotent coredump/kdump and encrypted-swap desired-state tasks; include from `tasks/main.yml`. `[VERIFIED: codebase pattern]` |
| `deploy/ansible/playbooks/kernel-persistence-cutover.yml` | Thin, explicitly invoked maintenance playbook for the guarded first conversion and reboot proof. `[VERIFIED: AGENTS.md thin-playbook rule]` |
| `deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2` | Add `LimitCORE=0`. `[VERIFIED: codebase]` |
| `deploy/ansible/roles/autographs_deploy/templates/autographs-coredump.conf.j2` | Render `Storage=none` and `ProcessSizeMax=0`. `[CITED: systemd-coredump documentation above]` |
| `controller/tests/runtime_kernel_persistence.rs` | Fast contract test for Quadlet limit, crypttab/fstab ownership, no raw swap entry, and non-ignored kdump control. `[VERIFIED: existing Rust contract-test pattern in controller/tests/caddy_static_routes.rs]` |
| `.github/workflows/ci.yml` | Add the new playbook to syntax checks; existing `ansible-lint deploy/ansible/` already covers it. `[VERIFIED: .github/workflows/ci.yml]` |
| `docs/deployment-runbook.md`, `docs/configuration-contract.md` | Replace plaintext-swap wording and document cutover, verification, rollback boundary, and reboot behavior. `[VERIFIED: current docs still declare raw /.swapfile]` |

## Verification and Rollback Gate

CI commands:

```bash
cargo test --manifest-path controller/Cargo.toml --test runtime_kernel_persistence
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote \
  ANSIBLE_CONFIG=deploy/ansible/ansible.cfg \
  ansible-playbook --syntax-check \
  deploy/ansible/playbooks/deploy.yml \
  deploy/ansible/playbooks/kernel-persistence-cutover.yml
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote \
  ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/
```

The local environment has Ansible Core 2.19.0, ansible-lint 25.6.1, `ansible.posix` 2.1.0, and `containers.podman` 1.17.0; `cryptsetup`, `systemd`, and `grubby` are target-only and must be probed on production before cutover. `[VERIFIED: local tool audit 2026-08-28]`

After the operator playbook and reboot, require all of these read-only checks:

```bash
sudo cryptsetup status autographs-swap
sudo swapon --show=NAME,TYPE,SIZE,USED
sudo systemctl is-active 'systemd-cryptsetup@autographs\x2dswap.service'
sudo systemctl show autographs-controller.service -p LimitCORE
pid="$(sudo podman inspect --format '{{.State.Pid}}' autographs-controller)"
sudo grep '^Max core file size' "/proc/${pid}/limits"
sudo systemctl is-enabled kdump.service || true
sudo systemctl is-active kdump.service || true
cat /sys/kernel/kexec_crash_loaded
grep -o 'crashkernel[^ ]*' /proc/cmdline || true
curl --fail --silent --show-error https://autographs.jetsaredim.net/admin/api/health
```

Expected: mapper `PLAIN` over `/.swapfile`; only the mapper is active swap; controller `LimitCORE=0` and process soft/hard core limits are zero; kdump is masked/inactive; `kexec_crash_loaded` is `0`; no `crashkernel` token; health passes. `[CITED: systemd and Oracle Linux sources above]` Capture the dm-crypt swap UUID before and after reboot with `blkid -s UUID -o value /dev/mapper/autographs-swap`; it should change because `swap` runs `mkswap` at every activation, while the `/dev/urandom` crypttab contract establishes a fresh key. Never run or log `dmsetup table --showkeys`. `[CITED: https://www.freedesktop.org/software/systemd/man/250/crypttab.html]`

Run one disposable host crash probe with `systemd-run -p LimitCORE=infinity` and compare `/var/lib/systemd/coredump` file inventory before/after; a nonzero probe exit is expected, but no new core body may appear. `[CITED: https://www.freedesktop.org/software/systemd/man/250/systemd-coredump.socket.html]`

If SSH does not return after reboot, recovery through OCI serial/console access cannot be safely automated from the repo. `nofail` should prevent swap setup alone from blocking boot, but the operator must retain console access and the saved fstab/crypttab/grubby evidence. `[ASSUMED: operator has tenancy-level console recovery authority]` Post-secret-rotation rollback must prefer temporarily running with no swap; it must never restore plaintext swap containing current secrets.

## Common Pitfalls

- **Overwriting active swap:** can corrupt memory; require successful `swapoff` before any write. `[CITED: https://docs.oracle.com/en/operating-systems/oracle-linux/6/admin/about-swap-space.html]`
- **Treating tmpfs as non-pageable:** tmpfs and process pages can reach swap, so wallet tmpfs is safe only after this gate. `[VERIFIED: Spike 004]`
- **Using only `Storage=none`:** systemd still recommends `ProcessSizeMax=0` to disable processing. `[CITED: systemd-coredump documentation]`
- **Only disabling kdump service:** boot configuration and the running capture-kernel state must also be verified after reboot. `[CITED: Oracle Linux kdump and grubby documentation]`
- **Printing the dm-crypt key:** `dmsetup --showkeys` defeats the evidence boundary; prove configuration, mapper status, boot ID, and swap UUID change instead.

## Security Domain

OWASP ASVS 5.0 is the current stable version and includes Stored Cryptography, Error Handling/Logging, Data Protection, and Configuration categories. `[CITED: https://owasp.org/www-project-application-security-verification-standard/]` This slice maps primarily to V6 (kernel dm-crypt rather than custom crypto), V7/V8 (no core/vmcore persistence of secrets), and V14 (Ansible-owned, reboot-verified configuration); V2/V3/V5 are unchanged.

## Assumptions Log

| # | Assumption | Risk if wrong |
|---|---|---|
| A1 | Oracle Linux package is named `cryptsetup`. | Cutover playbook fails before mutation; verify with `dnf info cryptsetup`. |
| A2 | A 256 MiB free-memory margin is adequate for this single VM. | `swapoff` may fail or create OOM risk; abort safely and choose a larger margin/maintenance reduction. |
| A3 | Operator retains OCI console recovery authority. | A post-reboot SSH failure requires external help. |

## Sources

- [systemd `crypttab`](https://www.freedesktop.org/software/systemd/man/250/crypttab.html) — file-backed encrypted devices, `/dev/urandom`, `swap`, generator ordering, destructive warning.
- [systemd-coredump](https://www.freedesktop.org/software/systemd/man/250/systemd-coredump.socket.html) — `LimitCORE`, drop-ins, `Storage=none ProcessSizeMax=0`.
- [Oracle Linux 10 boot parameters](https://docs.oracle.com/en/operating-systems/oracle-linux/10/boot/boot-UsinggrubbyToManageKernels_change_kernel_command_line_boot_parameters.html) — `grubby` and `/proc/cmdline` verification.
- [Oracle Linux kdump](https://docs.oracle.com/en/operating-systems/oracle-linux/8/boot/monitoring-WorkingWithKernelDumps.html) and [configuration](https://docs.oracle.com/en/operating-systems/oracle-linux/10/boot/monitoring-ConfiguringKdump.html) — memory capture and OCI dump paths.
- [Ansible systemd_service](https://docs.ansible.com/projects/ansible/latest/collections/ansible/builtin/systemd_service_module.html), [sysctl](https://docs.ansible.com/projects/ansible/latest/collections/ansible/posix/sysctl_module.html), and [reboot](https://docs.ansible.com/projects/ansible-core/2.17/collections/ansible/builtin/reboot_module.html) — idempotent service/sysctl/reboot patterns.

## Metadata

**Confidence breakdown:** steady-state architecture HIGH; first live conversion MEDIUM until target package/cipher support and OOM margin are preflighted; rollback MEDIUM because OCI console access is external.

**Valid until:** 2026-09-27
