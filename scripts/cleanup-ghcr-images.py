#!/usr/bin/env python3

import base64
import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path


API_VERSION = "2022-11-28"
SEMVER_TAG_RE = r"^v[0-9]+\.[0-9]+\.[0-9]+$"


def main() -> int:
    token = require_env("GITHUB_TOKEN")
    image_repository = require_env("GHCR_IMAGE_REPOSITORY")
    retain_count = parse_positive_int(os.getenv("GHCR_CLEANUP_RETAIN_TAGGED"), 10)
    retain_semver_count = parse_positive_int(
        os.getenv("GHCR_CLEANUP_RETAIN_SEMVER_TAGGED"),
        retain_count,
    )
    min_age_days = parse_positive_int(os.getenv("GHCR_CLEANUP_MIN_AGE_DAYS"), 7)
    protected_tags = parse_csv_set(os.getenv("GHCR_CLEANUP_PROTECTED_TAGS"))
    status = read_release_status()
    dry_run = os.getenv("GHCR_CLEANUP_DRY_RUN", "true").lower() != "false"
    for field in ("latestRepositoryVersion", "deployedRepositoryVersion", "deployedControllerVersion", "deployedControllerDigest", "previousControllerVersion", "previousControllerDigest"):
        print(f"{field}: {status.get(field, '')}")

    package_info = parse_ghcr_image_repository(image_repository)
    versions = list_package_versions(token, package_info)
    cutoff_time = time.time() - min_age_days * 24 * 60 * 60

    resolve_children = make_child_resolver(token, package_info)
    selections = select_versions(
        versions, status, retain_count, retain_semver_count, cutoff_time, protected_tags, resolve_children
    )
    candidate_count = 0
    failed_count = 0
    # Print the complete inventory before the first deletion.
    for version, reasons in selections:
        tags = version.get("metadata", {}).get("container", {}).get("tags", [])
        print(f"GHCR version {version['id']}: {format_tags(tags)} ({', '.join(reasons) if reasons else 'deletion candidate'})")
    for version, reasons in selections:
        if reasons:
            continue
        if dry_run:
            print(f"Would delete GHCR version {version['id']}")
        else:
            if delete_package_version(token, package_info, str(version["id"])):
                print(f"Deleted GHCR version {version['id']}")
            else:
                failed_count += 1
                print(f"GHCR version {version['id']} returned 404: already deleted, or the token lacks permission to delete it")
        candidate_count += 1
    deleted_count = candidate_count - failed_count
    summary = f"GHCR cleanup complete for {image_repository}: {deleted_count} {'would delete' if dry_run else 'deleted'}, {len(versions) - candidate_count} kept"
    if failed_count:
        summary += f", {failed_count} not deleted (404)"
    print(summary)
    return 1 if failed_count else 0


def read_release_status() -> dict:
    status_path = Path(os.getenv("GHCR_CLEANUP_RELEASE_STATUS_PATH", ".release-status.json"))
    status = json.loads(status_path.read_text(encoding="utf-8"))
    import re
    if not isinstance(status, dict) or not re.fullmatch(SEMVER_TAG_RE, status.get("deployedControllerVersion", "")):
        raise RuntimeError("Valid deployed controller status is required before cleanup")
    return status


