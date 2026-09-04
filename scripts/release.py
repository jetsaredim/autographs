#!/usr/bin/env python3

"""Pure release planning and production-state reconciliation helpers.

Version selection, Git tag creation, and GitHub Release mutation belong to
release-please and the calling workflow. This module only validates and
reconciles the inputs presented to it.
"""

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path


SEMVER_TAG_RE = re.compile(r"^v([0-9]+)\.([0-9]+)\.([0-9]+)$")
DIGEST_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
IMPACTS = {"controller-image", "runtime-config", "repo-only"}


class ReleaseError(RuntimeError):
    """A release input is ambiguous or violates the release contract."""


def _git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        raise ReleaseError(f"git {' '.join(args)} failed: {detail}")
    return result.stdout.strip()


def _semver_key(tag: str) -> tuple[int, int, int]:
    match = SEMVER_TAG_RE.fullmatch(tag)
    if not match:
        raise ReleaseError(f"release tag must be semantic vX.Y.Z, got {tag!r}")
    return tuple(int(part) for part in match.groups())


def validate_target_tag(repo: Path, tag: str, source_revision: str) -> str:
    """Require a semantic target tag to resolve to the declared source commit."""
    _semver_key(tag)
    tag_revision = _git(repo, "rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}")
    source_commit = _git(repo, "rev-parse", "--verify", f"{source_revision}^{{commit}}")
    if tag_revision != source_commit:
        raise ReleaseError(
            f"target tag {tag} does not resolve to declared source {source_commit}; "
            f"it resolves to {tag_revision}"
        )
    return tag_revision


def find_previous_release(repo: Path, target_tag: str) -> str | None:
    """Return the closest reachable semantic release before target_tag."""
    _semver_key(target_tag)
    candidates: list[tuple[int, tuple[int, int, int], str]] = []
    for tag in _git(repo, "tag", "--merged", target_tag, "--list", "v*").splitlines():
        if tag == target_tag or not SEMVER_TAG_RE.fullmatch(tag):
            continue
        distance = int(_git(repo, "rev-list", "--count", f"{tag}..{target_tag}"))
        candidates.append((distance, tuple(-part for part in _semver_key(tag)), tag))
    return min(candidates)[2] if candidates else None


def _changed_paths(repo: Path, previous_tag: str | None, target_tag: str) -> list[str]:
    if previous_tag:
        output = _git(
            repo,
            "diff",
            "--name-only",
            "--diff-filter=ACDMRTUXB",
            f"{previous_tag}..{target_tag}",
        )
    else:
        output = _git(repo, "ls-tree", "-r", "--name-only", target_tag)
    return sorted({path for path in output.splitlines() if path})


def _is_controller_path(path: str) -> bool:
    return path.startswith("controller/") or path == ".github/docker-bake.hcl"


def _is_terraform_path(path: str) -> bool:
    return path.startswith("infra/terraform/")


def _is_deploy_path(path: str) -> bool:
    return (
        path.startswith("deploy/ansible/")
        or path.startswith(".github/actions/resolve-runtime-ip/")
        or path in {
            ".github/workflows/deploy.yml",
            ".github/workflows/release.yml",
            "scripts/release.py",
        }
    )


def classify_release_range(
    repo: Path, previous_tag: str | None, target_tag: str
) -> dict[str, object]:
    """Classify every path in the full previous-to-target release range."""
    _semver_key(target_tag)
    if previous_tag:
        _semver_key(previous_tag)
    paths = _changed_paths(repo, previous_tag, target_tag)
    controller_changed = any(_is_controller_path(path) for path in paths)
    terraform_changed = any(_is_terraform_path(path) for path in paths)
    deploy_changed = any(_is_deploy_path(path) for path in paths)
    classified = {
        path
        for path in paths
        if _is_controller_path(path) or _is_terraform_path(path) or _is_deploy_path(path)
    }
    if controller_changed:
        impact = "controller-image"
    elif terraform_changed or deploy_changed:
        impact = "runtime-config"
    else:
        impact = "repo-only"
    return {
        "impact": impact,
        "controllerChanged": controller_changed,
        "terraformChanged": terraform_changed,
        "ansibleOrDeployChanged": deploy_changed,
        "repositoryOnlyChanged": bool(set(paths) - classified),
        "productionMutationRequired": impact != "repo-only",
        "changedPaths": paths,
    }


