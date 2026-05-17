#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR/app"

NODE_OPTIONS="${NODE_OPTIONS:---max-old-space-size=384}" corepack pnpm db:migrate
NODE_OPTIONS="${NODE_OPTIONS:---max-old-space-size=384}" corepack pnpm db:seed
NODE_OPTIONS="${NODE_OPTIONS:---max-old-space-size=384}" corepack pnpm data:smoke
