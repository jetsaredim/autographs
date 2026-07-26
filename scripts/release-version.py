#!/usr/bin/env python3

import argparse
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path


API_VERSION = "2022-11-28"
SEMVER_RE = re.compile(r"^v([0-9]+)\.([0-9]+)\.([0-9]+)$")
README_START = "<!-- autographs-release-status:start -->"
README_END = "<!-- autographs-release-status:end -->"


def main() -> int:
    parser = argparse.ArgumentParser(description="Manage Autographs release versions.")
    subcommands = parser.add_subparsers(dest="command", required=True)

    classify_parser = subcommands.add_parser("classify")
    classify_parser.add_argument("--pr-json", type=Path)
    classify_parser.add_argument("--title", default="")
    classify_parser.add_argument("--body", default="")
    classify_parser.add_argument("--labels", default="")

    next_parser = subcommands.add_parser("next")
    next_parser.add_argument("--current", required=True)
    next_parser.add_argument("--bump", required=True, choices=["major", "minor", "patch"])

    prepare_parser = subcommands.add_parser("prepare")
    prepare_parser.add_argument("--status-file", type=Path, default=Path(".release-status.json"))
    prepare_parser.add_argument("--version-file", type=Path, default=Path("VERSION"))
    prepare_parser.add_argument("--readme", type=Path, default=Path("README.md"))
    prepare_parser.add_argument("--pr-json", type=Path)
    prepare_parser.add_argument("--controller-image-impact", required=True)
    prepare_parser.add_argument("--runtime-deploy-impact", required=True)
    prepare_parser.add_argument("--source-revision", default="")
    prepare_parser.add_argument("--github-output", type=Path)
    prepare_parser.add_argument("--committed-status-ref", default="")
    prepare_parser.add_argument("--committed-status-file", type=Path)
    prepare_parser.add_argument("--reuse-current-status", action="store_true")

    deployed_parser = subcommands.add_parser("mark-deployed")
    deployed_parser.add_argument("--status-file", type=Path, default=Path(".release-status.json"))
    deployed_parser.add_argument("--readme", type=Path, default=Path("README.md"))
    deployed_parser.add_argument("--controller-version", required=True)

    ghcr_parser = subcommands.add_parser("assert-ghcr-tag-absent")
    ghcr_parser.add_argument("--image-repository", required=True)
    ghcr_parser.add_argument("--tag", required=True)

    ghcr_exists_parser = subcommands.add_parser("ghcr-tag-exists")
    ghcr_exists_parser.add_argument("--image-repository", required=True)
    ghcr_exists_parser.add_argument("--tag", required=True)
    ghcr_exists_parser.add_argument("--github-output", type=Path)

    args = parser.parse_args()

    if args.command == "classify":
        metadata = read_pr_metadata(args.pr_json, args.title, args.body, args.labels)
        print(classify_bump(metadata))
        return 0

    if args.command == "next":
        print(next_version(args.current, args.bump))
        return 0

    if args.command == "prepare":
        prepare_release(args)
        return 0

    if args.command == "mark-deployed":
        mark_deployed(args.status_file, args.readme, args.controller_version)
        return 0

    if args.command == "assert-ghcr-tag-absent":
        assert_ghcr_tag_absent(args.image_repository, args.tag)
        return 0

    if args.command == "ghcr-tag-exists":
        exists = ghcr_tag_exists(args.image_repository, args.tag)
        if args.github_output:
            with args.github_output.open("a", encoding="utf-8") as output:
                output.write(f"exists={str(exists).lower()}\n")
        else:
            print(str(exists).lower())
        return 0

    raise RuntimeError(f"unknown command: {args.command}")


