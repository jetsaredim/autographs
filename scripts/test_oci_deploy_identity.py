import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from urllib.error import HTTPError

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
        self.config = {"slot": "b", "fingerprint": self.fingerprint}
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
            "GITHUB_REPOSITORY": "owner/repo",
            "GITHUB_SHA": "a" * 40,
            "GITHUB_SERVER_URL": "https://github.com",
            "GITHUB_RUN_ID": "123",
        }, clear=True)
        self.settings.start()
        self.addCleanup(self.settings.stop)

    def deployment(self, number, **overrides):
        return {
            "id": number,
            "task": identity.SELECTION_TASK,
            "environment": identity.SELECTION_ENVIRONMENT,
            "creator": {"login": identity.BOT},
            "payload": self.config,
            **overrides,
        }

    def test_legacy_and_selected_bundle_are_independent(self):
        self.assertEqual(identity.credentials({"slot": "legacy"})[0], "ocid1.user.oc1..legacy")
        self.assertEqual(identity.credentials(self.config)[0], "ocid1.user.oc1..candidate")
        for bundle in ("", "{}", json.dumps({"fingerprint": "bad", "private_key_pem": self.pem})):
            with self.subTest(bundle_configured=bool(bundle)), patch.dict(os.environ, {"OCI_DEPLOY_KEY_B": bundle}):
                with self.assertRaises(ValueError):
                    identity.credentials(self.config)

    def test_bundle_must_match_pem_even_if_selection_and_bundle_agree(self):
        other = "00:" * 15 + "00"
        with patch.dict(os.environ, {
            "OCI_DEPLOY_KEY_B": json.dumps({"fingerprint": other, "private_key_pem": self.pem}),
        }), self.assertRaisesRegex(ValueError, "fingerprint"):
            identity.credentials({**self.config, "fingerprint": other})

    def test_loader_does_not_export_private_key_material(self):
        with tempfile.TemporaryDirectory() as tmp, patch.dict(os.environ, {
            "RUNNER_TEMP": tmp, "GITHUB_ENV": f"{tmp}/env",
        }):
            old_umask = os.umask(0o077)
            try:
                identity.load(self.config)
            finally:
                os.umask(old_umask)
            pem = Path(tmp, "oci", "api_key.pem")
            self.assertEqual(pem.stat().st_mode & 0o777, 0o600)
            self.assertEqual(pem.read_text(), self.pem)
            exported = Path(tmp, "env").read_text()
            self.assertIn(f"TF_VAR_private_key_path={pem}", exported)
            self.assertNotIn("PRIVATE KEY-----", exported)

    def test_reads_latest_successful_activation_and_skips_incomplete_records(self):
        outgoing = {**self.config, "slot": "a"}
        success = [{"state": "success", "creator": {"login": identity.BOT}}]
        with patch.object(identity, "signer_deployments", return_value=[
            self.deployment(1, payload=outgoing),
            self.deployment(4, creator={"login": "someone"}),
            self.deployment(2, payload=json.dumps(self.config)),
            self.deployment(3),
        ]), patch.object(identity, "api", side_effect=[[], success]) as api:
            self.assertEqual(identity.active_selector(), self.config)
            self.assertEqual([call.args[0] for call in api.call_args_list], [
                "deployments/3/statuses?per_page=1", "deployments/2/statuses?per_page=1",
            ])

    def test_bootstraps_with_legacy_but_surfaces_github_errors(self):
        with patch.object(identity, "api", return_value=[]):
            self.assertEqual(identity.active_selector(), {"slot": "legacy"})
        with patch.object(identity, "api", side_effect=HTTPError("", 403, "", None, None)):
            with self.assertRaises(HTTPError):
                identity.active_selector()

    def test_activation_records_public_selector_and_noops_when_already_selected(self):
        with patch.object(identity, "active_selector", return_value={"slot": "legacy"}), patch.object(
            identity, "api", side_effect=[{"id": 42}, {}],
        ) as api:
            identity.record_activation(self.config)
            deployment, status = api.call_args_list
            self.assertEqual(deployment.args[0], "deployments")
            self.assertEqual(deployment.args[1]["payload"], self.config)
            self.assertFalse(deployment.args[1]["auto_merge"])
            self.assertEqual(status.args[0], "deployments/42/statuses")
            self.assertEqual(status.args[1]["state"], "success")
            self.assertFalse(status.args[1]["auto_inactive"])
        with patch.object(identity, "active_selector", return_value=self.config), patch.object(
            identity, "api",
        ) as api:
            identity.record_activation(self.config)
            api.assert_not_called()

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
                index = steps.index("Load OCI deploy signing key")
                self.assertLess(index, steps.index(consumer))
                self.assertTrue(job["steps"][index]["run"].endswith("oci_deploy_identity.py load"))

    def test_manual_workflow_verifies_current_state_before_optional_activation(self):
        workflow = yaml.load(
            Path(".github/workflows/oci-signing-key-rotation.yml").read_text(), Loader=yaml.BaseLoader,
        )
        self.assertEqual(set(workflow["on"]), {"workflow_dispatch"})
        self.assertEqual(workflow["on"]["workflow_dispatch"]["inputs"]["activate"]["default"], "false")
        job = workflow["jobs"]["candidate"]
        self.assertIn("github.actor == github.repository_owner", job["if"])
        self.assertIn("github.ref == 'refs/heads/main'", job["if"])
        steps = job["steps"]
        self.assertEqual(steps[0]["with"]["ref"], "${{ github.sha }}")
        self.assertEqual(steps[0]["with"]["persist-credentials"], "false")
        proof = steps[-2]["run"]
        self.assertIn("envs/prod/terraform.tfstate", proof)
        self.assertIn("terraform state list", proof)
        self.assertIn("terraform plan -refresh-only", proof)
        self.assertIn("secrets secret-bundle get", proof)
        self.assertNotIn("terraform apply", proof)
        self.assertEqual(steps[-1]["if"], "inputs.activate")
        self.assertTrue(steps[-1]["run"].endswith("oci_deploy_identity.py activate"))


if __name__ == "__main__":
    unittest.main()
