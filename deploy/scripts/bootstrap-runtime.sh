#!/usr/bin/env bash

set -euo pipefail

DEPLOY_USER="${DEPLOY_USER:-opc}"
DEPLOY_PATH="${DEPLOY_PATH:-/opt/autographs}"

if [ "$(id -u)" -ne 0 ]; then
  echo "bootstrap-runtime.sh must run as root" >&2
  exit 1
fi

if ! id "$DEPLOY_USER" >/dev/null 2>&1; then
  echo "Deploy user does not exist: ${DEPLOY_USER}" >&2
  exit 1
fi

install_docker() {
  if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    return
  fi

  if ! command -v dnf >/dev/null 2>&1; then
    echo "dnf is required to install Docker on this runtime image" >&2
    exit 1
  fi

  dnf install -y dnf-plugins-core
  dnf config-manager --add-repo=https://download.docker.com/linux/centos/docker-ce.repo
  dnf install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
}

install_docker
systemctl enable --now docker

usermod -aG docker "$DEPLOY_USER"

install -d -o "$DEPLOY_USER" -m 0755 "$DEPLOY_PATH" "$DEPLOY_PATH/compose" "$DEPLOY_PATH/nginx"

if command -v firewall-cmd >/dev/null 2>&1 && systemctl is-active --quiet firewalld; then
  firewall-cmd --permanent --add-service=http
  firewall-cmd --permanent --add-service=https
  firewall-cmd --reload
fi