def prepare_release(args: argparse.Namespace) -> None:
    status = read_status(args.status_file)
    current_repo_version = read_version(args.version_file, status["repoVersion"])
    metadata = read_pr_metadata(args.pr_json)
    controller_image_impact = parse_bool(args.controller_image_impact)
    runtime_deploy_impact = parse_bool(args.runtime_deploy_impact)
    committed_status = read_committed_status(args)

    if args.reuse_current_status:
        reused_existing_version = True
        reused_status = status
    elif args.source_revision and status.get("sourceRevision") == args.source_revision:
        reused_existing_version = True
        reused_status = status
    elif (
        args.source_revision
        and committed_status
        and committed_status.get("sourceRevision") == args.source_revision
    ):
        reused_existing_version = True
        reused_status = committed_status
    else:
        reused_existing_version = False
        reused_status = {}

    if reused_existing_version:
        repo_version = reused_status["repoVersion"]
        bump = reused_status["lastBump"]
        deploy_impact = reused_status["lastDeployImpact"]
        source_revision = reused_status.get("sourceRevision") or args.source_revision
        deployed_controller_version = (
            repo_version
            if controller_image_impact
            else reused_status.get("deployedControllerVersion") or ""
        )
        controller_image_build = controller_image_impact
        deploy_required = controller_image_impact or runtime_deploy_impact
    else:
        bump = classify_bump(metadata)
        repo_version = next_version(current_repo_version, bump)
        source_revision = args.source_revision
        if controller_image_impact:
            deploy_impact = "controller-image"
            deployed_controller_version = repo_version
            deploy_required = True
            controller_image_build = True
        elif runtime_deploy_impact:
            deploy_impact = "runtime-config"
            deployed_controller_version = status.get("deployedControllerVersion") or ""
            if not deployed_controller_version:
                raise RuntimeError(
                    "runtime deploy impact requires an existing deployedControllerVersion"
                )
            deploy_required = True
            controller_image_build = False
        else:
            deploy_impact = "repo-only"
            deployed_controller_version = status.get("deployedControllerVersion") or ""
            deploy_required = False
            controller_image_build = False

        updated = {
            "repoVersion": repo_version,
            "deployedControllerVersion": status.get("deployedControllerVersion") or "",
            "lastBump": bump,
            "lastDeployImpact": deploy_impact,
            "sourceRevision": args.source_revision,
            "updatedAt": now_utc(),
        }

        write_status(args.status_file, updated)
        write_version(args.version_file, repo_version)
        update_readme(args.readme, updated)

    outputs = {
        "version": repo_version,
        "bump": bump,
        "deploy_impact": deploy_impact,
        "deploy_required": str(deploy_required).lower(),
        "controller_image_build": str(controller_image_build).lower(),
        "controller_deploy_version": deployed_controller_version,
        "reused_existing_version": str(bool(reused_existing_version)).lower(),
        "source_revision": source_revision,
    }
    if args.github_output:
        with args.github_output.open("a", encoding="utf-8") as output:
            for name, value in outputs.items():
                output.write(f"{name}={value}\n")
    else:
        print(json.dumps(outputs, indent=2, sort_keys=True))


def mark_deployed(status_file: Path, readme: Path, controller_version: str) -> None:
    status = read_status(status_file)
    if not SEMVER_RE.fullmatch(controller_version.strip()):
        raise RuntimeError(f"invalid deployed controller version: {controller_version!r}")
    status["deployedControllerVersion"] = controller_version.strip()
    status["updatedAt"] = now_utc()
    write_status(status_file, status)
    update_readme(readme, status)


def read_pr_metadata(
    pr_json_path: Path | None = None,
    title: str = "",
    body: str = "",
    labels: str = "",
) -> dict:
    if pr_json_path and pr_json_path.exists() and pr_json_path.stat().st_size > 0:
        data = json.loads(pr_json_path.read_text(encoding="utf-8"))
        label_values = data.get("labels", [])
        if isinstance(label_values, dict) and "nodes" in label_values:
            label_values = label_values["nodes"]
        return {
            "title": data.get("title") or "",
            "body": data.get("body") or "",
            "labels": normalize_labels(label_values),
        }
    return {
        "title": title,
        "body": body,
        "labels": normalize_labels(labels.split(",") if labels else []),
    }


