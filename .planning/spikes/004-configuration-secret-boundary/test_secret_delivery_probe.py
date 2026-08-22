import importlib.util
import json
from pathlib import Path
import unittest


MODULE_PATH = Path(__file__).with_name("secret_delivery_probe.py")
SPEC = importlib.util.spec_from_file_location("secret_delivery_probe", MODULE_PATH)
probe = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(probe)


class SecretDeliveryProbeTests(unittest.TestCase):
    def setUp(self):
        self.report = probe.run_probe()
        self.models = {model["model"]: model for model in self.report["models"]}

    def test_output_is_redacted(self):
        self.assertNotIn(probe.FAKE_SECRET, json.dumps(self.report))

    def test_persistent_env_exposes_secrets_on_disk_and_in_environment(self):
        model = self.models["persistent-env"]
        self.assertEqual(model["persistent_secret_file_count"], 1)
        self.assertEqual(model["process_env_secret_count"], 3)

    def test_startup_materialization_uses_private_runtime_files(self):
        model = self.models["startup-materialization"]
        self.assertEqual(model["persistent_secret_file_count"], 0)
        self.assertEqual(model["ephemeral_secret_file_count"], 4)
        self.assertTrue(all(mode == 0o400 for mode in model["runtime_file_modes"].values()))

    def test_direct_application_avoids_app_managed_persistent_secret_files(self):
        model = self.models["direct-application"]
        self.assertEqual(model["persistent_secret_file_count"], 0)
        self.assertEqual(model["ephemeral_secret_file_count"], 1)
        self.assertEqual(model["process_env_secret_count"], 0)
        self.assertTrue(model["supports_wallet_file_requirement"])

    def test_report_discloses_unmeasured_kernel_persistence_surfaces(self):
        self.assertIn("process memory paging", self.report["excluded_surfaces"])
        self.assertIn("tmpfs paging to swap", self.report["excluded_surfaces"])
        self.assertIn("core dumps", self.report["excluded_surfaces"])


if __name__ == "__main__":
    unittest.main()
