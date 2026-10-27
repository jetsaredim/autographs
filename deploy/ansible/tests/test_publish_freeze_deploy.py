#!/usr/bin/env python3
"""Local regression proof for the production publish-freeze invocation and audit."""

import hashlib
import json
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import textwrap
import time
import unittest

import yaml


REPO = Path(__file__).resolve().parents[3]
AUDIT = REPO / "deploy/ansible/roles/autographs_deploy/files/publish_freeze_audit.py"
WORKFLOW = REPO / ".github/workflows/deploy.yml"
PLAYBOOKS = REPO / "deploy/ansible/playbooks"
PAYLOAD = b"RIFF0000WEBPcommitted WebP fixture"


def media_path(label: str) -> str:
    return f"media/item/{label}-detail-{hashlib.sha256(PAYLOAD).hexdigest()[:16]}.webp"


class MediaAuditFixture:
    def __init__(self, root: Path):
        self.root = root
        self.static = root / "static"
        self.quarantine = root / "private/quarantine"
        self.record = root / "private/freeze.json"
        self.current = self.static / "releases/active"
        self.current.mkdir(parents=True)
        (self.static / "current").symlink_to("releases/active")

    def add_release_media(self, release: Path, relative: str, shared: bool = True):
        target = release / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(PAYLOAD)
        (release / "manifest.json").write_text(
            json.dumps({"release_id": release.name, "artifacts": [{"path": relative, "variant": "detail", "content_type": "image/webp", "byte_size": len(PAYLOAD)}]}),
            encoding="utf-8",
        )
        if shared:
            shared_target = self.static / relative
            shared_target.parent.mkdir(parents=True, exist_ok=True)
            shared_target.write_bytes(PAYLOAD)

    def audit(self, *extra: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable, str(AUDIT), "--static-root", str(self.static),
                "--quarantine", str(self.quarantine), "--record", str(self.record),
                "--release-tag", "v1.2.3", "--controller-digest", "sha256:" + "a" * 64,
                "--old-container-id", "old-container", "--old-pid", "12345",
                *extra,
            ],
            capture_output=True,
            text=True,
            check=False,
        )

    def promote(self, release_id: str, relative: str, *, after_pointer: bool = True):
        """Model the Rust writer's durable intent and pointer boundary."""
        previous = json.loads((self.static / ".media-commit-history/history.json").read_text())
        old = previous["current_release_id"]
        release = self.static / "releases" / release_id
        release.mkdir()
        self.add_release_media(release, relative, shared=False)
        digest = hashlib.sha256((release / "manifest.json").read_bytes()).hexdigest()
        intent = {"version": 1, "old_release_id": old, "new_release_id": release_id,
                  "old_manifest_sha256": previous["releases"][old]["manifest_sha256"],
                  "new_manifest_sha256": digest,
                  "new_media": {relative: hashlib.sha256(PAYLOAD).hexdigest()}}
        (self.static / ".media-commit-history/intent.json").write_text(json.dumps(intent))
        if after_pointer:
            (self.static / "current").unlink()
            (self.static / "current").symlink_to("releases/" + release_id)
        return intent


