#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [ -z "${TERRAFORM_BIN:-}" ] && [ -x "$ROOT_DIR/.tools/terraform/terraform" ]; then
  TERRAFORM_BIN="$ROOT_DIR/.tools/terraform/terraform"
else
  TERRAFORM_BIN="${TERRAFORM_BIN:-terraform}"
fi

cd "$ROOT_DIR"

if [ -z "${TF_DATA_DIR:-}" ]; then
  TF_DATA_DIR="$(mktemp -d)"
  export TF_DATA_DIR
  trap 'rm -rf "$TF_DATA_DIR"' EXIT
fi

corepack pnpm install --frozen-lockfile
corepack pnpm --filter app lint
corepack pnpm --filter app typecheck
corepack pnpm --filter app build

"$TERRAFORM_BIN" -chdir=infra/terraform fmt -check -recursive
"$TERRAFORM_BIN" -chdir=infra/terraform/tenancy init -backend=false
"$TERRAFORM_BIN" -chdir=infra/terraform/tenancy validate
"$TERRAFORM_BIN" -chdir=infra/terraform init -backend=false
"$TERRAFORM_BIN" -chdir=infra/terraform validate
