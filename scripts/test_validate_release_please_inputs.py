import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


SCRIPT_PATH = Path(__file__).with_name("validate_release_please_inputs.py")
spec = importlib.util.spec_from_file_location("validate_release_please_inputs", SCRIPT_PATH)
validator = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(validator)


RELEASE_TYPES = ("feat", "fix", "perf", "revert", "docs", "chore")


class ValidateSubjectTests(unittest.TestCase):
    def test_accepts_configured_type(self):
        self.assertIsNone(validator.validate_subject("feat: add catalog", RELEASE_TYPES))

    def test_accepts_scope_and_breaking_marker(self):
        self.assertIsNone(
            validator.validate_subject("fix(admin/media)!: replace crop model", RELEASE_TYPES)
        )

    def test_rejects_unconfigured_type(self):
        error = validator.validate_subject("test: cover catalog", RELEASE_TYPES)
        self.assertIn("type 'test' is not configured", error)

    def test_rejects_missing_description(self):
        self.assertIn(
            "must match", validator.validate_subject("fix(controller):", RELEASE_TYPES)
        )

    def test_rejects_uppercase_type(self):
        self.assertIn("must match", validator.validate_subject("Fix: catalog", RELEASE_TYPES))

    def test_rejects_whitespace_in_scope(self):
        self.assertIn(
            "must match", validator.validate_subject("fix(admin media): align", RELEASE_TYPES)
        )


class ConfigurationTests(unittest.TestCase):
    def test_loads_root_changelog_section_types_in_order(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            config_path = Path(temporary_directory) / "release-please-config.json"
            config_path.write_text(
                json.dumps(
                    {
                        "packages": {
                            ".": {
                                "changelog-sections": [
                                    {"type": "feat", "section": "Features"},
                                    {"type": "fix", "section": "Fixes"},
                                    {"type": "feat", "section": "Duplicate"},
                                ]
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )
            self.assertEqual(validator.load_release_types(config_path), ("feat", "fix"))

    def test_rejects_missing_changelog_sections(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            config_path = Path(temporary_directory) / "release-please-config.json"
            config_path.write_text('{"packages": {".": {}}}', encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "cannot read release types"):
                validator.load_release_types(config_path)


class ObjectIdTests(unittest.TestCase):
    def test_accepts_sha1_and_sha256_object_ids(self):
        validator.validate_object_id("a" * 40, "base")
        validator.validate_object_id("b" * 64, "head")

    def test_rejects_revision_expressions(self):
        with self.assertRaisesRegex(ValueError, "hexadecimal Git object ID"):
            validator.validate_object_id("main..HEAD", "base")


class PullRequestCommitTests(unittest.TestCase):
    @patch.object(validator, "run_git")
    def test_returns_non_merge_subjects_and_skips_merges(self, run_git):
        run_git.side_effect = [
            "aaa parent\nbbb parent other-parent\nccc aaa\n",
            "feat: first\n",
            "fix(scope): third\n",
        ]
        commits, skipped_merges = validator.pull_request_commits(Path("."), "base", "head")
        self.assertEqual(commits, [("aaa", "feat: first"), ("ccc", "fix(scope): third")])
        self.assertEqual(skipped_merges, ["bbb"])


if __name__ == "__main__":
    unittest.main()
