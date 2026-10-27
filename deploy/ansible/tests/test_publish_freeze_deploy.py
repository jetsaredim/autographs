#!/usr/bin/env python3
"""Local regression proof for the production publish-freeze invocation and audit."""

import hashlib
import json
from pathlib import Path
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
            json.dumps({"release_id": release.name, "artifacts": [{"path": relative, "variant": "detail"}]}),
            encoding="utf-8",
        )
        if shared:
            shared_target = self.static / relative
            shared_target.parent.mkdir(parents=True, exist_ok=True)
            shared_target.write_bytes(PAYLOAD)

    def audit(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable, str(AUDIT), "--static-root", str(self.static),
                "--quarantine", str(self.quarantine), "--record", str(self.record),
                "--release-tag", "v1.2.3", "--controller-digest", "sha256:" + "a" * 64,
                "--old-container-id", "old-container", "--old-pid", "12345",
            ],
            capture_output=True,
            text=True,
            check=False,
        )


class PublishFreezeDeploymentTests(unittest.TestCase):
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
            result = fixture.audit()
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertTrue((fixture.static / committed).is_file())
            self.assertFalse(fixture.record.exists())
            for relative in (before_stop, during_stop):
                self.assertFalse((fixture.static / relative).exists())
                self.assertTrue((fixture.quarantine / relative).is_file())
            result = fixture.audit()
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(fixture.record.read_text())["audited_shared_media"], [committed])

    def test_retained_manifest_is_not_publication_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            committed = media_path("committed")
            ambiguous = media_path("ambiguous")
            fixture.add_release_media(fixture.current, committed)
            retained = fixture.static / "releases/retained"
            retained.mkdir(parents=True)
            fixture.add_release_media(retained, ambiguous)
            result = fixture.audit()
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn(ambiguous, result.stderr)
            self.assertFalse(fixture.record.exists())
            self.assertTrue((fixture.quarantine / ambiguous).is_file())

    def test_stale_record_is_removed_before_failed_reaudit(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            fixture.add_release_media(fixture.current, media_path("committed"))
            self.assertEqual(fixture.audit().returncode, 0)
            self.assertTrue(fixture.record.exists())
            path = fixture.static / media_path("late-candidate")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(PAYLOAD)
            self.assertEqual(fixture.audit().returncode, 2)
            self.assertFalse(fixture.record.exists())

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
        self.assertLess(task_names.index("Verify and drain an installed runtime"), len(task_names))
        role = yaml.safe_load(
            (REPO / "deploy/ansible/roles/autographs_deploy/tasks/main.yml").read_text()
        )
        role_names = [task["name"] for task in role]
        self.assertLess(role_names.index("Require matching preflight and stopped old publisher"),
                        role_names.index("Copy Caddyfile"))
        self.assertLess(role_names.index("Start reviewed controller before new Caddy routing"),
                        role_names.index("Restart Caddy only after reviewed controller readiness"))
        rollback = (PLAYBOOKS / "controller-rollback.yml").read_text()
        self.assertIn("Refuse rollback to an unverified old publisher", rollback)


if __name__ == "__main__":
    unittest.main()
