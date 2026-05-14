#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR"

corepack pnpm --filter app db:migrate
corepack pnpm --filter app db:seed
corepack pnpm --filter app data:smoke