def select_versions(versions, status, retain_count, retain_semver_count, cutoff_time, protected_tags, resolve_children):
    """Return (version, keep_reasons) pairs; an empty reasons list means "delete".

    resolve_children(version) returns the digests an image index references
    (platform images and attestation manifests). Untagged versions are only
    deleted when no retained version references them.
    """
    newest_first = sorted(versions, key=lambda version: parse_created_at(version["created_at"]), reverse=True)

    def tags_of(version):
        return version.get("metadata", {}).get("container", {}).get("tags", [])

    deployed_tag = status.get("deployedControllerVersion", "")
    if not any(deployed_tag in tags_of(version) for version in versions):
        raise RuntimeError(f"Deployed controller tag {deployed_tag} was not found in GHCR; refusing to clean up")

    # Rank tagged versions only, so untagged child manifests do not eat into the retention count.
    tagged_rank = {
        str(version["id"]): rank
        for rank, version in enumerate(version for version in newest_first if tags_of(version))
    }
    newest_semver_ids = {
        str(version["id"])
        for version in [version for version in newest_first if semver_tags(tags_of(version))][:retain_semver_count]
    }

    selections = []
    for version in newest_first:
        tags = tags_of(version)
        created_at = parse_created_at(version["created_at"])
        keep_reasons = []

        if tagged_rank.get(str(version["id"]), retain_count) < retain_count:
            keep_reasons.append(f"among {retain_count} newest tagged versions")

        for prefix in ("deployed", "previous"):
            tag = status.get(f"{prefix}ControllerVersion", "")
            digest = status.get(f"{prefix}ControllerDigest", "")
            if tag and tag in tags:
                keep_reasons.append(f"{prefix} controller tag {tag}")
            if digest and version.get("name") == digest:
                keep_reasons.append(f"{prefix} controller digest {digest}")

        if str(version["id"]) in newest_semver_ids:
            keep_reasons.append(f"among {retain_semver_count} newest semver-tagged versions")

        matching_protected_tags = sorted(set(tags) & protected_tags)
        if matching_protected_tags:
            keep_reasons.append(f"protected tag {', '.join(matching_protected_tags)}")

        if created_at > cutoff_time:
            keep_reasons.append("newer than minimum age")
        selections.append((version, keep_reasons))

    # Anything a retained version points at must survive too. If a manifest cannot be
    # read, resolve_children raises and the whole run aborts before deleting anything.
    referenced = {}
    for version, keep_reasons in selections:
        if keep_reasons:
            for child_digest in resolve_children(version):
                referenced.setdefault(child_digest, version["id"])
    for version, keep_reasons in selections:
        if not keep_reasons and version.get("name") in referenced:
            keep_reasons.append(f"referenced by retained version {referenced[version['name']]}")
    return selections


def make_child_resolver(token: str, package_info: dict[str, str]):
    repo_path = f"{package_info['owner']}/{package_info['package_name']}".lower()
    actor = os.getenv("GITHUB_ACTOR", "github")
    cache: dict[str, str] = {}
    accept = ", ".join([
        "application/vnd.oci.image.index.v1+json",
        "application/vnd.docker.distribution.manifest.list.v2+json",
        "application/vnd.oci.image.manifest.v1+json",
        "application/vnd.docker.distribution.manifest.v2+json",
    ])

    def registry_token() -> str:
        if "token" not in cache:
            query = urllib.parse.urlencode({"service": "ghcr.io", "scope": f"repository:{repo_path}:pull"})
            basic = base64.b64encode(f"{actor}:{token}".encode()).decode()
            status, body = http_get(f"https://ghcr.io/token?{query}", {"Authorization": f"Basic {basic}"})
            if status != 200:
                raise RuntimeError(f"Could not get a GHCR registry token: {status} {body}")
            cache["token"] = json.loads(body)["token"]
        return cache["token"]

    def child_digests(version: dict) -> list[str]:
        digest = version["name"]
        status, body = http_get(
            f"https://ghcr.io/v2/{repo_path}/manifests/{digest}",
            {"Authorization": f"Bearer {registry_token()}", "Accept": accept},
        )
        if status != 200:
            raise RuntimeError(f"Could not read manifest {digest} to prove what it references: {status} {body[:200]}")
        document = json.loads(body)
        return [entry["digest"] for entry in document.get("manifests", [])]

    return child_digests


def http_get(url: str, headers: dict[str, str]) -> tuple[int, str]:
    request = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.status, response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        return error.code, error.read().decode("utf-8")


def require_env(name: str) -> str:
    value = os.getenv(name)
    if not value:
        raise RuntimeError(f"{name} is required")
    return value


def parse_positive_int(value: str | None, fallback: int) -> int:
    try:
        parsed = int(value or "")
    except ValueError:
        return fallback
    return parsed if parsed > 0 else fallback


def parse_csv_set(value: str | None) -> set[str]:
    return {item.strip() for item in (value or "").split(",") if item.strip()}


