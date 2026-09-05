#!/usr/bin/env python3

"""Structural regression tests for the privileged production release graph."""

import json
import unittest
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_PATH = ROOT / ".github/workflows/deploy.yml"
FIXTURE_PATH = ROOT / "scripts/test-fixtures/release-workflow/cases.json"


def load_workflow() -> dict:
    return yaml.load(WORKFLOW_PATH.read_text(encoding="utf-8"), Loader=yaml.BaseLoader)


def named_steps(job: dict) -> list[dict]:
    return [step for step in job.get("steps", []) if isinstance(step, dict) and step.get("name")]


class ReleaseWorkflowContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.workflow = load_workflow()
        cls.cases = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        cls.release_job = cls.workflow["jobs"]["release"]
        cls.production_job = cls.workflow["jobs"]["production"]
        cls.release_steps = {step["name"]: step for step in named_steps(cls.release_job)}
        cls.production_steps = {
            step["name"]: step for step in named_steps(cls.production_job)
        }
        cls.production_step_names = [
            step["name"] for step in named_steps(cls.production_job)
        ]

    def assert_ordered(self, expected: list[str]):
        indexes = [self.production_step_names.index(name) for name in expected]
        self.assertEqual(indexes, sorted(indexes), expected)

    def test_normal_push_gating(self):
        case = self.cases["normal_push_gating"]
        release_names = [step["name"] for step in named_steps(self.release_job)]
        indexes = [release_names.index(name) for name in case["release_steps"]]
        self.assertEqual(indexes, sorted(indexes))
        condition = self.production_job.get("if", "")
        for term in case["production_condition_terms"]:
            self.assertIn(term, condition)
        self.assertNotIn("release:", WORKFLOW_PATH.read_text(encoding="utf-8"))

    def test_automatic_release(self):
        self.assert_ordered(self.cases["automatic_release"]["ordered_steps"])
        action = self.release_steps["Run release-please"]
        self.assertEqual(
            action["uses"],
            "googleapis/release-please-action@45996ed1f6d02564a971a2fa1b5860e934307cf7",
        )
        self.assertEqual(action["with"]["token"], "${{ secrets.RELEASE_PLEASE_TOKEN }}")
        self.assertNotIn("continue-on-error", action)

    def test_exact_automatic_retry_tag_checkout(self):
        case = self.cases["exact_automatic_retry_tag_checkout"]
        step = self.production_steps[case["step"]]
        for key, value in case["required_with"].items():
            self.assertEqual(step["with"][key], value)
        self.assertIn("steps.request.outputs.operation != 'rollback'", step["if"])

    def test_unresolved_draft_block(self):
        case = self.cases["unresolved_draft_block"]
        run = self.release_steps[case["step"]]["run"]
        for term in case["required_run_terms"]:
            self.assertIn(term, run)

    def test_retry_draft_lookup(self):
        case = self.cases["retry_draft_lookup"]
        run = self.production_steps[case["step"]]["run"]
        for term in case["required_run_terms"]:
            self.assertIn(term, run)

    def test_published_manifest_rollback(self):
        self.assert_ordered(self.cases["published_manifest_rollback"]["ordered_steps"])
        load = self.production_steps["Load published rollback manifest"]
        self.assertIn("steps.request.outputs.operation == 'rollback'", load["if"])
        self.assertIn("gh release download", load["run"])

    def test_identical_versus_conflicting_manifest_assets(self):
        case = self.cases["identical_versus_conflicting_manifest_assets"]
        run = self.production_steps[case["step"]]["run"]
        for term in case["required_run_terms"]:
            self.assertIn(term, run)
        for term in case["forbidden_run_terms"]:
            self.assertNotIn(term, run)

    def test_current_main_no_terraform_controller_rollback(self):
        case = self.cases["current_main_no_terraform_controller_rollback"]
        checkout = self.production_steps[case["checkout_step"]]
        self.assertEqual(checkout["with"]["ref"], "main")
        rollback = self.production_steps[case["playbook_step"]]
        self.assertEqual(rollback["with"]["playbook"], case["playbook"])
        self.assertIn("steps.request.outputs.operation == 'rollback'", rollback["if"])
        terraform = self.production_steps[case["forbidden_step"]]
        self.assertIn("steps.request.outputs.operation != 'rollback'", terraform["if"])
        status = self.production_steps["Commit production release status"]["run"]
        self.assertIn("rollback-status", status)
        self.assertNotIn("deployedRepositoryVersion", status)
        self.assertNotIn("sourceRevision", status)

    def test_immediate_pre_mutation_digest_verification(self):
        for digest_step, mutation_step in self.cases[
            "immediate_pre_mutation_digest_verification"
        ]["pairs"]:
            digest_index = self.production_step_names.index(digest_step)
            mutation_index = self.production_step_names.index(mutation_step)
            self.assertEqual(mutation_index, digest_index + 1)
            self.assertIn(
                "scripts/release.py assert-digest",
                self.production_steps[digest_step]["run"],
            )


if __name__ == "__main__":
    unittest.main()