def normalize_labels(labels: list | tuple) -> list[str]:
    normalized = []
    for label in labels:
        if isinstance(label, str):
            value = label
        elif isinstance(label, dict):
            value = label.get("name", "")
        else:
            value = ""
        value = value.strip()
        if value:
            normalized.append(value)
    return normalized


def classify_bump(metadata: dict) -> str:
    labels = {label.lower() for label in metadata.get("labels", [])}
    title = metadata.get("title", "")
    body = metadata.get("body", "")

    if (
        "version:major" in labels
        or re.match(r"^[a-z]+(?:\(.+\))?!:", title.strip(), re.I)
        or re.search(r"(?im)^BREAKING(?:-| )CHANGE:\s+\S", body)
    ):
        return "major"
    if "version:minor" in labels or re.match(r"^feat(?:\(.+\))?!?:", title.strip(), re.I):
        return "minor"
    if "version:patch" in labels:
        return "patch"
    return "patch"


def next_version(current: str, bump: str) -> str:
    match = SEMVER_RE.fullmatch(current.strip())
    if not match:
        raise RuntimeError(f"invalid current version: {current!r}")
    major, minor, patch = (int(part) for part in match.groups())
    if bump == "major":
        return f"v{major + 1}.0.0"
    if bump == "minor":
        return f"v{major}.{minor + 1}.0"
    if bump == "patch":
        return f"v{major}.{minor}.{patch + 1}"
    raise RuntimeError(f"invalid bump type: {bump!r}")


def read_status(path: Path) -> dict:
    if not path.exists():
        return {
            "repoVersion": "v0.0.0",
            "deployedControllerVersion": "",
            "lastBump": "patch",
            "lastDeployImpact": "repo-only",
            "sourceRevision": "",
            "updatedAt": "",
        }
    status = json.loads(path.read_text(encoding="utf-8"))
    return normalize_status(status, str(path))


def read_version(path: Path, fallback: str) -> str:
    if not path.exists():
        return fallback
    version = path.read_text(encoding="utf-8").strip()
    if not SEMVER_RE.fullmatch(version):
        raise RuntimeError(f"{path} has invalid version: {version!r}")
    return version


def write_version(path: Path, version: str) -> None:
    if not SEMVER_RE.fullmatch(version):
        raise RuntimeError(f"invalid version: {version!r}")
    path.write_text(f"{version}\n", encoding="utf-8")


def normalize_status(status: dict, source: str) -> dict:
    repo_version = status.get("repoVersion") or "v0.0.0"
    if not SEMVER_RE.fullmatch(repo_version):
        raise RuntimeError(f"{source} has invalid repoVersion: {repo_version!r}")
    return {
        "repoVersion": repo_version,
        "deployedControllerVersion": status.get("deployedControllerVersion") or "",
        "lastBump": status.get("lastBump") or "patch",
        "lastDeployImpact": status.get("lastDeployImpact") or "repo-only",
        "sourceRevision": status.get("sourceRevision") or "",
        "updatedAt": status.get("updatedAt") or "",
    }


