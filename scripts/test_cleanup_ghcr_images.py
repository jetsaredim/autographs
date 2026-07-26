#!/usr/bin/env python3

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("cleanup-ghcr-images.py")
spec = importlib.util.spec_from_file_location("cleanup_ghcr_images", SCRIPT_PATH)
cleanup_ghcr_images = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(cleanup_ghcr_images)


class CleanupGhcrImagesTests(unittest.TestCase):
    def test_semver_tags_only_match_release_versions(self):
        self.assertEqual(
            cleanup_ghcr_images.semver_tags(["v0.7.1", "latest", "v1.2", "v1.2.3-extra"]),
            ["v0.7.1"],
        )

    def test_deployed_controller_version_prefers_explicit_env(self):
        old_value = os.environ.get("GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION")
        try:
            os.environ["GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION"] = "v9.9.9"
            self.assertEqual(cleanup_ghcr_images.deployed_controller_version_from_status(), "v9.9.9")
        finally:
            if old_value is None:
                os.environ.pop("GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION", None)
            else:
                os.environ["GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION"] = old_value

    def test_deployed_controller_version_reads_release_status(self):
        old_explicit = os.environ.pop("GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION", None)
        old_path = os.environ.get("GHCR_CLEANUP_RELEASE_STATUS_PATH")
        try:
            with tempfile.TemporaryDirectory() as tmp:
                status = Path(tmp) / ".release-status.json"
                status.write_text(
                    json.dumps({"deployedControllerVersion": "v1.2.3"}),
                    encoding="utf-8",
                )
                os.environ["GHCR_CLEANUP_RELEASE_STATUS_PATH"] = str(status)
                self.assertEqual(cleanup_ghcr_images.deployed_controller_version_from_status(), "v1.2.3")
        finally:
            if old_explicit is not None:
                os.environ["GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION"] = old_explicit
            if old_path is None:
                os.environ.pop("GHCR_CLEANUP_RELEASE_STATUS_PATH", None)
            else:
                os.environ["GHCR_CLEANUP_RELEASE_STATUS_PATH"] = old_path


if __name__ == "__main__":
    unittest.main()
