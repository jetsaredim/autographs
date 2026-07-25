#!/usr/bin/env python3

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace


SCRIPT_PATH = Path(__file__).with_name("release-version.py")
spec = importlib.util.spec_from_file_location("release_version", SCRIPT_PATH)
release_version = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(release_version)


class ReleaseVersionTests(unittest.TestCase):
    def test_next_version_resets_downstream_numbers(self):
        self.assertEqual(release_version.next_version("v1.4.7", "patch"), "v1.4.8")
        self.assertEqual(release_version.next_version("v1.4.7", "minor"), "v1.5.0")
        self.assertEqual(release_version.next_version("v1.4.7", "major"), "v2.0.0")

    def test_classify_major_from_label_or_breaking_change(self):
        self.assertEqual(
            release_version.classify_bump({"title": "fix: route", "body": "", "labels": ["version:major"]}),
            "major",
        )
        self.assertEqual(
            release_version.classify_bump(
                {"title": "refactor runtime", "body": "BREAKING CHANGE: static schema v3", "labels": []}
            ),
            "major",
        )

    def test_classify_minor_from_label_or_feat_title(self):
        self.assertEqual(
            release_version.classify_bump({"title": "fix: route", "body": "", "labels": ["version:minor"]}),
            "minor",
        )
        self.assertEqual(
            release_version.classify_bump({"title": "feat(admin): suggest taxonomy", "body": "", "labels": []}),
            "minor",
        )

    def test_classify_defaults_to_patch(self):
        self.assertEqual(
            release_version.classify_bump({"title": "docs: clarify deploy", "body": "", "labels": []}),
            "patch",
        )

    def test_prepare_updates_status_and_readme_for_repo_only_patch(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            status = root / ".release-status.json"
            readme = root / "README.md"
            status.write_text(
                json.dumps(
                    {
                        "repoVersion": "v0.7.2",
                        "deployedControllerVersion": "v0.7.0",
                        "lastBump": "minor",
                        "lastDeployImpact": "controller-image",
                        "sourceRevision": "abc123",
                        "updatedAt": "2026-01-01T00:00:00Z",
                    }
                ),
                encoding="utf-8",
            )
            readme.write_text(
                "# Autographs\n\n![Renovate configured](https://img.shields.io/badge/Renovate-configured-1f8b4c)\n",
                encoding="utf-8",
            )
            pr = root / "pr.json"
            pr.write_text(json.dumps({"title": "docs: update README", "body": "", "labels": []}), encoding="utf-8")

            release_version.prepare_release(
                SimpleNamespace(
                    status_file=status,
                    readme=readme,
                    pr_json=pr,
                    controller_image_impact="false",
                    runtime_deploy_impact="false",
                    source_revision="def456",
                    github_output=root / "github-output.txt",
                )
            )

            updated = json.loads(status.read_text(encoding="utf-8"))
            self.assertEqual(updated["repoVersion"], "v0.7.3")
            self.assertEqual(updated["deployedControllerVersion"], "v0.7.0")
            self.assertEqual(updated["lastBump"], "patch")
            self.assertEqual(updated["lastDeployImpact"], "repo-only")
            readme_text = readme.read_text(encoding="utf-8")
            self.assertIn("Repo version: `v0.7.3`", readme_text)
            self.assertIn("Deployed controller image: `v0.7.0`", readme_text)

    def test_prepare_updates_deployed_controller_for_image_impact(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            status = root / ".release-status.json"
            readme = root / "README.md"
            status.write_text(
                json.dumps(
                    {
                        "repoVersion": "v0.7.2",
                        "deployedControllerVersion": "v0.7.0",
                        "lastBump": "patch",
                        "lastDeployImpact": "repo-only",
                        "sourceRevision": "abc123",
                        "updatedAt": "2026-01-01T00:00:00Z",
                    }
                ),
                encoding="utf-8",
            )
            readme.write_text(
                "# Autographs\n\n![Renovate configured](https://img.shields.io/badge/Renovate-configured-1f8b4c)\n",
                encoding="utf-8",
            )
            pr = root / "pr.json"
            pr.write_text(json.dumps({"title": "feat: add release metadata", "body": "", "labels": []}), encoding="utf-8")

            release_version.prepare_release(
                SimpleNamespace(
                    status_file=status,
                    readme=readme,
                    pr_json=pr,
                    controller_image_impact="true",
                    runtime_deploy_impact="false",
                    source_revision="def456",
                    github_output=root / "github-output.txt",
                )
            )

            updated = json.loads(status.read_text(encoding="utf-8"))
            self.assertEqual(updated["repoVersion"], "v0.8.0")
            self.assertEqual(updated["deployedControllerVersion"], "v0.8.0")
            self.assertEqual(updated["lastBump"], "minor")
            self.assertEqual(updated["lastDeployImpact"], "controller-image")


if __name__ == "__main__":
    unittest.main()
