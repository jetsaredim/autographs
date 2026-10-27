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
bytes_a='RIFF0000WEBPcommitted A'
bytes_b='RIFF0000WEBPcandidate B'
hash_a="$(printf '%s' "${bytes_a}" | sha256sum | cut -c 1-16)"
hash_b="$(printf '%s' "${bytes_b}" | sha256sum | cut -c 1-16)"
media_a="/media/card/image-1-detail-${hash_a}.webp"
media_b="/media/card/image-1-detail-${hash_b}.webp"
mkdir -p "${static_root}/releases/a/media/card" \
  "${static_root}/media/card"
printf '%s' "${bytes_a}" > "${static_root}/releases/a${media_a}"
printf '{"release_id":"a","artifacts":[{"path":"%s","variant":"detail","content_type":"image/webp","byte_size":%s}]}' \
  "${media_a#/}" "${#bytes_a}" > "${static_root}/releases/a/manifest.json"
cp "${static_root}/releases/a${media_a}" "${static_root}${media_a}"
ln -s releases/a "${static_root}/current"
audit="${repo_root}/deploy/ansible/roles/autographs_deploy/files/publish_freeze_audit.py"
audit_args=(--static-root "${static_root}" --quarantine "${fixture}/private/quarantine" \
  --record "${fixture}/private/freeze.json" --release-tag v1.2.3 \
  --controller-digest "sha256:$(printf 'a%.0s' {1..64})" --old-container-id old --old-pid 123)
python3 "${audit}" "${audit_args[@]}" --initial-cutover --old-route-current-only >/dev/null
mkdir -p "${static_root}/releases/b/media/card"
printf '%s' "${bytes_b}" > "${static_root}/releases/b${media_b}"
printf '{"release_id":"b","artifacts":[{"path":"%s","variant":"detail","content_type":"image/webp","byte_size":%s}]}' \
  "${media_b#/}" "${#bytes_b}" > "${static_root}/releases/b/manifest.json"

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
python3 -c 'import hashlib,json,pathlib,sys; r=pathlib.Path(sys.argv[1]); old=json.loads((r/".media-commit-history/history.json").read_text()); name=sys.argv[2].lstrip("/"); new={"version":1,"old_release_id":"a","new_release_id":"b","old_manifest_sha256":old["releases"]["a"]["manifest_sha256"],"new_manifest_sha256":hashlib.sha256((r/"releases/b/manifest.json").read_bytes()).hexdigest(),"new_media":{name:hashlib.sha256((r/name).read_bytes()).hexdigest() if (r/name).exists() else hashlib.sha256((r/"releases/b"/name).read_bytes()).hexdigest()}}; (r/".media-commit-history/intent.json").write_text(json.dumps(new))' "${static_root}" "${media_b}"
ln -s releases/b "${static_root}/.current-next"
mv -Tf "${static_root}/.current-next" "${static_root}/current"
expect_status 200 "${media_b}" 'B immediately after pointer commit, before shared copy'
expect_status 200 "${media_a}" 'A through shared retention'
python3 "${audit}" "${audit_args[@]}" --frozen-exit-proved >/dev/null
python3 "${audit}" "${audit_args[@]}" --frozen-exit-proved >/dev/null
expect_status 200 "${media_a}" 'A after later preflight and role reaudit'
cp "${static_root}/releases/b${media_b}" "${static_root}${media_b}"
python3 "${repo_root}/deploy/ansible/roles/autographs_deploy/files/static_release_rollback.py" \
  --static-root "${static_root}" --target-release-id a --frozen-exit-proved
python3 "${audit}" "${audit_args[@]}" --frozen-exit-proved >/dev/null
expect_status 200 "${media_a}" 'A restored by journaled rollback'
expect_status 200 "${media_b}" 'B retained after rollback'
mkdir -p "${static_root}/.media-tombstones/media/card"
: > "${static_root}/.media-tombstones${media_b}"
expect_status 404 "${media_b}" 'B tombstoned with shared and retained copies'
rm -- "${static_root}${media_b}"
expect_status 404 "${media_b}" 'B tombstoned with retained release copy'
expect_status 200 "${media_a}" 'A remains available'
touch "${static_root}/.media-tombstones${media_a}"
rm -- "${static_root}${media_a}" "${static_root}/current${media_a}"
python3 "${audit}" "${audit_args[@]}" --frozen-exit-proved >/dev/null
expect_status 404 "${media_a}" 'marked active A deleted before later reaudit'
expect_status 404 "/.media-tombstones${media_b}" 'marker path remains private'
expect_status 404 '/.media-commit-history/history.json' 'publication history remains private'

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
