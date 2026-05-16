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

if [[ ! "$DEPLOY_PATH" =~ ^/opt/autographs(/[A-Za-z0-9_-][A-Za-z0-9._-]*)*$ ]]; then
  echo "DEPLOY_PATH must be /opt/autographs or a safe child path: ${DEPLOY_PATH}" >&2
  exit 1
fi

enable_oracle_epel() {
  if ! [ -r /etc/os-release ]; then
    echo "/etc/os-release is required to enable Oracle Linux EPEL repositories" >&2
    exit 1
  fi

  . /etc/os-release

  case "${VERSION_ID%%.*}" in
    8)
      dnf install -y oracle-epel-release-el8
      dnf config-manager --enable ol8_developer_EPEL
      ;;
    9)
      dnf install -y oracle-epel-release-el9
      dnf config-manager --enable ol9_developer_EPEL
      ;;
    10)
      dnf install -y oracle-epel-release-el10
      dnf config-manager --enable $(grep -E '^\[' $(rpm -ql oracle-epel-release-el10 | grep repo$) | tr -d '[]')
      ;;
    *)
      echo "Unsupported Oracle Linux version for podman-compose bootstrap: ${VERSION_ID}" >&2
      exit 1
      ;;
  esac
}

install_podman_compose() {
  if command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1 && command -v podman-compose >/dev/null 2>&1 && podman-compose version >/dev/null 2>&1; then
    return
  fi

  if ! command -v dnf >/dev/null 2>&1; then
    echo "dnf is required to install Podman on this runtime image" >&2
    exit 1
  fi

  if ! rpm -q dnf-plugins-core >/dev/null 2>&1; then
    dnf install -y dnf-plugins-core
  fi

  if ! rpm -q podman >/dev/null 2>&1; then
    dnf install -y podman
  fi

  if ! rpm -q podman-compose >/dev/null 2>&1; then
    enable_oracle_epel
    dnf install -y podman-compose
  fi
}

configure_swap() {
  local swap_file="/.swapfile"
  local swap_size_mib=2048
  local current_size_mib=0

  if [ -f "$swap_file" ]; then
    current_size_mib=$(du -m "$swap_file" | awk '{print $1}')
  fi

  if [ "$current_size_mib" -lt "$swap_size_mib" ]; then
    if swapon --show=NAME --noheadings | grep -Fxq "$swap_file"; then
      swapoff "$swap_file"
    fi

    rm -f "$swap_file"
    fallocate -l "${swap_size_mib}M" "$swap_file"
    chmod 600 "$swap_file"
    mkswap "$swap_file"
  fi

  if ! swapon --show=NAME --noheadings | grep -Fxq "$swap_file"; then
    swapon "$swap_file"
  fi

  if ! grep -Eq '^[^#[:space:]]+[[:space:]]+none[[:space:]]+swap[[:space:]]' /etc/fstab; then
    printf '%s\n' "$swap_file none swap sw 0 0" >>/etc/fstab
  elif ! grep -Eq "^${swap_file//\//\\/}[[:space:]]+none[[:space:]]+swap[[:space:]]" /etc/fstab; then
    sed -i.bak -E "s|^[^#[:space:]]+[[:space:]]+none[[:space:]]+swap[[:space:]].*|$swap_file none swap sw 0 0|" /etc/fstab
  fi

  cat >/etc/sysctl.d/99-autographs-swap.conf <<'SYSCTL'
vm.swappiness=20
SYSCTL
  sysctl --system >/dev/null
}

configure_swap
install_podman_compose

install -d -o "$DEPLOY_USER" -m 0755 "$DEPLOY_PATH" "$DEPLOY_PATH/compose" "$DEPLOY_PATH/caddy"
install -d -o "$DEPLOY_USER" -m 0700 "$DEPLOY_PATH/secrets"

if command -v firewall-cmd >/dev/null 2>&1 && systemctl is-active --quiet firewalld; then
  firewall-cmd --permanent --add-service=http
  firewall-cmd --permanent --add-service=https
  firewall-cmd --reload
fi
