#!/usr/bin/env python3
"""Local regression proof for the production publish-freeze invocation and audit."""

import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import textwrap
import threading
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

    def probe(self, *extra: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(AUDIT), "--static-root", str(self.static),
             "--probe-only", *extra], capture_output=True, text=True, check=False,
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
    def test_actual_unfreeze_all_marked_against_real_caddy(self):
        if not shutil.which("docker") or not shutil.which("ansible-playbook"):
            self.fail("Docker and Ansible are mandatory for the actual unfreeze origin proof")
        with tempfile.TemporaryDirectory(prefix="autographs-unfreeze-", dir="/tmp") as temporary:
            root = Path(temporary)
            fixture = MediaAuditFixture(root)
            relative = media_path("sole")
            fixture.add_release_media(fixture.current, relative)
            initial = fixture.audit("--initial-cutover", "--old-route-current-only")
            self.assertEqual(initial.returncode, 0, initial.stderr)
            marker = fixture.static / ".media-tombstones" / relative
            marker.parent.mkdir(parents=True)
            marker.touch()
            (fixture.current / relative).unlink()
            self.assertEqual(fixture.audit().returncode, 0)
            self.assertEqual(json.loads(fixture.record.read_text())["active_probe"]["kind"], "document")

            class Controller(BaseHTTPRequestHandler):
                def do_GET(self):
                    if self.path == "/admin/api/health":
                        body = json.dumps({"release": {"repoVersion": "v1.2.3", "sourceRevision": "b" * 40}}).encode()
                        self.send_response(200)
                    elif self.path == "/admin/api/publish/status" and self.headers.get("Authorization") == "Bearer fixture-token":
                        body = b"{}"
                        self.send_response(200)
                    else:
                        body = b""
                        self.send_response(401)
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)

                def do_POST(self):
                    self.send_response(401)
                    self.end_headers()

                def log_message(self, *_args):
                    pass

            server = ThreadingHTTPServer(("127.0.0.1", 0), Controller)
            server_thread = threading.Thread(target=server.serve_forever, daemon=True)
            server_thread.start()
            container = f"autographs-unfreeze-{os.getpid()}"
            caddy_dir = root / "caddy"
            caddy_dir.mkdir()
            source = (REPO / "deploy/ansible/roles/autographs_deploy/files/Caddyfile").read_text()
            source = source.replace("\n{$AUTOGRAPHS_DOMAIN} {\n", "\n{$AUTOGRAPHS_DOMAIN} {\n\ttls internal\n", 1)
            source = source.replace("reverse_proxy autographs-controller:8080", f"reverse_proxy 127.0.0.1:{server.server_port}")
            gate = """# BEGIN AUTOGRAPHS PUBLISH FREEZE
  handle /admin/api/publish/full {
    header Cache-Control \"no-store\"
    respond \"Publishing temporarily unavailable\" 503
  }
  handle /admin/api/publish/incremental {
    header Cache-Control \"no-store\"
    respond \"Publishing temporarily unavailable\" 503
  }
# END AUTOGRAPHS PUBLISH FREEZE
"""
            frozen_config = source.replace("\thandle /admin/api/* {", gate + "\thandle /admin/api/* {", 1)
            (caddy_dir / "Caddyfile").write_text(frozen_config)
            podman = root / "bin/podman"
            podman.parent.mkdir()
            podman.write_text(textwrap.dedent("""\
                #!/usr/bin/env python3
                import os, pathlib, subprocess, sys
                args = sys.argv[1:]
                if args[:2] == ['volume', 'inspect']:
                    print(os.environ['FIXTURE_STATIC'])
                elif args[:2] == ['image', 'inspect']:
                    print('fixture-image')
                elif args[:1] == ['inspect']:
                    if '.State.Pid' in ' '.join(args):
                        print('old-container 0')
                    elif '.State.Running' in ' '.join(args):
                        print('false')
                    else:
                        print('fixture-image')
                elif args[:1] == ['exec']:
                    if args[2:5] == ['caddy', 'reload', '--config'] and os.environ.get('FIXTURE_BREAK_AFTER_UNFREEZE') == '1':
                        caddyfile = pathlib.Path(os.environ['FIXTURE_CADDYFILE'])
                        manifest = pathlib.Path(os.environ['FIXTURE_STATIC']) / 'current/manifest.json'
                        saved = manifest.with_name('manifest.json.fixture-saved')
                        if '# BEGIN AUTOGRAPHS PUBLISH FREEZE' not in caddyfile.read_text() and not saved.exists():
                            manifest.rename(saved)
                        elif '# BEGIN AUTOGRAPHS PUBLISH FREEZE' in caddyfile.read_text() and saved.exists():
                            saved.rename(manifest)
                    sys.exit(subprocess.run(['docker', 'exec', os.environ['FIXTURE_CADDY'], *args[2:]], check=False).returncode)
                else:
                    sys.exit('unsupported fixture podman call: ' + repr(args))
                """))
            podman.chmod(0o755)
            systemctl = root / "bin/systemctl"
            systemctl.write_text("#!/bin/sh\nprintf 'inactive\\n'\n")
            systemctl.chmod(0o755)
            timedatectl = root / "bin/timedatectl"
            timedatectl.write_text("#!/bin/sh\nprintf 'yes\\n'\n")
            timedatectl.chmod(0o755)
            quadlet = root / "autographs-caddy.container"
            quadlet.write_text("[Unit]\nAfter=network-online.target autographs-controller.service\nWants=network-online.target autographs-controller.service\n")
            (root / "app.env").write_text("AUTOGRAPHS_CONTROLLER_IMAGE=ghcr.io/example/controller:v1.2.3\nAUTOGRAPHS_CONTROLLER_DIGEST=sha256:" + "a" * 64 + "\n")
            inventory = root / "inventory"
            inventory.write_text("[runtime]\nlocalhost ansible_connection=local ansible_become=false\n")
            docker = ["docker", "run", "--detach", "--rm", "--name", container, "--network", "host",
                      "--env", "AUTOGRAPHS_DOMAIN=media-probe.invalid",
                      "--volume", f"{caddy_dir}:/etc/caddy",
                      "--volume", f"{fixture.static}:/srv/autographs/static:ro",
                      "docker.io/library/caddy:2-alpine"]
            started = False
            try:
                result = subprocess.run(docker, capture_output=True, text=True, check=False)
                self.assertEqual(result.returncode, 0, result.stderr)
                started = True
                certificate = root / "root.crt"
                for _ in range(40):
                    cert = subprocess.run(["docker", "exec", container, "cat", "/data/caddy/pki/authorities/local/root.crt"],
                                          capture_output=True, check=False)
                    if cert.returncode == 0:
                        certificate.write_bytes(cert.stdout)
                        break
                    time.sleep(0.2)
                logs = subprocess.run(["docker", "logs", container], capture_output=True, text=True, check=False)
                self.assertTrue(certificate.is_file(), "Caddy internal CA was not created: " + logs.stderr)
                env = os.environ.copy()
                env.update({"PATH": str(podman.parent) + os.pathsep + env["PATH"],
                            "FIXTURE_STATIC": str(fixture.static), "FIXTURE_CADDY": container,
                            "FIXTURE_CADDYFILE": str(caddy_dir / "Caddyfile"),
                            "CURL_CA_BUNDLE": str(certificate), "AUTOGRAPHS_OPERATOR_API_TOKEN": "fixture-token",
                            "AUTOGRAPHS_PUBLISH_FIXTURE_MODE": "1", "ANSIBLE_CONFIG": str(REPO / "deploy/ansible/ansible.cfg"),
                            "ANSIBLE_LOCAL_TEMP": "/tmp/ansible-local", "ANSIBLE_REMOTE_TEMP": "/tmp/ansible-remote"})
                variables = {"freeze_record_path": str(fixture.record), "caddyfile_path": str(caddy_dir / "Caddyfile"),
                             "frozen_stage_path": str(root / "frozen-stage"), "autographs_deploy_caddy_path": str(caddy_dir),
                             "frozen_backup_path": str(root / "private/old-frozen-Caddyfile"),
                             "quarantine_path": str(fixture.quarantine), "legacy_wait_record_path": str(root / "private/legacy-wait.json"),
                             "caddy_quadlet_path": str(quadlet),
                             "fixture_static_root": str(fixture.static),
                             "fixture_file_owner": str(os.getuid()),
                             "fixture_file_group": str(os.getgid()),
                             "audit_script_path": str(root / "private/publish_freeze_audit.py"),
                             "rollback_script_path": str(root / "private/static_release_rollback.py"),
                             "app_env_path": str(root / "app.env"),
                             "approved_digests_path": str(root / "approved-digests"),
                             "autographs_deploy_repo_version": "v1.2.3", "autographs_deploy_controller_digest": "sha256:" + "a" * 64,
                             "autographs_deploy_source_revision": "b" * 40, "autographs_deploy_domain": "media-probe.invalid",
                             "autographs_publish_unfreeze_fixture_mode": True,
                             "autographs_publish_freeze_fixture_mode": True,
                             "autographs_publish_file_owner": str(os.getuid()),
                             "autographs_publish_file_group": str(os.getgid()), "ansible_become": False}

                def invoke(verify_only: bool, break_after_restart: bool = False):
                    invocation_env = {**env, "FIXTURE_BREAK_AFTER_UNFREEZE": "1" if break_after_restart else "0"}
                    return subprocess.run(["ansible-playbook", "-i", str(inventory),
                                           str(PLAYBOOKS / "publish-unfreeze.yml"), "--extra-vars",
                                           json.dumps({**variables, "autographs_publish_unfreeze_verify_only": verify_only})],
                                          capture_output=True, text=True, check=False, env=invocation_env)

                def invoke_preflight():
                    return subprocess.run(["ansible-playbook", "-i", str(inventory),
                                           str(PLAYBOOKS / "publish-freeze-preflight.yml"),
                                           "--extra-vars", json.dumps(variables)],
                                          capture_output=True, text=True, check=False, env=env)

                def invoke_role_reaudit():
                    return subprocess.run(["ansible-playbook", "-i", str(inventory),
                                           str(PLAYBOOKS / "publish-freeze-role-reaudit-test.yml"),
                                           "--tags", "publish_reaudit", "--extra-vars", json.dumps(variables)],
                                          capture_output=True, text=True, check=False, env=env)

                def origin(path: str, method: str = "GET") -> str:
                    for _ in range(20):
                        response = subprocess.run(
                            ["curl", "--silent", "--dump-header", "-", "--output", "/dev/null",
                             "--write-out", "HTTP_STATUS=%{http_code}", "--noproxy", "*",
                             "--resolve", "media-probe.invalid:443:127.0.0.1", "--request", method,
                             "https://media-probe.invalid" + path],
                            capture_output=True, text=True, check=False, env=env)
                        if response.returncode == 0:
                            break
                        time.sleep(0.2)
                    self.assertEqual(response.returncode, 0, response.stderr)
                    return response.stdout

                self.assertIn("HTTP_STATUS=200", origin("/manifest.json"))
                self.assertIn("HTTP_STATUS=404", origin("/" + relative))
                frozen_post = origin("/admin/api/publish/full", "POST")
                self.assertIn("HTTP_STATUS=503", frozen_post)
                self.assertIn("cache-control: no-store", frozen_post.lower())

                preflight = invoke_preflight()
                self.assertEqual(preflight.returncode, 0, preflight.stdout + preflight.stderr)
                self.assertIn("Require dual-origin active 200 and every exact marker 404", preflight.stdout)
                self.assertTrue(fixture.record.exists())
                self.assertNotIn("autographs-controller.service", quadlet.read_text())

                reaudit = invoke_role_reaudit()
                self.assertEqual(reaudit.returncode, 0, reaudit.stdout + reaudit.stderr)
                self.assertIn("Require refreshed active 200 and exact marker 404 after Caddy restart", reaudit.stdout)
                self.assertTrue(fixture.record.exists())

                verify = invoke(True)
                self.assertEqual(verify.returncode, 0, verify.stdout + verify.stderr)
                self.assertTrue(fixture.record.exists())
                self.assertIn("# BEGIN AUTOGRAPHS PUBLISH FREEZE", (caddy_dir / "Caddyfile").read_text())
                unfreeze = invoke(False)
                self.assertEqual(unfreeze.returncode, 0, unfreeze.stdout + unfreeze.stderr)
                self.assertFalse(fixture.record.exists())
                self.assertNotIn("# BEGIN AUTOGRAPHS PUBLISH FREEZE", (caddy_dir / "Caddyfile").read_text())
                self.assertTrue(marker.exists())
                self.assertIn("HTTP_STATUS=200", origin("/manifest.json"))
                self.assertIn("HTTP_STATUS=404", origin("/" + relative))
                self.assertIn("HTTP_STATUS=401", origin("/admin/api/publish/full", "POST"))
                self.assertLess(unfreeze.stdout.index("Require selected dual-origin 200 and all exact marker 404 after restart"),
                                unfreeze.stdout.index("Remove freeze record only after all postchecks"))

                (caddy_dir / "Caddyfile").write_text(frozen_config)
                reload_result = subprocess.run(["docker", "exec", container, "caddy", "reload", "--config", "/etc/caddy/Caddyfile"],
                                               capture_output=True, text=True, check=False)
                self.assertEqual(reload_result.returncode, 0, reload_result.stderr)
                self.assertEqual(fixture.audit().returncode, 0)

                failed = invoke(False, break_after_restart=True)
                self.assertNotEqual(failed.returncode, 0, failed.stdout + failed.stderr)
                self.assertIn("Restore frozen Caddyfile after failed unfreeze", failed.stdout)
                self.assertTrue(fixture.record.exists())
                self.assertIn("# BEGIN AUTOGRAPHS PUBLISH FREEZE", (caddy_dir / "Caddyfile").read_text())
                self.assertIn("HTTP_STATUS=503", origin("/admin/api/publish/full", "POST"))

                def promote_release(release_id: str, paths: list[str]):
                    history = json.loads((fixture.static / ".media-commit-history/history.json").read_text())
                    old_id = history["current_release_id"]
                    release = fixture.static / "releases" / release_id
                    release.mkdir()
                    artifacts = []
                    for name in paths:
                        target = release / name
                        target.parent.mkdir(parents=True, exist_ok=True)
                        target.write_bytes(PAYLOAD)
                        artifacts.append({"path": name, "variant": "detail", "content_type": "image/webp",
                                          "byte_size": len(PAYLOAD)})
                    (release / "manifest.json").write_text(json.dumps({"release_id": release_id, "artifacts": artifacts}))
                    manifest_hash = hashlib.sha256((release / "manifest.json").read_bytes()).hexdigest()
                    intent = {"version": 1, "old_release_id": old_id, "new_release_id": release_id,
                              "old_manifest_sha256": history["releases"][old_id]["manifest_sha256"],
                              "new_manifest_sha256": manifest_hash,
                              "new_media": {name: hashlib.sha256(PAYLOAD).hexdigest() for name in paths}}
                    (fixture.static / ".media-commit-history/intent.json").write_text(json.dumps(intent))
                    (fixture.static / "current").unlink()
                    (fixture.static / "current").symlink_to("releases/" + release_id)
                    result = fixture.audit("--frozen-exit-proved")
                    self.assertEqual(result.returncode, 0, result.stderr)
                    return release

                def refreeze():
                    (caddy_dir / "Caddyfile").write_text(frozen_config)
                    result = subprocess.run(["docker", "exec", container, "caddy", "reload", "--config", "/etc/caddy/Caddyfile"],
                                            capture_output=True, text=True, check=False)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertIn("HTTP_STATUS=503", origin("/admin/api/publish/full", "POST"))

                def prove_frozen_deploy(expected_kind: str):
                    before = fixture.record.read_bytes()
                    preflight_result = invoke_preflight()
                    self.assertEqual(preflight_result.returncode, 0, preflight_result.stdout + preflight_result.stderr)
                    role_result = invoke_role_reaudit()
                    self.assertEqual(role_result.returncode, 0, role_result.stdout + role_result.stderr)
                    self.assertEqual(json.loads(fixture.record.read_text())["active_probe"]["kind"], expected_kind)
                    self.assertTrue(before)
                    self.assertIn("HTTP_STATUS=503", origin("/admin/api/publish/full", "POST"))

                def prove_unfreeze(probe_path: str, denied: list[str]):
                    verify_result = invoke(True)
                    self.assertEqual(verify_result.returncode, 0, verify_result.stdout + verify_result.stderr)
                    self.assertTrue(fixture.record.exists())
                    mutate_result = invoke(False)
                    self.assertEqual(mutate_result.returncode, 0, mutate_result.stdout + mutate_result.stderr)
                    self.assertFalse(fixture.record.exists())
                    self.assertIn("HTTP_STATUS=200", origin("/" + probe_path))
                    for name in denied:
                        self.assertIn("HTTP_STATUS=404", origin("/" + name))
                    self.assertIn("HTTP_STATUS=401", origin("/admin/api/publish/full", "POST"))

                # Empty current release: the same production calls must use a document probe.
                promote_release("empty", [])
                prove_frozen_deploy("document")
                prove_unfreeze("manifest.json", [relative])

                # A mixed release must select verified unmarked media and deny marked siblings.
                refreeze()
                good, bad = media_path("mixed-good"), media_path("mixed-marked")
                mixed_release = promote_release("mixed", [good, bad])
                marked_bad = fixture.static / ".media-tombstones" / bad
                marked_bad.parent.mkdir(parents=True, exist_ok=True)
                marked_bad.touch()
                (mixed_release / bad).unlink()
                self.assertEqual(fixture.audit().returncode, 0)
                prove_frozen_deploy("media")

                # Missing unmarked bytes fail before any document fallback or gate removal.
                (mixed_release / good).unlink()
                missing = invoke(True)
                self.assertNotEqual(missing.returncode, 0, missing.stdout + missing.stderr)
                self.assertTrue(fixture.record.exists())
                self.assertIn("HTTP_STATUS=503", origin("/admin/api/publish/full", "POST"))
                (mixed_release / good).write_bytes(PAYLOAD)

                # An exact marker added after the role audit invalidates the frozen record.
                drift_name = media_path("drift")
                drift_marker = fixture.static / ".media-tombstones" / drift_name
                drift_marker.parent.mkdir(parents=True, exist_ok=True)
                drift_marker.touch()
                drift = invoke(True)
                self.assertNotEqual(drift.returncode, 0, drift.stdout + drift.stderr)
                self.assertTrue(fixture.record.exists())
                self.assertIn("HTTP_STATUS=503", origin("/admin/api/publish/full", "POST"))
                drift_marker.unlink()
                prove_unfreeze(good, [relative, bad])

                # Journaled rollback to the all-marked prior release retains mixed-good.
                refreeze()
                rollback = subprocess.run(
                    [sys.executable, str(REPO / "deploy/ansible/roles/autographs_deploy/files/static_release_rollback.py"),
                     "--static-root", str(fixture.static), "--target-release-id", "active", "--frozen-exit-proved"],
                    capture_output=True, text=True, check=False)
                self.assertEqual(rollback.returncode, 0, rollback.stderr)
                self.assertEqual(fixture.audit().returncode, 0)
                retained_good = fixture.static / good
                self.assertTrue(retained_good.is_file())
                retained_good.unlink()
                retained_failure = invoke_preflight()
                self.assertNotEqual(retained_failure.returncode, 0, retained_failure.stdout + retained_failure.stderr)
                self.assertTrue(fixture.record.exists())
                self.assertIn("HTTP_STATUS=503", origin("/admin/api/publish/full", "POST"))
                retained_good.parent.mkdir(parents=True, exist_ok=True)
                retained_good.write_bytes(PAYLOAD)
                prove_frozen_deploy("document")
                prove_unfreeze("manifest.json", [relative, bad])
                self.assertIn("HTTP_STATUS=200", origin("/" + good))
            finally:
                if started:
                    subprocess.run(["docker", "stop", container], capture_output=True, check=False)
                server.shutdown()
                server.server_close()
                server_thread.join(timeout=2)

    def test_versioned_probe_selects_document_for_empty_and_all_marked(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            (fixture.current / "manifest.json").write_text(
                json.dumps({"release_id": "active", "artifacts": []}))
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            state = json.loads(fixture.probe().stdout)
            self.assertEqual(state["active_probe"], {"version": 1, "kind": "document", "path": "manifest.json"})
            relative = media_path("sole")
            fixture.add_release_media(fixture.current, relative)
            self.assertEqual(fixture.audit().returncode, 1)  # committed manifest may not drift

        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            relative = media_path("sole")
            fixture.add_release_media(fixture.current, relative)
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            marker = fixture.static / ".media-tombstones" / relative
            marker.parent.mkdir(parents=True)
            marker.touch()
            (fixture.current / relative).unlink()
            self.assertEqual(fixture.audit().returncode, 0)
            state = json.loads(fixture.probe().stdout)
            self.assertEqual(state["active_probe"], {"version": 1, "kind": "document", "path": "manifest.json"})
            self.assertEqual(state["marker_paths"], [relative])
            self.assertEqual(state["marker_digest"], json.loads(fixture.record.read_text())["marker_digest"])

    def test_probe_validates_every_unmarked_media_and_never_falls_back_on_corruption(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            good, bad = media_path("good"), media_path("bad")
            fixture.add_release_media(fixture.current, good)
            (fixture.current / bad).parent.mkdir(parents=True, exist_ok=True)
            (fixture.current / bad).write_bytes(PAYLOAD)
            value = json.loads((fixture.current / "manifest.json").read_text())
            value["artifacts"].append({"path": bad, "variant": "detail", "content_type": "image/webp", "byte_size": len(PAYLOAD)})
            (fixture.current / "manifest.json").write_text(json.dumps(value))
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            state = json.loads(fixture.probe().stdout)
            self.assertEqual(state["active_probe"], {"version": 1, "kind": "media", "path": good})
            (fixture.current / bad).write_bytes(b"corrupt")
            self.assertNotEqual(fixture.probe().returncode, 0)
            (fixture.current / bad).unlink()
            self.assertNotEqual(fixture.probe().returncode, 0)

    def test_probe_rejects_malformed_marker_and_missing_retained_history(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            relative = media_path("active")
            fixture.add_release_media(fixture.current, relative)
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            marker = fixture.static / ".media-tombstones/invalid"
            marker.parent.mkdir(parents=True)
            marker.touch()
            self.assertNotEqual(fixture.probe().returncode, 0)
            marker.unlink()
            retained = fixture.static / "releases/retained"
            retained.mkdir()
            prior = media_path("prior")
            fixture.add_release_media(retained, prior)
            history_path = fixture.static / ".media-commit-history/history.json"
            history = json.loads(history_path.read_text())
            history["releases"]["retained"] = {
                "manifest_sha256": hashlib.sha256((retained / "manifest.json").read_bytes()).hexdigest(),
                "media": {prior: hashlib.sha256(PAYLOAD).hexdigest()},
                "first_committed_at": int(time.time()), "last_active_end": int(time.time()),
            }
            history_path.write_text(json.dumps(history))
            (fixture.static / prior).unlink()
            self.assertNotEqual(fixture.audit("--report-only").returncode, 0)

    def test_journaled_rollback_to_all_marked_release_selects_document(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = MediaAuditFixture(Path(temporary))
            old, new = media_path("old"), media_path("new")
            fixture.add_release_media(fixture.current, old)
            self.assertEqual(fixture.audit("--initial-cutover", "--old-route-current-only").returncode, 0)
            fixture.promote("release-b", new)
            self.assertEqual(fixture.audit("--frozen-exit-proved").returncode, 0)
            for relative in (old, new):
                marker = fixture.static / ".media-tombstones" / relative
                marker.parent.mkdir(parents=True, exist_ok=True)
                marker.touch()
            self.assertEqual(fixture.audit().returncode, 0)
            rollback = subprocess.run(
                [sys.executable, str(REPO / "deploy/ansible/roles/autographs_deploy/files/static_release_rollback.py"),
                 "--static-root", str(fixture.static), "--target-release-id", "active", "--frozen-exit-proved"],
                capture_output=True, text=True, check=False)
            self.assertEqual(rollback.returncode, 0, rollback.stderr)
            self.assertEqual(fixture.audit().returncode, 0)
            state = json.loads(fixture.probe().stdout)
            self.assertEqual(state["current_target"], "releases/active")
            self.assertEqual(state["active_probe"], {"version": 1, "kind": "document", "path": "manifest.json"})

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
            self.assertNotEqual(fixture.audit("--report-only", "--frozen-exit-proved").returncode, 0)
            self.assertTrue((fixture.static / ".media-commit-history/intent.json").exists())
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
