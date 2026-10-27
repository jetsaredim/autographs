#!/usr/bin/env python3
"""Journaled static pointer rollback, run only under the frozen stopped-controller gate."""
import argparse
import json
import os
from pathlib import Path
import sys
import time

from publish_freeze_audit import atomic_json, current, manifest, marked, receipt, verified


def rollback(root: Path, target: str, frozen_exit_proved: bool) -> None:
    if not frozen_exit_proved:
        raise ValueError("rollback requires verified publish freeze and controller exit")
    current_id = current(root)
    directory = root / ".media-commit-history"
    if (directory / "intent.json").exists():
        raise ValueError("interrupted transition requires frozen audit reconciliation first")
    history = json.loads((directory / "history.json").read_text())
    if history.get("version") != 1 or history.get("current_release_id") != current_id:
        raise ValueError("unrecorded current pointer or invalid history")
    if target == current_id or target not in history["releases"]:
        raise ValueError("rollback target must be a distinct committed release")
    if not (root / "releases" / target).is_dir():
        raise ValueError("committed rollback target was pruned")
    for release_id in (current_id, target):
        recorded = history["releases"][release_id]
        fresh = receipt(root, release_id)
        if fresh["manifest_sha256"] != recorded["manifest_sha256"]:
            raise ValueError("committed rollback manifest changed")
        manifest(root, release_id)
        for name, full_hash in recorded["media"].items():
            if marked(root, name):
                continue
            if fresh["media"].get(name) != full_hash:
                raise ValueError(f"committed rollback media changed: {name}")
            shared = root / name
            if shared.exists():
                verified(shared, name, expected=full_hash)
            else:
                shared.parent.mkdir(parents=True, exist_ok=True)
                source = root / "releases" / release_id / name
                staging = shared.with_name("." + shared.name + ".rollback")
                staging.write_bytes(source.read_bytes())
                with staging.open("rb") as handle:
                    os.fsync(handle.fileno())
                os.replace(staging, shared)
    new_manifest = history["releases"][target]["manifest_sha256"]
    intent = {"version": 1, "old_release_id": current_id, "new_release_id": target,
              "old_manifest_sha256": history["releases"][current_id]["manifest_sha256"],
              "new_manifest_sha256": new_manifest,
              "new_media": history["releases"][target]["media"]}
    atomic_json(directory / "intent.json", intent)
    next_pointer = root / ".current-next"
    if next_pointer.exists() or next_pointer.is_symlink():
        raise ValueError("stale pointer staging path")
    os.symlink("releases/" + target, next_pointer)
    os.replace(next_pointer, root / "current")
    descriptor = os.open(root, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    now = int(time.time())
    history["releases"][current_id]["last_active_end"] = now
    history["releases"][target]["last_active_end"] = None
    history["current_release_id"] = target
    history["transitions"].append({"from": current_id, "to": target, "at": now})
    atomic_json(directory / "history.json", history)
    (directory / "intent.json").unlink()
    descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--static-root", required=True)
    parser.add_argument("--target-release-id", required=True)
    parser.add_argument("--frozen-exit-proved", action="store_true")
    args = parser.parse_args()
    if not args.target_release_id.isascii() or not all(c.isalnum() or c == '-' for c in args.target_release_id):
        raise ValueError("invalid rollback target ID")
    rollback(Path(args.static_root).resolve(strict=True), args.target_release_id, args.frozen_exit_proved)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        print(f"Static rollback refused: {error}", file=sys.stderr)
        sys.exit(1)
