#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
fixture="$(mktemp -d)"
container="autographs-media-origin-probe-$$"
started=false

cleanup() {
  if [[ "${started}" == true ]]; then
    docker stop "${container}" >/dev/null 2>&1 || true
  fi
  rm -rf -- "${fixture}"
}
trap cleanup EXIT

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required for the Caddy HTTP origin proof" >&2
  exit 1
fi
if ! command -v ansible-playbook >/dev/null 2>&1; then
  echo "ansible-playbook is required to render the frozen Caddyfile" >&2
  exit 1
fi

AUTOGRAPHS_FREEZE_TEST_DIR="${fixture}" \
  ANSIBLE_LOCAL_TEMP=/tmp/ansible-local \
  ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote \
  ANSIBLE_CONFIG="${repo_root}/deploy/ansible/ansible.cfg" \
  ansible-playbook "${repo_root}/deploy/ansible/playbooks/publish-freeze-render-test.yml" >/dev/null

# Use the repository Caddyfile with only the public site label changed to local
# HTTP; the media handlers and the preview site are otherwise unmodified.
for state in normal frozen; do
  if [[ "${state}" == normal ]]; then
    source_caddyfile="${fixture}/Caddyfile"
  else
    source_caddyfile="${fixture}/Caddyfile.frozen"
  fi
  awk '
    NR <= 3 { next }
    /^\{\$AUTOGRAPHS_DOMAIN\} \{$/ { $0 = "http://media-probe.invalid {" }
    { print }
  ' "${source_caddyfile}" > "${fixture}/Caddyfile.${state}-http"
done
cp "${fixture}/Caddyfile.normal-http" "${fixture}/Caddyfile.http"

static_root="${fixture}/static"
media_a="/media/card/image-1-detail-aaaaaaaaaaaaaaaa.webp"
media_b="/media/card/image-1-detail-bbbbbbbbbbbbbbbb.webp"
mkdir -p "${static_root}/releases/a/media/card" "${static_root}/releases/b/media/card" \
  "${static_root}/media/card"
printf 'committed A\n' > "${static_root}/releases/a${media_a}"
printf 'candidate B\n' > "${static_root}/releases/b${media_b}"
cp "${static_root}/releases/a${media_a}" "${static_root}${media_a}"
ln -s releases/a "${static_root}/current"

docker run --detach --rm --name "${container}" \
  --publish 127.0.0.1::80 --publish 127.0.0.1::8081 \
  --volume "${fixture}/Caddyfile.http:/etc/caddy/Caddyfile:ro" \
  --volume "${static_root}:/srv/autographs/static:ro" \
  docker.io/library/caddy:2-alpine >/dev/null
started=true
docker exec "${container}" caddy adapt --config /etc/caddy/Caddyfile >/dev/null
docker exec "${container}" caddy validate --config /etc/caddy/Caddyfile >/dev/null

public_port="$(docker port "${container}" 80/tcp)"
preview_port="$(docker port "${container}" 8081/tcp)"
public_base="http://127.0.0.1:${public_port##*:}"
preview_base="http://127.0.0.1:${preview_port##*:}"

http_status() {
  local site="$1" path="$2" method="${3:-GET}"
  if [[ "${site}" == public ]]; then
    curl --silent --output /dev/null --max-time 5 --request "${method}" \
      --header 'Host: media-probe.invalid' \
      --write-out '%{http_code}' "${public_base}${path}"
  else
    curl --silent --output /dev/null --max-time 5 --request "${method}" --write-out '%{http_code}' \
      "${preview_base}${path}"
  fi
}

expect_status() {
  local expected="$1" path="$2" label="$3" site actual
  for site in public preview; do
    actual="$(http_status "${site}" "${path}")"
    if [[ "${actual}" != "${expected}" ]]; then
      echo "${label}: ${site} ${path} returned ${actual}, expected ${expected}" >&2
      exit 1
    fi
  done
  echo "${label}: public and preview returned ${expected}"
}

for attempt in {1..30}; do
  if [[ "$(http_status preview "${media_a}" 2>/dev/null || true)" == 200 ]]; then
    break
  fi
  sleep 0.2
done

expect_status 200 "${media_a}" 'committed A'
expect_status 404 "${media_b}" 'candidate B before pointer commit'
ln -s releases/b "${static_root}/.current-next"
mv -Tf "${static_root}/.current-next" "${static_root}/current"
expect_status 200 "${media_b}" 'B immediately after pointer commit, before shared copy'
expect_status 200 "${media_a}" 'A through shared retention'
cp "${static_root}/releases/b${media_b}" "${static_root}${media_b}"
mkdir -p "${static_root}/.media-tombstones/media/card"
: > "${static_root}/.media-tombstones${media_b}"
expect_status 404 "${media_b}" 'B tombstoned with shared and current copies'
rm -- "${static_root}${media_b}"
expect_status 404 "${media_b}" 'B tombstoned with current fallback copy'
expect_status 200 "${media_a}" 'A remains available'
expect_status 404 "/.media-tombstones${media_b}" 'marker path remains private'

cat "${fixture}/Caddyfile.frozen-http" > "${fixture}/Caddyfile.http"
docker exec "${container}" caddy validate --config /etc/caddy/Caddyfile >/dev/null
docker exec "${container}" caddy reload --config /etc/caddy/Caddyfile >/dev/null
for path in /admin/api/publish/full /admin/api/publish/incremental; do
  actual="$(http_status public "${path}" POST)"
  [[ "${actual}" == 503 ]] || { echo "frozen ${path} returned ${actual}, expected 503" >&2; exit 1; }
done
status_result="$(http_status public /admin/api/publish/status)"
[[ "${status_result}" != 503 ]] || { echo 'publish status was frozen' >&2; exit 1; }
echo 'Ansible-rendered freeze: both publish POSTs 503; status remains proxied'

cat "${fixture}/Caddyfile.normal-http" > "${fixture}/Caddyfile.http"
docker exec "${container}" caddy validate --config /etc/caddy/Caddyfile >/dev/null
docker exec "${container}" caddy reload --config /etc/caddy/Caddyfile >/dev/null
for path in /admin/api/publish/full /admin/api/publish/incremental; do
  actual="$(http_status public "${path}" POST)"
  [[ "${actual}" != 503 ]] || { echo "normal ${path} remained frozen" >&2; exit 1; }
done
echo 'Ansible-rendered unfreeze: publish POSTs again reach the controller proxy'
echo 'Caddy origin media commit and tombstone HTTP matrix passed'
