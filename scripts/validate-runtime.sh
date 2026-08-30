#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
cd "${repo_root}"

required_artifacts=(
  deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml
  deploy/ansible/roles/autographs_deploy/templates/autographs-coredump.conf.j2
  deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2
  deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml
  controller/tests/runtime_kernel_persistence.rs
)

for artifact in "${required_artifacts[@]}"; do
  if [[ ! -f "${artifact}" ]]; then
    echo "missing runtime persistence artifact: ${artifact}" >&2
    exit 1
  fi
done

cargo test --manifest-path controller/Cargo.toml --test runtime_kernel_persistence