def collect_release_trailers(
    repo: Path, previous_tag: str | None, target_tag: str
) -> dict[str, list[str]]:
    """Collect the two explicitly supported operator-authored commit trailers."""
    revision_range = f"{previous_tag}..{target_tag}" if previous_tag else target_tag
    messages = _git(repo, "log", "--reverse", "--format=%B%x00", revision_range)
    migration_notes: list[str] = []
    operator_warnings: list[str] = []
    for message in messages.split("\x00"):
        for line in message.splitlines():
            name, separator, value = line.partition(":")
            if not separator or not value.strip():
                continue
            if name == "Autographs-Migration-Note":
                migration_notes.append(value.strip())
            elif name == "Autographs-Release-Warning":
                operator_warnings.append(value.strip())
    return {
        "migrationNotes": list(dict.fromkeys(migration_notes)),
        "operatorWarnings": list(dict.fromkeys(operator_warnings)),
    }


def draft_preflight(releases: object) -> dict[str, object]:
    """Return every unresolved semantic draft, newest first."""
    if not isinstance(releases, list):
        raise ReleaseError("GitHub Releases JSON must be an array")
    tags = {
        item.get("tag_name")
        for item in releases
        if isinstance(item, dict)
        and item.get("draft") is True
        and isinstance(item.get("tag_name"), str)
        and SEMVER_TAG_RE.fullmatch(item["tag_name"])
    }
    ordered = sorted(tags, key=_semver_key, reverse=True)
    return {"blocked": bool(ordered), "tags": ordered}


def validate_digest(digest: str) -> str:
    if not DIGEST_RE.fullmatch(digest):
        raise ReleaseError(f"digest must use sha256:<64 lowercase hex>, got {digest!r}")
    return digest


def assert_digest_matches(expected: str, actual: str) -> None:
    validate_digest(expected)
    validate_digest(actual)
    if expected != actual:
        raise ReleaseError(f"resolved digest {actual} does not match expected digest {expected}")


def plan_release(
    repo: Path,
    target_tag: str,
    source_revision: str,
    status: dict[str, object],
    resolved_controller_digest: str,
) -> dict[str, object]:
    """Resolve the immutable inputs needed to produce a release manifest."""
    source_commit = validate_target_tag(repo, target_tag, source_revision)
    previous = find_previous_release(repo, target_tag)
    classification = classify_release_range(repo, previous, target_tag)
    trailers = collect_release_trailers(repo, previous, target_tag)
    validate_digest(resolved_controller_digest)

    controller_changed = bool(classification["controllerChanged"])
    if controller_changed:
        controller_tag = target_tag
        controller_reused = False
    else:
        controller_tag = str(status.get("deployedControllerVersion") or "")
        if not SEMVER_TAG_RE.fullmatch(controller_tag):
            raise ReleaseError("reused controller status must contain a semantic deployedControllerVersion")
        expected_digest = str(status.get("deployedControllerDigest") or "")
        if expected_digest:
            assert_digest_matches(expected_digest, resolved_controller_digest)
        controller_reused = True

    return {
        "repositoryVersion": target_tag,
        "previousRelease": previous,
        "sourceRevision": source_commit,
        **classification,
        "controllerTag": controller_tag,
        "controllerDigest": resolved_controller_digest,
        "controllerReused": controller_reused,
        **trailers,
    }


def public_schema_version(contracts_file: Path) -> int:
    content = contracts_file.read_text(encoding="utf-8")
    match = re.search(r"pub\s+const\s+PUBLIC_SCHEMA_VERSION\s*:\s*u32\s*=\s*([0-9]+)\s*;", content)
    if not match:
        raise ReleaseError(f"could not find PUBLIC_SCHEMA_VERSION in {contracts_file}")
    return int(match.group(1))


def build_release_manifest(
    repo: Path,
    plan: dict[str, object],
    contracts_file: Path | None = None,
) -> bytes:
    digest = validate_digest(str(plan["controllerDigest"]))
    controller_tag = str(plan["controllerTag"])
    _semver_key(controller_tag)
    impact = str(plan["impact"])
    if impact not in IMPACTS:
        raise ReleaseError(f"unknown release impact {impact!r}")
    contract_path = contracts_file or repo / "controller/src/contracts.rs"
    output = {
        "schemaVersion": 1,
        "repositoryVersion": str(plan["repositoryVersion"]),
        "previousRelease": plan.get("previousRelease"),
        "sourceRevision": str(plan["sourceRevision"]),
        "impact": impact,
        "controller": {
            "tag": controller_tag,
            "digest": digest,
            "reused": bool(plan["controllerReused"]),
        },
        "controllerChanged": bool(plan["controllerChanged"]),
        "terraformChanged": bool(plan["terraformChanged"]),
        "ansibleOrDeployChanged": bool(plan["ansibleOrDeployChanged"]),
        "publicSchemaVersion": public_schema_version(contract_path),
        "migrationNotes": list(plan.get("migrationNotes") or []),
        "operatorWarnings": list(plan.get("operatorWarnings") or []),
    }
    return (json.dumps(output, indent=2, sort_keys=True) + "\n").encode("utf-8")


