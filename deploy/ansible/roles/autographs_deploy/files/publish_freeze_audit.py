#!/usr/bin/env python3
"""Fail-closed shared-media audit for the one-time publisher cutover.

Only the current symlink proves publication without an independent database or
archived promotion record. Retained manifests alone are deliberately not proof.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import sys
import time


MEDIA_PATH = re.compile(
    r"^media/[a-z0-9-]+/[a-z0-9-]+-(?:thumbnail|detail)-([0-9a-f]{16})\.webp$"
)
MIN_AGE_SECONDS = 86_400


def release_media(manifest_path: Path) -> set[str]:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    paths = set()
    for artifact in manifest["artifacts"]:
        if artifact.get("variant") is None:
            continue
        path = artifact["path"]
        if MEDIA_PATH.fullmatch(path) is None:
            raise ValueError(f"invalid manifest media path: {path}")
        paths.add(path)
    return paths


def verified_media(path: Path, relative: str) -> bytes:
    match = MEDIA_PATH.fullmatch(relative)
    if match is None or path.is_symlink() or not path.is_file():
        raise ValueError(f"invalid shared media path: {relative}")
    content = path.read_bytes()
    if hashlib.sha256(content).hexdigest()[:16] != match.group(1):
        raise ValueError(f"shared media fingerprint mismatch: {relative}")
    if not (content.startswith(b"RIFF") and content[8:12] == b"WEBP"):
        raise ValueError(f"shared media is not WebP: {relative}")
    return content


def protected_path(path: Path, static_root: Path) -> Path:
    resolved = path.resolve()
    if resolved == static_root or static_root in resolved.parents:
        raise ValueError("freeze record and quarantine must be outside the served static root")
    return resolved


def audit(args: argparse.Namespace) -> int:
    static_root = Path(args.static_root).resolve(strict=True)
    quarantine = protected_path(Path(args.quarantine), static_root)
    record = protected_path(Path(args.record), static_root)
    quarantine.mkdir(parents=True, exist_ok=True, mode=0o700)
    os.chmod(quarantine, 0o700)
    record.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    os.chmod(record.parent, 0o700)
    record.unlink(missing_ok=True)

    current_link = static_root / "current"
    if not current_link.is_symlink():
        raise ValueError("current release pointer is absent or not a symlink")
    current = current_link.resolve(strict=True)
    releases_root = (static_root / "releases").resolve(strict=True)
    if current.parent != releases_root:
        raise ValueError("current pointer escapes releases directory")
    active = release_media(current / "manifest.json")
    for relative in active:
        verified_media(current / relative, relative)
    retained = set()
    for release in releases_root.iterdir():
        if release.is_symlink():
            raise ValueError("retained release is a symlink")
        if release.is_dir() and release != current and (release / "manifest.json").is_file():
            retained.update(release_media(release / "manifest.json"))

    shared_root = static_root / "media"
    if shared_root.is_symlink():
        raise ValueError("shared media root is a symlink")
    shared_root.mkdir(exist_ok=True)
    kept = []
    quarantined = []
    unresolved = []
    for shared in sorted(shared_root.rglob("*")):
        if shared.is_dir():
            if shared.is_symlink():
                raise ValueError("shared media contains a directory symlink")
            continue
        relative = shared.relative_to(static_root).as_posix()
        age = time.time() - shared.stat(follow_symlinks=False).st_mtime
        try:
            content = verified_media(shared, relative)
            if relative in active:
                if content != (current / relative).read_bytes():
                    raise ValueError(f"active shared media differs from current: {relative}")
                kept.append(relative)
                continue
        except (ValueError, OSError) as error:
            reason = str(error)
        else:
            reason = "no committed publication evidence"
        destination = quarantine / relative
        destination.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
        os.chmod(destination.parent, 0o700)
        if destination.exists():
            raise ValueError(f"quarantine collision: {relative}")
        shutil.move(str(shared), str(destination))
        quarantined.append({"path": relative, "reason": reason})
        if relative in retained or age < MIN_AGE_SECONDS:
            unresolved.append(relative)

    audit_path = quarantine / f"audit-result-{time.time_ns()}.json"
    audit_path.write_text(
        json.dumps({"kept": kept, "quarantined": quarantined, "unresolved": unresolved}, indent=2),
        encoding="utf-8",
    )
    os.chmod(audit_path, 0o600)
    if unresolved:
        print("Cutover blocked: shared media lacks committed evidence for " + ", ".join(unresolved), file=sys.stderr)
        return 2
    payload = {
        "release_tag": args.release_tag,
        "controller_digest": args.controller_digest,
        "current_target": str(current.relative_to(static_root)),
        "old_container_id": args.old_container_id,
        "old_pid": args.old_pid,
        "audited_shared_media": kept,
        "bootstrap": False,
    }
    staging = record.with_suffix(".next")
    staging.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    os.chmod(staging, 0o600)
    os.replace(staging, record)
    print(f"Audited {len(kept)} committed shared URLs; quarantined {len(quarantined)} unproven URLs")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--static-root", required=True)
    parser.add_argument("--quarantine", required=True)
    parser.add_argument("--record", required=True)
    parser.add_argument("--release-tag", required=True)
    parser.add_argument("--controller-digest", required=True)
    parser.add_argument("--old-container-id", required=True)
    parser.add_argument("--old-pid", required=True, type=int)
    return audit(parser.parse_args())


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        print(f"Cutover audit failed closed: {error}", file=sys.stderr)
        sys.exit(1)
