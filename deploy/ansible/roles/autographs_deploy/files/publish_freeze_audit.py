#!/usr/bin/env python3
"""Frozen audit of committed media; never infer publication from mtime."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import sys
import time

MEDIA = re.compile(r"^media/[a-z0-9-]+/[a-z0-9-]+-(?:thumbnail|detail)-([0-9a-f]{16})\.webp$")
HEX = re.compile(r"^[0-9a-f]{64}$")
WINDOW = 86_400
WAIT = WINDOW + 60 + 300


def digest(data):
    return hashlib.sha256(data).hexdigest()


def outside(path, root):
    path = Path(path).resolve()
    if path == root or root in path.parents:
        raise ValueError("audit output and evidence must be outside static root")
    return path


def atomic_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    next_path = path.with_name("." + path.name + ".next")
    with next_path.open("w", encoding="utf-8") as handle:
        os.chmod(next_path, 0o600)
        json.dump(value, handle, indent=2, sort_keys=True)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(next_path, path)
    descriptor = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def current(root):
    pointer = root / "current"
    if not pointer.is_symlink():
        raise ValueError("current pointer missing or not a symlink")
    target = os.readlink(pointer)
    if not re.fullmatch(r"releases/[A-Za-z0-9-]+", target) or not (root / target).is_dir():
        raise ValueError("current pointer is not a direct release")
    return target.split("/")[1]


def marked(root, relative):
    path = root / ".media-tombstones" / relative
    if path.is_symlink():
        raise ValueError(f"linked emergency marker: {relative}")
    if not path.exists():
        return False
    if not path.is_file():
        raise ValueError(f"malformed emergency marker: {relative}")
    return True


def marker_digest(root):
    directory = root / ".media-tombstones"
    names = []
    if directory.exists():
        for path in sorted(directory.rglob("*")):
            if path.is_symlink():
                raise ValueError("linked emergency marker")
            if path.is_file():
                name = path.relative_to(directory).as_posix()
                if not MEDIA.fullmatch(name):
                    raise ValueError(f"invalid emergency marker path: {name}")
                names.append(name)
            elif not path.is_dir():
                raise ValueError("nonregular emergency marker")
    return digest("\n".join(names).encode())


def verified(path, relative, size=None, expected=None):
    match = MEDIA.fullmatch(relative)
    if not match or path.is_symlink() or not path.is_file():
        raise ValueError(f"invalid shared media path: {relative}")
    data = path.read_bytes()
    actual = digest(data)
    if actual[:16] != match.group(1) or not (data.startswith(b"RIFF") and data[8:12] == b"WEBP"):
        raise ValueError(f"media fingerprint or WebP mismatch: {relative}")
    if size is not None and len(data) != size:
        raise ValueError(f"media size mismatch: {relative}")
    if expected is not None and actual != expected:
        raise ValueError(f"committed media digest mismatch: {relative}")
    return actual


def manifest(root, release_id, archive=None):
    path = archive or root / "releases" / release_id / "manifest.json"
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"manifest missing or linked: {release_id}")
    raw = path.read_bytes()
    value = json.loads(raw)
    if value.get("releaseId", value.get("release_id")) != release_id:
        raise ValueError(f"manifest release ID mismatch: {release_id}")
    return value, digest(raw)


def receipt(root, release_id, archive=None, historical=None):
    value, manifest_hash = manifest(root, release_id, archive)
    media = {}
    for artifact in value["artifacts"]:
        if artifact.get("variant") is None:
            continue
        name = artifact["path"]
        if not MEDIA.fullmatch(name) or artifact.get("contentType", artifact.get("content_type")) != "image/webp":
            raise ValueError(f"invalid manifest media path: {name}")
        if marked(root, name):
            continue
        path = root / name if archive else root / "releases" / release_id / name
        media[name] = verified(path, name, artifact.get("byteSize", artifact.get("byte_size")),
                               (historical or {}).get(name))
    return {"manifest_sha256": manifest_hash, "media": media,
            "first_committed_at": int(time.time()), "last_active_end": None}


def shared_files(root):
    media = root / "media"
    if media.is_symlink():
        raise ValueError("linked shared media root")
    found = {}
    if media.exists():
        for path in sorted(media.rglob("*")):
            if path.is_symlink():
                raise ValueError("linked shared media")
            if path.is_file():
                found[path.relative_to(root).as_posix()] = path
            elif not path.is_dir():
                raise ValueError("nonregular shared media")
    return found


def validate_history(root, history, pointer):
    if history.get("version") != 1 or history.get("current_release_id") != pointer:
        raise ValueError("media history version or pointer mismatch")
    if pointer not in history["releases"]:
        raise ValueError("active release lacks committed receipt")
    for release_id, saved in history["releases"].items():
        if not HEX.fullmatch(saved["manifest_sha256"]):
            raise ValueError(f"invalid manifest hash: {release_id}")
        for name, full_hash in saved["media"].items():
            if not MEDIA.fullmatch(name) or not HEX.fullmatch(full_hash):
                raise ValueError(f"invalid committed media receipt: {release_id}")
    for release in (root / "releases").iterdir():
        if release.is_symlink():
            raise ValueError("linked retained release")
        if release.is_dir():
            if release.name not in history["releases"]:
                continue  # Interrupted uncommitted candidate remains private under releases/.
            saved = history["releases"][release.name]
            actual = receipt(root, release.name)
            if actual["manifest_sha256"] != saved["manifest_sha256"]:
                raise ValueError(f"committed manifest changed: {release.name}")
            for name, full_hash in saved["media"].items():
                if not marked(root, name) and actual["media"].get(name) != full_hash:
                    raise ValueError(f"committed release media changed: {name}")


def reconcile(root, args, pointer):
    directory = root / ".media-commit-history"
    history_path = directory / "history.json"
    if not history_path.is_file():
        raise ValueError("postmigration history missing; explicit initial mode required")
    history = json.loads(history_path.read_text())
    intent_path = directory / "intent.json"
    if intent_path.exists():
        if not args.frozen_exit_proved:
            raise ValueError("interrupted intent needs frozen old-process exit proof")
        intent = json.loads(intent_path.read_text())
        old, new = intent["old_release_id"], intent["new_release_id"]
        if intent.get("version") != 1:
            raise ValueError("intent and history disagree")
        if old and history["releases"][old]["manifest_sha256"] != intent["old_manifest_sha256"]:
            raise ValueError("old intent digest mismatch")
        if history["current_release_id"] == new and pointer == new and history["releases"][new]["manifest_sha256"] == intent["new_manifest_sha256"]:
            intent_path.unlink()
        elif old != history["current_release_id"]:
            raise ValueError("intent and history disagree")
        elif pointer == old:
            intent_path.unlink()
        elif pointer == new:
            promoted = receipt(root, new)
            if promoted["manifest_sha256"] != intent["new_manifest_sha256"]:
                raise ValueError("new intent digest mismatch")
            if not isinstance(intent.get("new_media"), dict):
                raise ValueError("new intent lacks candidate media hashes")
            if any(intent["new_media"].get(name) != full_hash for name, full_hash in promoted["media"].items()):
                raise ValueError("new intent media mismatch")
            if any(name not in promoted["media"] and not marked(root, name) for name in intent["new_media"]):
                raise ValueError("new intent media missing without marker")
            promoted["media"] = intent["new_media"]
            now = int(time.time())
            if old:
                history["releases"][old]["last_active_end"] = now
            promoted["first_committed_at"] = history["releases"].get(new, {}).get("first_committed_at", now)
            history["releases"][new] = promoted
            history["transitions"].append({"from": old, "to": new, "at": now})
            history["current_release_id"] = new
            atomic_json(history_path, history)
            intent_path.unlink()
        else:
            raise ValueError("pointer matches neither intent side")
        descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
    validate_history(root, history, pointer)
    return history


def initial_history(root, args, pointer, shared):
    if (root / ".media-commit-history").exists() or (root / ".media-commit-migrated.json").exists():
        raise ValueError("migration already attempted; missing history is fatal")
    if not args.old_route_current_only:
        raise ValueError("initial cutover requires verified old current-only route")
    marker_root = root / ".media-tombstones"
    if marker_root.exists() and any(path.is_file() for path in marker_root.rglob("*")) and not args.old_marker_deny_proved:
        raise ValueError("legacy marker exception requires old-route readback and dual-origin 404 proof")
    now = int(time.time())
    active = receipt(root, pointer)
    history = {"version": 1, "current_release_id": pointer,
               "releases": {pointer: active}, "transitions": [{"from": None, "to": pointer, "at": now}]}
    proven = set(active["media"])
    legacy_evidence = args.legacy_evidence or os.environ.get("AUTOGRAPHS_LEGACY_MEDIA_EVIDENCE")
    evidence = json.loads(outside(legacy_evidence, root).read_text()) if legacy_evidence else {}
    archives = evidence.get("archived_receipts", {})
    unknown_retained = set()
    for release in sorted((root / "releases").iterdir()):
        if not release.is_dir() or release.name == pointer:
            continue
        if release.name not in archives:
            value, _ = manifest(root, release.name)
            unknown_retained.update(artifact["path"] for artifact in value["artifacts"]
                                    if artifact.get("variant") is not None
                                    and not marked(root, artifact["path"]))
            continue
        proof = archives[release.name]
        verify_promotion_proof(root, release.name, proof)
        saved = receipt(root, release.name)
        if saved["manifest_sha256"] != proof["manifest_sha256"]:
            raise ValueError("retained manifest differs from trusted promotion proof")
        saved["last_active_end"] = now
        for name, full_hash in saved["media"].items():
            if not marked(root, name):
                if name not in shared:
                    raise ValueError(f"retained committed shared media missing: {name}")
                verified(shared[name], name, expected=full_hash)
                proven.add(name)
        history["releases"][release.name] = saved
    for release_id, proof in archives.items():
        if release_id in history["releases"]:
            continue
        verify_promotion_proof(root, release_id, proof)
        archive = outside(proof["manifest_path"], root)
        saved = receipt(root, release_id, archive, proof.get("media_sha256"))
        if saved["manifest_sha256"] != proof["manifest_sha256"]:
            raise ValueError("archived manifest hash mismatch")
        saved["last_active_end"] = now
        history["releases"][release_id] = saved
        proven.update(saved["media"])
    unresolved = sorted(name for name in shared if name not in proven and not marked(root, name))
    if unknown_retained:
        return history, sorted(set(unresolved) | unknown_retained)
    if unresolved and (not args.trusted_utc or args.old_exit_epoch is None
                       or now < args.old_exit_epoch + WAIT or not args.frozen_exit_proved
                       or args.old_pointer != pointer):
        return history, unresolved
    return history, []


def verify_promotion_proof(root, release_id, proof):
    receipt_path = outside(proof["promotion_receipt_path"], root)
    raw = receipt_path.read_bytes()
    if digest(raw) != proof["promotion_receipt_sha256"]:
        raise ValueError("archived promotion receipt digest mismatch")
    saved = json.loads(raw)
    if saved.get("release_id") != release_id or saved.get("manifest_sha256") != proof["manifest_sha256"] or saved.get("pointer_promoted") is not True:
        raise ValueError("archived promotion receipt does not bind release and manifest")


def audit(args):
    root = Path(args.static_root).resolve(strict=True)
    quarantine = outside(args.quarantine, root)
    record = outside(args.record, root)
    pointer = current(root)
    shared = shared_files(root)
    if args.initial_cutover:
        history, unresolved = initial_history(root, args, pointer, shared)
    else:
        history, unresolved = reconcile(root, args, pointer), []
    now = int(time.time())
    protected = {}
    for release_id, saved in history["releases"].items():
        retained = (root / "releases" / release_id).is_dir()
        end = saved["last_active_end"]
        for name, full_hash in saved["media"].items():
            if retained or end is None or now < end + WINDOW:
                protected[name] = full_hash
    keep, move = [], []
    for name, path in shared.items():
        if marked(root, name):
            move.append(name)
        elif name in protected:
            verified(path, name, expected=protected[name])
            keep.append(name)
        else:
            move.append(name)
    for name in protected:
        if not marked(root, name) and name not in shared and name not in history["releases"][pointer]["media"]:
            raise ValueError(f"committed shared media missing: {name}")
    marker_hash = marker_digest(root)
    report = {"current_release_id": pointer, "marker_digest": marker_hash,
              "history_digest": digest(json.dumps(history, sort_keys=True).encode()),
              "kept": sorted(keep), "quarantine": sorted(move), "unresolved": unresolved,
              "old_exit_epoch": args.old_exit_epoch, "old_pointer": args.old_pointer}
    report_path = record.parent / f"audit-report-{time.time_ns()}.json"
    atomic_json(report_path, report)
    if args.report_only:
        print(f"Read-only inventory: {report_path}")
        return 2 if unresolved else 0
    if unresolved:
        print("Cutover blocked; shared paths lack committed provenance: " + ", ".join(unresolved), file=sys.stderr)
        return 2
    quarantine.mkdir(parents=True, exist_ok=True, mode=0o700)
    os.chmod(quarantine, 0o700)
    for name in move:
        if (quarantine / name).exists():
            raise ValueError(f"quarantine collision: {name}")
    for name in move:
        destination = quarantine / name
        destination.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
        shutil.move(str(shared[name]), str(destination))
    if args.initial_cutover:
        history_path = root / ".media-commit-history/history.json"
        atomic_json(history_path, history)
        atomic_json(root / ".media-commit-migrated.json", {"version": 1, "history_digest": digest(history_path.read_bytes())})
    history_path = root / ".media-commit-history/history.json"
    payload = {"release_tag": args.release_tag, "controller_digest": args.controller_digest,
               "current_target": "releases/" + pointer,
               "current_manifest_sha256": history["releases"][pointer]["manifest_sha256"],
               "history_version": 1, "history_digest": digest(history_path.read_bytes()),
               "marker_digest": marker_hash, "audited_shared_media": sorted(keep),
               "quarantined_shared_media": sorted(move), "old_container_id": args.old_container_id,
               "old_pid": args.old_pid, "bootstrap": False}
    payload["active_probe"] = next((name for name in history["releases"][pointer]["media"] if not marked(root, name)), "")
    payload["retained_probe"] = next((name for name in sorted(keep)
                                      if name not in history["releases"][pointer]["media"]), "")
    marker_root = root / ".media-tombstones"
    payload["marker_paths"] = sorted(path.relative_to(marker_root).as_posix()
                                     for path in marker_root.rglob("*") if path.is_file()) if marker_root.exists() else []
    atomic_json(record, payload)
    print(f"Audited {len(keep)} committed URLs; quarantined {len(move)} uncommitted URLs")
    return 0


def main():
    parser = argparse.ArgumentParser()
    for field in ("static-root", "quarantine", "record", "release-tag", "controller-digest", "old-container-id"):
        parser.add_argument("--" + field, required=True)
    parser.add_argument("--old-pid", required=True, type=int)
    parser.add_argument("--initial-cutover", action="store_true")
    parser.add_argument("--report-only", action="store_true")
    parser.add_argument("--old-route-current-only", action="store_true")
    parser.add_argument("--old-marker-deny-proved", action="store_true")
    parser.add_argument("--frozen-exit-proved", action="store_true")
    parser.add_argument("--old-exit-epoch", type=int)
    parser.add_argument("--old-pointer")
    parser.add_argument("--trusted-utc", action="store_true")
    parser.add_argument("--legacy-evidence")
    return audit(parser.parse_args())


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        print(f"Cutover audit failed closed: {error}", file=sys.stderr)
        sys.exit(1)