def deployed_controller_version_from_status() -> str:
    explicit = os.getenv("GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION", "").strip()
    if explicit:
        return explicit
    status_path = Path(os.getenv("GHCR_CLEANUP_RELEASE_STATUS_PATH", ".release-status.json"))
    if not status_path.exists():
        return ""
    try:
        status = json.loads(status_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return ""
    value = status.get("deployedControllerVersion")
    return value.strip() if isinstance(value, str) else ""


def semver_tags(tags: list[str]) -> list[str]:
    import re

    return [tag for tag in tags if re.fullmatch(SEMVER_TAG_RE, tag)]


def parse_ghcr_image_repository(repository: str) -> dict[str, str]:
    normalized = repository.removeprefix("https://").removeprefix("http://")
    parts = [part for part in normalized.split("/") if part]

    if len(parts) < 3 or parts[0] != "ghcr.io":
        raise RuntimeError(f"GHCR_IMAGE_REPOSITORY must look like ghcr.io/owner/package, got {repository}")

    return {
        "owner": parts[1],
        "package_name": "/".join(parts[2:]),
        "owner_kind": "",
    }


def list_package_versions(token: str, package_info: dict[str, str]) -> list[dict]:
    for owner_kind in ("orgs", "users"):
        package_info["owner_kind"] = owner_kind
        status, headers, body = github_request(token, package_info)

        if status == 404:
            continue

        if status < 200 or status >= 300:
            raise RuntimeError(f"Failed to list GHCR versions: {status} {body}")

        return paginate(token, package_info, json.loads(body), headers.get("link"))

    raise RuntimeError(
        f"Could not find GHCR package {package_info['owner']}/{package_info['package_name']} as an org or user package"
    )


def paginate(token: str, package_info: dict[str, str], first_page: list[dict], first_link_header: str | None) -> list[dict]:
    pages = list(first_page)
    next_url = get_next_url(first_link_header)

    while next_url:
        status, headers, body = github_request(token, package_info, next_url)

        if status < 200 or status >= 300:
            raise RuntimeError(f"Failed to paginate GHCR versions: {status} {body}")

        pages.extend(json.loads(body))
        next_url = get_next_url(headers.get("link"))

    return pages


def delete_package_version(token: str, package_info: dict[str, str], version_id: str) -> bool:
    status, _, body = github_request(token, package_info, version_id, method="DELETE")

    if status == 204:
        return True
    if status == 404:
        return False
    raise RuntimeError(f"Failed to delete GHCR version {version_id}: {status} {body}")


def github_request(
    token: str,
    package_info: dict[str, str],
    suffix_or_url: str | None = None,
    method: str = "GET",
) -> tuple[int, dict[str, str], str]:
    package_name = urllib.parse.quote(package_info["package_name"], safe="")
    base_path = (
        f"{package_info['owner_kind']}/{package_info['owner']}/packages/container/{package_name}/versions"
    )

    if suffix_or_url and suffix_or_url.startswith("https://"):
        url = suffix_or_url
    else:
        suffix = f"/{suffix_or_url}" if suffix_or_url else "?per_page=100"
        url = f"https://api.github.com/{base_path}{suffix}"

    request = urllib.request.Request(
        url,
        method=method,
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "X-GitHub-Api-Version": API_VERSION,
        },
    )

    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.status, normalize_headers(response.headers), response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        return error.code, normalize_headers(error.headers), error.read().decode("utf-8")


def normalize_headers(headers) -> dict[str, str]:
    return {key.lower(): value for key, value in headers.items()}


def get_next_url(link_header: str | None) -> str | None:
    if not link_header:
        return None

    for link in (part.strip() for part in link_header.split(",")):
        if link.endswith('rel="next"'):
            start = link.find("<")
            end = link.find(">")
            if start != -1 and end != -1:
                return link[start + 1 : end]

    return None


def parse_created_at(value: str) -> float:
    return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc).timestamp()


def format_tags(tags: list[str]) -> str:
    return ", ".join(tags) if tags else "untagged"


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