class PublishFreezeDeploymentTests(unittest.TestCase):
    def test_later_deploy_keeps_committed_a_through_two_reaudits_and_prune(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            old = media_path("old")
            new = media_path("new")
            fixture.add_release_media(fixture.current, old)
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            fixture.promote("release-b", new)
            for _ in range(2):
                result = fixture.audit("--frozen-exit-proved")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue((fixture.static / old).is_file())
            history = json.loads((fixture.static / ".media-commit-history/history.json").read_text())
            self.assertEqual(history["current_release_id"], "release-b")
            self.assertIsNotNone(history["releases"]["active"]["last_active_end"])
            history["releases"]["active"]["last_active_end"] = 0
            (fixture.static / ".media-commit-history/history.json").write_text(json.dumps(history))
            self.assertEqual(fixture.audit().returncode, 0)  # retained A still protects URL
            self.assertTrue((fixture.static / old).exists())
            shutil.rmtree(fixture.current)
            self.assertEqual(fixture.audit().returncode, 0)
            self.assertFalse((fixture.static / old).exists())

    def test_marked_active_deletion_passes_but_unmarked_missing_fails(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            relative = media_path("active")
            fixture.add_release_media(fixture.current, relative)
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            marker = fixture.static / ".media-tombstones" / relative
            marker.parent.mkdir(parents=True)
            marker.write_bytes(b"")
            (fixture.current / relative).unlink()
            self.assertEqual(fixture.audit().returncode, 0)
            self.assertFalse((fixture.static / relative).exists())
            marker.unlink()
            self.assertEqual(fixture.audit().returncode, 1)

    def test_uncommitted_intent_cannot_authorize_new_shared_url(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            old, new = media_path("old"), media_path("new")
            fixture.add_release_media(fixture.current, old)
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            fixture.promote("release-b", new, after_pointer=False)
            shared_new = fixture.static / new
            shared_new.parent.mkdir(parents=True, exist_ok=True)
            shared_new.write_bytes(PAYLOAD)
            self.assertEqual(fixture.audit("--frozen-exit-proved").returncode, 0)
            self.assertFalse(shared_new.exists())
            self.assertTrue((fixture.quarantine / new).exists())

    def test_pruned_legacy_release_needs_archive_or_conservative_wait(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            current_path, prior_path = media_path("current"), media_path("pruned")
            fixture.add_release_media(fixture.current, current_path)
            prior = fixture.static / prior_path
            prior.parent.mkdir(parents=True, exist_ok=True)
            prior.write_bytes(PAYLOAD)
            cutoff = int(time.time()) - 86_400 - 60 - 290
            args = ("--initial-cutover", "--old-route-current-only", "--frozen-exit-proved",
                    "--trusted-utc", "--old-pointer", "active", "--old-exit-epoch", str(cutoff))
            result = fixture.audit("--report-only", *args)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertTrue(prior.exists())
            self.assertFalse((fixture.static / ".media-commit-history").exists())
            result = fixture.audit(*args)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertTrue(prior.exists())
            cutoff -= 10
            args = args[:-1] + (str(cutoff),)
            self.assertEqual(fixture.audit("--report-only", *(value for value in args if value != "--trusted-utc")).returncode, 2)
            mismatched = args[:-3] + ("changed-pointer", "--old-exit-epoch", str(cutoff))
            self.assertEqual(fixture.audit("--report-only", *mismatched).returncode, 2)
            self.assertEqual(fixture.audit("--report-only", *args).returncode, 0)
            self.assertTrue(prior.exists())
            self.assertEqual(fixture.audit(*args).returncode, 0)
            self.assertFalse(prior.exists())
            self.assertTrue((fixture.quarantine / prior_path).exists())

    def test_unproven_retained_reference_blocks_after_legacy_wait(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            fixture.add_release_media(fixture.current, media_path("current"))
            retained = fixture.static / "releases/retained"
            retained.mkdir()
            relative = media_path("retained")
            fixture.add_release_media(retained, relative)
            cutoff = int(time.time()) - 86_400 - 60 - 400
            result = fixture.audit("--report-only", "--initial-cutover", "--old-route-current-only",
                                   "--frozen-exit-proved", "--trusted-utc", "--old-pointer", "active",
                                   "--old-exit-epoch", str(cutoff))
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertTrue((fixture.static / relative).exists())
            self.assertFalse((fixture.static / ".media-commit-history").exists())

    def test_trusted_pruned_archive_import_preserves_shared_url(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            current_path, prior_path = media_path("current"), media_path("archived")
            fixture.add_release_media(fixture.current, current_path)
            archive = fixture.root / "private/archived-manifest.json"
            archive.parent.mkdir(parents=True)
            archive.write_text(json.dumps({"release_id": "pruned", "artifacts": [
                {"path": prior_path, "variant": "detail", "content_type": "image/webp", "byte_size": len(PAYLOAD)}]}))
            shared = fixture.static / prior_path
            shared.parent.mkdir(parents=True, exist_ok=True)
            shared.write_bytes(PAYLOAD)
            manifest_hash = hashlib.sha256(archive.read_bytes()).hexdigest()
            promotion = fixture.root / "private/promotion-receipt.json"
            promotion.write_text(json.dumps({"release_id": "pruned", "manifest_sha256": manifest_hash,
                                             "pointer_promoted": True}))
            proof = fixture.root / "private/legacy-evidence.json"
            proof.write_text(json.dumps({"archived_receipts": {"pruned": {
                "promotion_receipt_path": str(promotion),
                "promotion_receipt_sha256": hashlib.sha256(promotion.read_bytes()).hexdigest(),
                "manifest_path": str(archive), "manifest_sha256": manifest_hash,
                "media_sha256": {prior_path: hashlib.sha256(PAYLOAD).hexdigest()}}}}))
            result = fixture.audit("--initial-cutover", "--old-route-current-only",
                                   "--legacy-evidence", str(proof))
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(shared.exists())
            self.assertIn("pruned", json.loads((fixture.static / ".media-commit-history/history.json").read_text())["releases"])

    def test_postpointer_intent_is_reconciled_without_losing_old_url(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            old, new = media_path("old"), media_path("new")
            fixture.add_release_media(fixture.current, old)
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            fixture.promote("release-b", new)
            result = fixture.audit("--frozen-exit-proved")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse((fixture.static / ".media-commit-history/intent.json").exists())
            self.assertTrue((fixture.static / old).exists())
            self.assertEqual(fixture.audit().returncode, 0)

    def test_preexisting_and_stop_time_candidate_bytes_are_quarantined(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            committed = media_path("committed")
            before_stop = media_path("before-stop")
            during_stop = media_path("during-stop")
            fixture.add_release_media(fixture.current, committed)
            for relative in (before_stop, during_stop):
                path = fixture.static / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(PAYLOAD)
            result = fixture.audit("--initial-cutover", "--old-route-current-only", "--report-only")
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertTrue((fixture.static / committed).is_file())
            self.assertFalse(fixture.record.exists())
            for relative in (before_stop, during_stop):
                self.assertTrue((fixture.static / relative).is_file())
                self.assertFalse((fixture.quarantine / relative).exists())
            self.assertFalse((fixture.static / ".media-commit-history").exists())

    def test_retained_manifest_is_not_publication_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            committed = media_path("committed")
            ambiguous = media_path("ambiguous")
            fixture.add_release_media(fixture.current, committed)
            retained = fixture.static / "releases/retained"
            retained.mkdir(parents=True)
            fixture.add_release_media(retained, ambiguous)
            result = fixture.audit("--initial-cutover", "--old-route-current-only", "--report-only")
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertFalse(fixture.record.exists())
            self.assertTrue((fixture.static / ambiguous).is_file())
            self.assertFalse((fixture.quarantine / ambiguous).exists())

    def test_successful_reaudit_replaces_record_after_candidate_quarantine(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            fixture.add_release_media(fixture.current, media_path("committed"))
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            self.assertTrue(fixture.record.exists())
            path = fixture.static / media_path("late-candidate")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(PAYLOAD)
            self.assertEqual(fixture.audit().returncode, 0)
            self.assertTrue(fixture.record.exists())
            self.assertTrue((fixture.quarantine / media_path("late-candidate")).is_file())

    def test_failed_reaudit_never_moves_committed_bytes_or_replaces_record(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            path = media_path("committed")
            fixture.add_release_media(fixture.current, path)
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            before = fixture.record.read_bytes()
            (fixture.static / path).write_bytes(b"corrupt")
            result = fixture.audit()
            self.assertEqual(result.returncode, 1)
            self.assertTrue((fixture.static / path).exists())
            self.assertFalse((fixture.quarantine / path).exists())
            self.assertEqual(fixture.record.read_bytes(), before)

    def test_process_exit_cancels_waiter_before_any_post_exit_write(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "candidate-after-exit.webp"
            worker = textwrap.dedent(
                """
                import pathlib, sys, threading, time
                lock = threading.Lock()
                lock.acquire()
                def queued_publish():
                    with lock:
                        pathlib.Path(sys.argv[1]).write_bytes(b'candidate')
                threading.Thread(target=queued_publish).start()
                print('queued', flush=True)
                time.sleep(60)
                lock.release()
                """
            )
            process = subprocess.Popen(
                [sys.executable, "-c", worker, str(output)],
                stdout=subprocess.PIPE,
                text=True,
            )
            try:
                self.assertEqual(process.stdout.readline().strip(), "queued")
                process.send_signal(signal.SIGTERM)
                process.wait(timeout=5)
                time.sleep(0.05)
                self.assertFalse(output.exists())
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait()
                process.stdout.close()

    def test_actual_production_workflow_requires_preflight_and_explicit_unfreeze(self):
        workflow = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
        steps = workflow["jobs"]["production"]["steps"]
        named = {step.get("name"): (index, step) for index, step in enumerate(steps)}
        preflight = named["Verify frozen publisher cutover preflight"]
        deploy = named["Run full deployment"]
        readiness = named["Verify frozen origin readiness"]
        unfreeze = named["Explicitly unfreeze verified production publication"]
        gate = named["Validate release completion gate"]
        self.assertLess(preflight[0], deploy[0])
        self.assertLess(deploy[0], readiness[0])
        self.assertLess(readiness[0], unfreeze[0])
        self.assertLess(unfreeze[0], gate[0])
        self.assertEqual(preflight[1]["with"]["playbook"], "playbooks/publish-freeze-preflight.yml")
        self.assertIn("autographs_deploy_publish_frozen=true", deploy[1]["with"]["options"])
        self.assertIn("steps.publish_freeze_preflight.outcome == 'success'", deploy[1]["if"])
        self.assertIn("production_mutation_required == 'true'", preflight[1]["if"])
        self.assertIn("production_mutation_required == 'true'", unfreeze[1]["if"])
        self.assertIn("autographs_publish_unfreeze_verify_only=true", readiness[1]["with"]["options"])
        self.assertIn("autographs_publish_unfreeze_verify_only=false", unfreeze[1]["with"]["options"])
        self.assertIn("PUBLISH_UNFREEZE_OUTCOME", gate[1]["env"])

    def test_deploy_and_rollback_source_order_guard(self):
        preflight = yaml.safe_load((PLAYBOOKS / "publish-freeze-preflight.yml").read_text())[0]["tasks"]
        task_names = [task["name"] for task in preflight]
        drain = preflight[task_names.index("Verify and drain an installed runtime")]["block"]
        drain_names = [task["name"] for task in drain]
        self.assertLess(drain_names.index("Inventory initial shared namespace without mutation before old process exit"),
                        drain_names.index("Stop old controller and all accepted or queued old requests"))
        self.assertLess(drain_names.index("Wait for old controller unit to become inactive"),
                        drain_names.index("Inventory initial shared namespace without mutation after old process exit"))
        self.assertLess(drain_names.index("Refuse cutover while initial provenance is unresolved"),
                        drain_names.index("Quarantine uncommitted shared URLs only after provenance classification"))
        role = yaml.safe_load(
            (REPO / "deploy/ansible/roles/autographs_deploy/tasks/main.yml").read_text()
        )
        role_names = [task["name"] for task in role]
        self.assertLess(role_names.index("Require matching preflight and stopped old publisher"),
                        role_names.index("Copy Caddyfile"))
        self.assertLess(role_names.index("Require unchanged committed release through role reaudit"),
                        role_names.index("Copy Caddyfile"))
        self.assertLess(role_names.index("Require dual-origin denial for changed marker inventory"),
                        role_names.index("Copy Caddyfile"))
        self.assertLess(role_names.index("Start reviewed controller before new Caddy routing"),
                        role_names.index("Restart Caddy only after reviewed controller readiness"))
        unfreeze = yaml.safe_load((PLAYBOOKS / "publish-unfreeze.yml").read_text())[0]["tasks"]
        unfreeze_names = [task["name"] for task in unfreeze]
        self.assertLess(unfreeze_names.index("Verify current pointer, manifest, history, and marker inventory before unfreeze"),
                        unfreeze_names.index("Remove managed gate only after readiness"))
        self.assertLess(unfreeze_names.index("Require dual-origin old committed media reachability"),
                        unfreeze_names.index("Remove managed gate only after readiness"))
        rollback = (PLAYBOOKS / "controller-rollback.yml").read_text()
        self.assertIn("Refuse rollback to an unverified old publisher", rollback)


if __name__ == "__main__":
    unittest.main()
