#!/usr/bin/env python3

"""Validate and apply a release-specific retry recovery contract."""

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path


SHA_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
SEMVER_TAG_RE = re.compile(r"^v[0-9]+\.[0-9]+\.[0-9]+$")
DEPLOY_PREFIX = "deploy/ansible/"


class RecoveryError(RuntimeError):
    """A retry recovery request violates its immutable contract."""


def _git(repo: Path, *args: str, binary: bool = False) -> str | bytes:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        check=False,
        capture_output=True,
        text=not binary,
    )
    if result.returncode != 0:
        stderr = result.stderr.decode() if binary else result.stderr
        stdout = result.stdout.decode() if binary else result.stdout
        detail = stderr.strip() or stdout.strip()
        raise RecoveryError(f"git {' '.join(args)} failed: {detail}")
    return result.stdout


def _commit(repo: Path, revision: str) -> str:
    value = str(_git(repo, "rev-parse", "--verify", f"{revision}^{{commit}}")).strip()
    if not SHA_RE.fullmatch(value):
        raise RecoveryError(f"revision did not resolve to a full Git SHA: {revision!r}")
    return value


def _load_contract(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise RecoveryError(f"could not read recovery contract {path}: {error}") from error
    if not isinstance(value, dict) or value.get("schemaVersion") != 1:
        raise RecoveryError("retry recovery contract must use schemaVersion 1")
    releases = value.get("releases")
    if not isinstance(releases, dict):
        raise RecoveryError("retry recovery contract releases must be an object")
    return releases


def _approved_files(entry: dict[str, object]) -> dict[str, str]:
    value = entry.get("approvedFiles")
    if not isinstance(value, dict) or not value:
        raise RecoveryError("recovery contract approvedFiles must be a non-empty object")
    approved: dict[str, str] = {}
    for path, digest in value.items():
        if (
            not isinstance(path, str)
            or not path.startswith(DEPLOY_PREFIX)
            or Path(path).is_absolute()
            or ".." in Path(path).parts
        ):
            raise RecoveryError(f"approved recovery path is unsafe: {path!r}")
        if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
            raise RecoveryError(f"approved recovery digest is invalid for {path}")
        approved[path] = digest
    return approved


def apply_recovery(
    repo: Path,
    contract_path: Path,
    release_tag: str,
    source_revision: str,
    automation_revision: str,
) -> dict[str, object]:
    """Validate every changed Ansible path before overlaying approved bytes."""
    if not SEMVER_TAG_RE.fullmatch(release_tag):
        raise RecoveryError(f"release tag must match vX.Y.Z, got {release_tag!r}")
    source_commit = _commit(repo, source_revision)
    tag_commit = _commit(repo, f"refs/tags/{release_tag}")
    if source_commit != tag_commit:
        raise RecoveryError(
            f"release tag {release_tag} resolves to {tag_commit}, not {source_commit}"
        )
    if not SHA_RE.fullmatch(automation_revision):
        raise RecoveryError("automation revision must be a full lowercase Git SHA")

    releases = _load_contract(contract_path)
    entry = releases.get(release_tag)
    if entry is None:
        return {
            "schemaVersion": 1,
            "releaseTag": release_tag,
            "releaseSourceRevision": source_commit,
            "automationRevision": automation_revision,
            "recoveryApplied": False,
            "recoveryRevision": source_commit,
            "approvedFiles": [],
        }
    if not isinstance(entry, dict):
        raise RecoveryError(f"recovery contract for {release_tag} must be an object")

    declared_revision = entry.get("recoveryRevision")
    if not isinstance(declared_revision, str) or not SHA_RE.fullmatch(declared_revision):
        raise RecoveryError(f"recoveryRevision for {release_tag} must be a full Git SHA")
    recovery_commit = _commit(repo, declared_revision)
    if recovery_commit != declared_revision:
        raise RecoveryError("recoveryRevision must be pinned to the resolved commit SHA")
    approved = _approved_files(entry)

    changed_output = str(
        _git(
            repo,
            "diff",
            "--no-renames",
            "--name-only",
            "--diff-filter=ACDMRTUXB",
            f"{source_commit}..{recovery_commit}",
            "--",
            "deploy/ansible",
        )
    )
    changed = sorted(path for path in changed_output.splitlines() if path)
    expected = sorted(approved)
    if changed != expected:
        unexpected = sorted(set(changed) - set(expected))
        missing = sorted(set(expected) - set(changed))
        raise RecoveryError(
            "recovery Ansible delta does not match the release allowlist; "
            f"unexpected={unexpected}, missing={missing}"
        )

    approved_content: list[tuple[str, bytes, str]] = []
    for path in expected:
        source_bytes = _git(repo, "show", f"{source_commit}:{path}", binary=True)
        current_path = repo / path
        try:
            current_bytes = current_path.read_bytes()
        except OSError as error:
            raise RecoveryError(f"could not read selected-tag file {path}: {error}") from error
        if current_bytes != source_bytes:
            raise RecoveryError(f"selected-tag workspace drifted before recovery overlay: {path}")
        recovery_bytes = _git(repo, "show", f"{recovery_commit}:{path}", binary=True)
        digest = f"sha256:{hashlib.sha256(recovery_bytes).hexdigest()}"
        if digest != approved[path]:
            raise RecoveryError(
                f"recovery content hash mismatch for {path}: expected {approved[path]}, got {digest}"
            )
        approved_content.append((path, recovery_bytes, digest))

    for path, content, _ in approved_content:
        target = repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(dir=target.parent, delete=False) as temporary:
            temporary.write(content)
            temporary_path = Path(temporary.name)
        temporary_path.replace(target)

    return {
        "schemaVersion": 1,
        "releaseTag": release_tag,
        "releaseSourceRevision": source_commit,
        "automationRevision": automation_revision,
        "recoveryApplied": True,
        "recoveryRevision": recovery_commit,
        "approvedFiles": [
            {"path": path, "sha256": digest}
            for path, _, digest in approved_content
        ],
    }


def _write_json_atomic(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", dir=path.parent, delete=False
    ) as temporary:
        json.dump(value, temporary, indent=2, sort_keys=True)
        temporary.write("\n")
        temporary_path = Path(temporary.name)
    temporary_path.replace(path)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--contract", type=Path, required=True)
    parser.add_argument("--release-tag", required=True)
    parser.add_argument("--source-revision", required=True)
    parser.add_argument("--automation-revision", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    audit = apply_recovery(
        args.repo,
        args.contract,
        args.release_tag,
        args.source_revision,
        args.automation_revision,
    )
    _write_json_atomic(args.output, audit)
    print(json.dumps(audit, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RecoveryError as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
