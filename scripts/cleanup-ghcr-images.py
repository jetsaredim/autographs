#!/usr/bin/env python3

import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone


API_VERSION = "2022-11-28"


def main() -> int:
    token = require_env("GITHUB_TOKEN")
    image_repository = require_env("GHCR_IMAGE_REPOSITORY")
    retain_count = parse_positive_int(os.getenv("GHCR_CLEANUP_RETAIN_TAGGED"), 10)
    min_age_days = parse_positive_int(os.getenv("GHCR_CLEANUP_MIN_AGE_DAYS"), 7)
    protected_tags = parse_csv_set(os.getenv("GHCR_CLEANUP_PROTECTED_TAGS"))
    dry_run = os.getenv("GHCR_CLEANUP_DRY_RUN", "false").lower() == "true"

    package_info = parse_ghcr_image_repository(image_repository)
    versions = list_package_versions(token, package_info)
    cutoff_time = time.time() - min_age_days * 24 * 60 * 60

    newest_first = sorted(versions, key=lambda version: parse_created_at(version["created_at"]), reverse=True)
    deleted_count = 0

    for index, version in enumerate(newest_first):
        tags = version.get("metadata", {}).get("container", {}).get("tags", [])
        created_at = parse_created_at(version["created_at"])
        keep_reasons = []

        if index < retain_count:
            keep_reasons.append(f"among {retain_count} newest versions")

        if "latest" in tags:
            keep_reasons.append("latest tag")

        matching_protected_tags = sorted(set(tags) & protected_tags)
        if matching_protected_tags:
            keep_reasons.append(f"protected tag {', '.join(matching_protected_tags)}")

        if created_at > cutoff_time:
            keep_reasons.append(f"newer than {min_age_days} days")

        if keep_reasons:
            print(f"Keeping GHCR version {version['id']}: {format_tags(tags)} ({', '.join(keep_reasons)})")
            continue

        if dry_run:
            print(f"Would delete GHCR version {version['id']}: {format_tags(tags)}")
        else:
            delete_package_version(token, package_info, str(version["id"]))
            print(f"Deleted GHCR version {version['id']}: {format_tags(tags)}")
        deleted_count += 1

    action = "would delete" if dry_run else "deleted"
    kept_count = len(versions) - deleted_count
    print(f"GHCR cleanup complete for {image_repository}: {deleted_count} {action}, {kept_count} kept")
    return 0


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


def delete_package_version(token: str, package_info: dict[str, str], version_id: str) -> None:
    status, _, body = github_request(token, package_info, version_id, method="DELETE")

    if status not in (204, 404):
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