def reconcile_manifest_asset(existing: bytes | None, generated: bytes) -> str:
    if existing is None:
        return "create"
    if existing == generated:
        return "same"
    existing_hash = hashlib.sha256(existing).hexdigest()
    generated_hash = hashlib.sha256(generated).hexdigest()
    raise ReleaseError(
        "release-manifest.json conflict: existing sha256 "
        f"{existing_hash} differs from generated sha256 {generated_hash}"
    )


def _validate_manifest(manifest: dict[str, object]) -> None:
    _semver_key(str(manifest.get("repositoryVersion") or ""))
    impact = manifest.get("impact")
    if impact not in IMPACTS:
        raise ReleaseError(f"unknown release impact {impact!r}")
    controller = manifest.get("controller")
    if not isinstance(controller, dict):
        raise ReleaseError("release manifest controller must be an object")
    _semver_key(str(controller.get("tag") or ""))
    validate_digest(str(controller.get("digest") or ""))
    source = str(manifest.get("sourceRevision") or "")
    if not re.fullmatch(r"[0-9a-f]{40}", source):
        raise ReleaseError("release manifest sourceRevision must be a full lowercase Git SHA")


def apply_deployment_status(
    status: dict[str, object],
    release_manifest: dict[str, object],
    mode: str,
    updated_at: str,
) -> dict[str, object]:
    """Apply an automatic/retry success without inventing release state."""
    if mode not in {"automatic", "retry"}:
        raise ReleaseError(f"status mode must be automatic or retry, got {mode!r}")
    _validate_manifest(release_manifest)
    result = dict(status)
    version = str(release_manifest["repositoryVersion"])
    impact = str(release_manifest["impact"])
    controller = release_manifest["controller"]
    assert isinstance(controller, dict)
    controller_tag = str(controller["tag"])
    controller_digest = str(controller["digest"])

    result["latestRepositoryVersion"] = version
    result["lastDeployImpact"] = impact
    if impact != "repo-only":
        result["deployedRepositoryVersion"] = version
        result["latestDeployImpactVersion"] = version
        result["sourceRevision"] = str(release_manifest["sourceRevision"])

    active_tag = str(result.get("deployedControllerVersion") or "")
    active_digest = str(result.get("deployedControllerDigest") or "")
    if active_tag != controller_tag:
        result["previousControllerVersion"] = active_tag
        result["previousControllerDigest"] = active_digest
    result["deployedControllerVersion"] = controller_tag
    result["deployedControllerDigest"] = controller_digest

    unchanged = {
        key: value for key, value in result.items() if key != "updatedAt"
    } == {key: value for key, value in status.items() if key != "updatedAt"}
    if unchanged:
        return dict(status)
    result["updatedAt"] = updated_at
    return result


def apply_controller_rollback(
    status: dict[str, object],
    controller_tag: str,
    controller_digest: str,
    updated_at: str,
) -> dict[str, object]:
    """Change only active/previous controller mapping for a controller rollback."""
    _semver_key(controller_tag)
    validate_digest(controller_digest)
    active_tag = str(status.get("deployedControllerVersion") or "")
    active_digest = str(status.get("deployedControllerDigest") or "")
    if active_tag == controller_tag and active_digest == controller_digest:
        return dict(status)
    result = dict(status)
    result["deployedControllerVersion"] = controller_tag
    result["deployedControllerDigest"] = controller_digest
    result["previousControllerVersion"] = active_tag
    result["previousControllerDigest"] = active_digest
    result["updatedAt"] = updated_at
    return result


