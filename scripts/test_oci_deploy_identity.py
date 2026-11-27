import base64
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import yaml

from scripts import oci_deploy_identity as identity


class DeployIdentityTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.pem = subprocess.check_output(
            ["openssl", "genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:2048"],
            stderr=subprocess.DEVNULL,
            text=True,
        )
        cls.fingerprint = identity.key_fingerprint(cls.pem)

    def setUp(self):
        self.config = {"slot": "b", "fingerprint": self.fingerprint, "approval_pr": 21}
        self.head = "a" * 40
        self.pr = {
            "state": "open",
            "merged_at": "2026-11-27T12:00:00Z",
            "base": {"ref": "main"},
            "head": {"sha": self.head, "repo": {"full_name": "owner/repo"}},
            "user": {"login": "owner"},
        }
        self.event = {
            "id": 18, "label": {"name": identity.LABEL}, "event": "labeled",
            "actor": {"login": "owner"}, "created_at": "2026-11-27T11:55:00Z",
            "commit_id": None,
        }
        self.status = {
            "context": identity.STATUS,
            "created_at": "2026-11-27T11:59:00Z",
            "state": "success",
            "creator": {"login": "github-actions[bot]"},
            "description": identity.description(self.config, self.event["id"]),
        }
        self.settings = patch.dict(os.environ, {
            "OCI_DEPLOY_USER_OCID": "ocid1.user.oc1..candidate",
            "OCI_CLI_USER_OCID": "ocid1.user.oc1..legacy",
            "OCI_TENANCY_OCID": "ocid1.tenancy.oc1..example",
            "OCI_PRIVATE_KEY_PEM": self.pem,
            "OCI_FINGERPRINT": self.fingerprint,
            "OCI_DEPLOY_KEY_B": json.dumps({
                "fingerprint": self.fingerprint,
                "private_key_pem": self.pem,
            }),
            "REPOSITORY_OWNER": "owner",
            "GITHUB_REPOSITORY": "owner/repo",
            "GITHUB_RUN_ATTEMPT": "1",
        }, clear=True)
        self.settings.start()
        self.addCleanup(self.settings.stop)

    def api(self, path):
        if path == "pulls/21":
            return self.pr
        if path == f"contents/{identity.SELECTOR}?ref={self.head}":
            return {"content": base64.b64encode(json.dumps(self.config).encode()).decode()}
        self.fail(f"Unexpected GitHub API path: {path}")

    def pages(self, path):
        resources = {
            "issues/21/events": [self.event],
            f"commits/{self.head}/statuses": [self.status],
            "pulls/21/files": [{"filename": identity.SELECTOR, "status": "modified"}],
        }
        return iter(resources[path])

    def test_legacy_and_selected_bundle_are_independent(self):
        self.assertEqual(identity.credentials({"slot": "legacy"})[0], "ocid1.user.oc1..legacy")
        self.assertEqual(identity.credentials(self.config)[0], "ocid1.user.oc1..candidate")
        for bundle in ("", "{}", json.dumps({"fingerprint": "bad", "private_key_pem": self.pem})):
            with self.subTest(bundle_configured=bool(bundle)), patch.dict(os.environ, {"OCI_DEPLOY_KEY_B": bundle}):
                with self.assertRaises(ValueError):
                    identity.credentials(self.config)

    def test_bundle_must_match_pem_even_if_config_and_bundle_agree(self):
        other = "00:" * 15 + "00"
        with patch.dict(os.environ, {
            "OCI_DEPLOY_KEY_B": json.dumps({"fingerprint": other, "private_key_pem": self.pem}),
        }), self.assertRaisesRegex(ValueError, "fingerprint"):
            identity.credentials({**self.config, "fingerprint": other})

    def test_loader_does_not_export_private_key_material(self):
        with tempfile.TemporaryDirectory() as tmp, patch.dict(os.environ, {
            "RUNNER_TEMP": tmp, "GITHUB_ENV": f"{tmp}/env",
        }), patch.object(identity, "require_merged_approval") as verify:
            old_umask = os.umask(0o077)
            try:
                identity.load(self.config)
            finally:
                os.umask(old_umask)
            verify.assert_called_once_with(self.config)
            pem = Path(tmp, "oci", "api_key.pem")
            self.assertEqual(pem.stat().st_mode & 0o777, 0o600)
            self.assertEqual(pem.read_text(), self.pem)
            exported = Path(tmp, "env").read_text()
            self.assertIn(f"TF_VAR_private_key_path={pem}", exported)
            self.assertNotIn("PRIVATE KEY-----", exported)

    def test_merge_requires_the_same_head_slot_fingerprint_and_label(self):
        with patch.object(identity, "api", side_effect=self.api), patch.object(
            identity, "pages", side_effect=self.pages,
        ):
            identity.require_merged_approval(self.config)
            with self.assertRaisesRegex(ValueError, "reviewed PR head"):
                identity.require_merged_approval({**self.config, "slot": "a"})
            for override in (
                {"state": "failure"},
                {"description": identity.description(self.config, 9)},
                {"created_at": "2026-11-27T12:01:00Z"},
                {"creator": {"login": "owner"}},
            ):
                with self.subTest(override=override), patch.dict(self.status, override):
                    with self.assertRaisesRegex(ValueError, "successful signing-key check"):
                        identity.require_merged_approval(self.config)
            with patch.dict(self.event, {"event": "unlabeled"}):
                with self.assertRaisesRegex(ValueError, "owner approval"):
                    identity.require_merged_approval(self.config)

    def test_candidate_success_requires_owner_label_on_current_head(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(
            identity, "api", side_effect=self.api,
        ), patch.object(identity, "pages", side_effect=self.pages), patch.object(
            identity, "set_status",
        ) as status, patch.dict(os.environ, {"GITHUB_EVENT_PATH": f"{tmp}/event"}):
            metadata, candidate = Path(tmp, "metadata"), Path(tmp, "candidate")
            payload = {
                "number": 21, "pull_request": self.pr,
                "action": "labeled", "label": {"name": identity.LABEL},
                "sender": {"login": "owner"},
            }
            event_file = Path(os.environ["GITHUB_EVENT_PATH"])
            event_file.write_text(json.dumps(payload))
            identity.prepare(metadata, candidate)
            identity.report(metadata, candidate, passed=True)
            status.assert_called_with(self.head, "success", self.status["description"])

            for overrides in ({"action": "synchronize"}, {"sender": {"login": "someone"}}):
                with self.subTest(overrides=overrides):
                    event_file.write_text(json.dumps({**payload, **overrides}))
                    identity.prepare(metadata, candidate)
                    with self.assertRaisesRegex(ValueError, "owner must apply"):
                        identity.report(metadata, candidate, passed=True)
                    self.assertEqual(status.call_args.args[1], "failure")

            with patch.dict(self.pr["head"], {"sha": "b" * 40}):
                status.reset_mock()
                identity.report(metadata, candidate, passed=True)
                status.assert_not_called()

    def test_regional_workflows_load_the_selected_signer_before_using_oci(self):
        workflows = Path(".github/workflows")
        consumers = {
            "ci": ("terraform-checks", "Terraform plan"),
            "deploy": ("production", "Terraform apply"),
            "apply-security-updates": ("update", "Resolve runtime VM IP"),
            "reboot-security-runtime": ("reboot", "Resolve runtime VM IP"),
            "weekly-security-scan": ("scan", "Resolve runtime VM IP"),
            "image-cleanup": ("resolve-runtime", "Resolve runtime VM IP"),
            "runtime-secret-rotation": ("rotation", "Resolve runtime coordinates from Terraform"),
        }
        for workflow, (job_name, consumer) in consumers.items():
            with self.subTest(workflow=workflow):
                jobs = yaml.load(
                    (workflows / f"{workflow}.yml").read_text(), Loader=yaml.BaseLoader,
                )["jobs"]
                job = jobs[job_name]
                self.assertEqual(job["environment"], "production")
                steps = [step.get("name") for step in job["steps"]]
                self.assertLess(steps.index("Load OCI deploy signing key"), steps.index(consumer))
                if workflow == "ci":
                    loader = job["steps"][steps.index("Load OCI deploy signing key")]
                    self.assertIn("github.event.pull_request.base.sha", loader["env"]["OCI_SELECTOR_REF"])

    def test_candidate_proof_uses_trusted_code_and_existing_state(self):
        workflow = yaml.load(
            Path(".github/workflows/oci-signing-key-rotation.yml").read_text(), Loader=yaml.BaseLoader,
        )
        steps = workflow["jobs"]["candidate"]["steps"]
        self.assertEqual(steps[0]["with"]["ref"], "${{ github.sha }}")
        self.assertEqual(steps[0]["with"]["persist-credentials"], "false")
        proof = next(step["run"] for step in steps if step["name"].startswith("Prove access"))
        self.assertIn("envs/prod/terraform.tfstate", proof)
        self.assertIn("terraform state list", proof)
        self.assertIn("terraform plan -refresh-only", proof)
        self.assertIn("secrets secret-bundle get", proof)
        self.assertNotIn("terraform apply", proof)


if __name__ == "__main__":
    unittest.main()