def read_committed_status(args: argparse.Namespace) -> dict | None:
    if args.committed_status_file:
        if not args.committed_status_file.exists():
            return None
        return normalize_status(
            json.loads(args.committed_status_file.read_text(encoding="utf-8")),
            str(args.committed_status_file),
        )
    if not args.committed_status_ref:
        return None

    status_path = args.status_file.as_posix()
    result = subprocess.run(
        ["git", "show", f"{args.committed_status_ref}:{status_path}"],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return None
    return normalize_status(
        json.loads(result.stdout),
        f"{args.committed_status_ref}:{status_path}",
    )


def write_status(path: Path, status: dict) -> None:
    path.write_text(json.dumps(status, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def update_readme(path: Path, status: dict) -> None:
    content = path.read_text(encoding="utf-8")
    block = release_status_block(status)
    if README_START in content and README_END in content:
        pattern = re.compile(
            rf"{re.escape(README_START)}.*?{re.escape(README_END)}",
            flags=re.S,
        )
        updated = pattern.sub(block, content, count=1)
    else:
        marker = "![Renovate configured](https://img.shields.io/badge/Renovate-configured-1f8b4c)"
        if marker not in content:
            raise RuntimeError("could not find README badge insertion point")
        updated = content.replace(marker, f"{marker}\n\n{block}", 1)
    path.write_text(updated, encoding="utf-8")


def release_status_block(status: dict) -> str:
    repo_version = status["repoVersion"]
    deployed = status.get("deployedControllerVersion") or "none yet"
    divergence = (
        "in sync"
        if deployed != "none yet" and repo_version == deployed
        else "repo ahead of deployed controller"
        if deployed != "none yet"
        else "no controller image deployed yet"
    )
    return "\n".join(
        [
            README_START,
            "## Release Status",
            "",
            f"- Repo version: `{repo_version}`",
            f"- Deployed controller image: `{deployed}`",
            f"- Version state: {divergence}",
            f"- Last bump: `{status['lastBump']}`",
            f"- Last deploy impact: `{status['lastDeployImpact']}`",
            README_END,
        ]
    )


def assert_ghcr_tag_absent(image_repository: str, tag: str) -> None:
    if ghcr_tag_exists(image_repository, tag):
        raise RuntimeError(f"GHCR image tag already exists and must not be overwritten: {tag}")
    print(f"GHCR image tag is available: {tag}")


def ghcr_tag_exists(image_repository: str, tag: str) -> bool:
    token = require_env("GITHUB_TOKEN")
    package_info = parse_ghcr_image_repository(image_repository)
    versions = list_package_versions(token, package_info, allow_missing=True)
    for version in versions:
        tags = version.get("metadata", {}).get("container", {}).get("tags", [])
        if tag in tags:
            return True
    return False


def parse_ghcr_image_repository(repository: str) -> dict[str, str]:
    normalized = repository.removeprefix("https://").removeprefix("http://")
    parts = [part for part in normalized.split("/") if part]
    if len(parts) < 3 or parts[0] != "ghcr.io":
        raise RuntimeError(f"image repository must look like ghcr.io/owner/package, got {repository}")
    return {"owner": parts[1], "package_name": "/".join(parts[2:]), "owner_kind": ""}


def list_package_versions(
    token: str,
    package_info: dict[str, str],
    allow_missing: bool = False,
) -> list[dict]:
    for owner_kind in ("orgs", "users"):
        package_info["owner_kind"] = owner_kind
        status, headers, body = github_request(token, package_info)
        if status == 404:
            continue
        if status < 200 or status >= 300:
            raise RuntimeError(f"failed to list GHCR versions: {status} {body}")
        return paginate(token, package_info, json.loads(body), headers.get("link"))
    if allow_missing:
        return []
    raise RuntimeError(
        f"could not find GHCR package {package_info['owner']}/{package_info['package_name']}"
    )


def paginate(token: str, package_info: dict[str, str], first_page: list[dict], first_link_header: str | None) -> list[dict]:
    pages = list(first_page)
    next_url = get_next_url(first_link_header)
    while next_url:
        status, headers, body = github_request(token, package_info, next_url)
        if status < 200 or status >= 300:
            raise RuntimeError(f"failed to paginate GHCR versions: {status} {body}")
        pages.extend(json.loads(body))
        next_url = get_next_url(headers.get("link"))
    return pages


def github_request(
    token: str,
    package_info: dict[str, str],
    suffix_or_url: str | None = None,
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


def parse_bool(value: str) -> bool:
    return value.strip().lower() == "true"


def now_utc() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def require_env(name: str) -> str:
    value = os.getenv(name)
    if not value:
        raise RuntimeError(f"{name} is required")
    return value


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