def _read_json(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ReleaseError(f"could not read JSON from {path}: {error}") from error


def _write_json_atomic(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", dir=path.parent, delete=False
    ) as temporary:
        json.dump(value, temporary, indent=2, sort_keys=True)
        temporary.write("\n")
        temporary_path = Path(temporary.name)
    temporary_path.replace(path)


def _write_bytes_atomic(path: Path, value: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode="wb", dir=path.parent, delete=False) as temporary:
        temporary.write(value)
        temporary_path = Path(temporary.name)
    temporary_path.replace(path)


def _json_object(path: Path) -> dict[str, object]:
    value = _read_json(path)
    if not isinstance(value, dict):
        raise ReleaseError(f"{path} must contain a JSON object")
    return value


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)

    validate = subcommands.add_parser("validate-target")
    validate.add_argument("--repo", type=Path, default=Path("."))
    validate.add_argument("--tag", required=True)
    validate.add_argument("--source-revision", required=True)

    previous = subcommands.add_parser("previous-release")
    previous.add_argument("--repo", type=Path, default=Path("."))
    previous.add_argument("--tag", required=True)

    classify = subcommands.add_parser("classify-range")
    classify.add_argument("--repo", type=Path, default=Path("."))
    classify.add_argument("--previous-tag")
    classify.add_argument("--tag", required=True)

    trailers = subcommands.add_parser("trailers")
    trailers.add_argument("--repo", type=Path, default=Path("."))
    trailers.add_argument("--previous-tag")
    trailers.add_argument("--tag", required=True)

    preflight = subcommands.add_parser("preflight-drafts")
    preflight.add_argument("--releases-json", type=Path, required=True)

    plan = subcommands.add_parser("plan")
    plan.add_argument("--repo", type=Path, default=Path("."))
    plan.add_argument("--tag", required=True)
    plan.add_argument("--source-revision", required=True)
    plan.add_argument("--status-file", type=Path, default=Path(".release-status.json"))
    plan.add_argument("--controller-digest", required=True)
    plan.add_argument("--output", type=Path)

    manifest_parser = subcommands.add_parser("manifest")
    manifest_parser.add_argument("--repo", type=Path, default=Path("."))
    manifest_parser.add_argument("--plan-json", type=Path, required=True)
    manifest_parser.add_argument("--contracts-file", type=Path)
    manifest_parser.add_argument("--output", type=Path, required=True)

    reconcile = subcommands.add_parser("reconcile-asset")
    reconcile.add_argument("--generated", type=Path, required=True)
    reconcile.add_argument("--existing", type=Path)

    digest = subcommands.add_parser("assert-digest")
    digest.add_argument("--expected", required=True)
    digest.add_argument("--actual", required=True)

    update = subcommands.add_parser("update-status")
    update.add_argument("--status-file", type=Path, default=Path(".release-status.json"))
    update.add_argument("--manifest", type=Path, required=True)
    update.add_argument("--mode", choices=("automatic", "retry"), required=True)
    update.add_argument("--updated-at", required=True)

    rollback = subcommands.add_parser("rollback-status")
    rollback.add_argument("--status-file", type=Path, default=Path(".release-status.json"))
    rollback.add_argument("--controller-tag", required=True)
    rollback.add_argument("--controller-digest", required=True)
    rollback.add_argument("--updated-at", required=True)

    args = parser.parse_args()
    if args.command == "validate-target":
        print(validate_target_tag(args.repo, args.tag, args.source_revision))
    elif args.command == "previous-release":
        print(find_previous_release(args.repo, args.tag) or "")
    elif args.command == "classify-range":
        print(json.dumps(classify_release_range(args.repo, args.previous_tag, args.tag), sort_keys=True))
    elif args.command == "trailers":
        print(json.dumps(collect_release_trailers(args.repo, args.previous_tag, args.tag), sort_keys=True))
    elif args.command == "preflight-drafts":
        result = draft_preflight(_read_json(args.releases_json))
        print(json.dumps(result, sort_keys=True))
        return 2 if result["blocked"] else 0
    elif args.command == "plan":
        value = plan_release(
            args.repo,
            args.tag,
            args.source_revision,
            _json_object(args.status_file),
            args.controller_digest,
        )
        encoded = json.dumps(value, indent=2, sort_keys=True) + "\n"
        if args.output:
            _write_bytes_atomic(args.output, encoded.encode("utf-8"))
        else:
            print(encoded, end="")
    elif args.command == "manifest":
        value = build_release_manifest(
            args.repo, _json_object(args.plan_json), args.contracts_file
        )
        _write_bytes_atomic(args.output, value)
    elif args.command == "reconcile-asset":
        existing = args.existing.read_bytes() if args.existing and args.existing.exists() else None
        print(reconcile_manifest_asset(existing, args.generated.read_bytes()))
    elif args.command == "assert-digest":
        assert_digest_matches(args.expected, args.actual)
    elif args.command == "update-status":
        value = apply_deployment_status(
            _json_object(args.status_file),
            _json_object(args.manifest),
            args.mode,
            args.updated_at,
        )
        _write_json_atomic(args.status_file, value)
    elif args.command == "rollback-status":
        value = apply_controller_rollback(
            _json_object(args.status_file),
            args.controller_tag,
            args.controller_digest,
            args.updated_at,
        )
        _write_json_atomic(args.status_file, value)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ReleaseError as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
