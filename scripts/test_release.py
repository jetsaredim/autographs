#!/usr/bin/env python3

import importlib.util
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("release.py")
spec = importlib.util.spec_from_file_location("release", SCRIPT_PATH)
release = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(release)


def git(repo: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def commit_file(repo: Path, path: str, content: str, message: str) -> str:
    target = repo / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")
    git(repo, "add", path)
    git(repo, "commit", "-m", message)
    return git(repo, "rev-parse", "HEAD")


def make_repo() -> Path:
    root = Path(tempfile.mkdtemp())
    git(root, "init", "-b", "main")
    git(root, "config", "user.name", "Release Tests")
    git(root, "config", "user.email", "release-tests@example.invalid")
    (root / "controller/src").mkdir(parents=True)
    (root / "controller/src/contracts.rs").write_text(
        "pub const PUBLIC_SCHEMA_VERSION: u32 = 2;\n", encoding="utf-8"
    )
    git(root, "add", "controller/src/contracts.rs")
    git(root, "commit", "-m", "feat(controller): initial release")
    git(root, "tag", "v1.2.2")
    return root


def base_status() -> dict:
    return {
        "latestRepositoryVersion": "v1.2.2",
        "deployedRepositoryVersion": "v1.2.2",
        "latestDeployImpactVersion": "v1.2.2",
        "deployedControllerVersion": "v1.2.2",
        "deployedControllerDigest": "sha256:" + "1" * 64,
        "previousControllerVersion": "v1.2.1",
        "previousControllerDigest": "sha256:" + "0" * 64,
        "lastDeployImpact": "controller-image",
        "sourceRevision": "a" * 40,
        "updatedAt": "2026-01-01T00:00:00Z",
    }


def manifest(
    *,
    version: str = "v1.2.3",
    source: str = "b" * 40,
    impact: str = "controller-image",
    controller_tag: str = "v1.2.3",
    controller_digest: str = "sha256:" + "2" * 64,
    reused: bool = False,
) -> dict:
    return {
        "schemaVersion": 1,
        "repositoryVersion": version,
        "previousRelease": "v1.2.2",
        "sourceRevision": source,
        "impact": impact,
        "controller": {
            "tag": controller_tag,
            "digest": controller_digest,
            "reused": reused,
        },
        "controllerChanged": impact == "controller-image",
        "terraformChanged": impact == "runtime-config",
        "ansibleOrDeployChanged": False,
        "publicSchemaVersion": 2,
        "migrationNotes": [],
        "operatorWarnings": [],
    }


class ReleaseRangeTests(unittest.TestCase):
    def test_classifies_complete_range_when_controller_change_precedes_infra(self):
        repo = make_repo()
        commit_file(
            repo,
            "controller/src/main.rs",
            "fn main() {}\n",
            "feat(controller): add endpoint\n\nAutographs-Migration-Note: republish static output",
        )
        target_sha = commit_file(
            repo,
            "infra/terraform/main.tf",
            "terraform {}\n",
            "fix(terraform): adjust runtime\n\nAutographs-Release-Warning: apply IAM first",
        )
        git(repo, "tag", "v1.3.0")

        release.validate_target_tag(repo, "v1.3.0", target_sha)
        previous = release.find_previous_release(repo, "v1.3.0")
        impact = release.classify_release_range(repo, previous, "v1.3.0")
        trailers = release.collect_release_trailers(repo, previous, "v1.3.0")

        self.assertEqual(previous, "v1.2.2")
        self.assertEqual(impact["impact"], "controller-image")
        self.assertTrue(impact["controllerChanged"])
        self.assertTrue(impact["terraformChanged"])
        self.assertEqual(trailers["migrationNotes"], ["republish static output"])
        self.assertEqual(trailers["operatorWarnings"], ["apply IAM first"])

    def test_infrastructure_only_range_reuses_active_controller(self):
        repo = make_repo()
        target_sha = commit_file(
            repo,
            "deploy/ansible/site.yml",
            "---\n- hosts: all\n",
            "fix(deploy): reconcile runtime",
        )
        git(repo, "tag", "v1.2.3")
        status = base_status()

        plan = release.plan_release(
            repo,
            "v1.2.3",
            target_sha,
            status,
            status["deployedControllerDigest"],
        )

        self.assertEqual(plan["impact"], "runtime-config")
        self.assertEqual(plan["controllerTag"], "v1.2.2")
        self.assertEqual(plan["controllerDigest"], status["deployedControllerDigest"])
        self.assertTrue(plan["controllerReused"])

    def test_repository_only_range_requires_no_production_mutation(self):
        repo = make_repo()
        target_sha = commit_file(repo, "docs/runbook.md", "notes\n", "docs: clarify release")
        git(repo, "tag", "v1.2.3")

        plan = release.plan_release(
            repo,
            "v1.2.3",
            target_sha,
            base_status(),
            "sha256:" + "1" * 64,
        )

        self.assertEqual(plan["impact"], "repo-only")
        self.assertFalse(plan["productionMutationRequired"])
        self.assertTrue(plan["controllerReused"])

    def test_target_tag_must_be_semantic_and_match_source(self):
        repo = make_repo()
        target_sha = commit_file(repo, "README.md", "next\n", "fix: next")
        git(repo, "tag", "v1.2.3")

        with self.assertRaisesRegex(release.ReleaseError, "semantic"):
            release.validate_target_tag(repo, "release-1.2.3", target_sha)
        with self.assertRaisesRegex(release.ReleaseError, "does not resolve"):
            release.validate_target_tag(repo, "v1.2.3", git(repo, "rev-parse", "v1.2.2"))


class DraftAndManifestTests(unittest.TestCase):
    def test_preflight_blocks_unresolved_semantic_draft_and_names_tag(self):
        result = release.draft_preflight(
            [
                {"tag_name": "notes", "draft": True},
                {"tag_name": "v1.2.3", "draft": True},
                {"tag_name": "v1.2.2", "draft": False},
            ]
        )

        self.assertEqual(result, {"blocked": True, "tags": ["v1.2.3"]})

    def test_manifest_is_deterministic_and_contains_required_contract(self):
        repo = make_repo()
        target_sha = commit_file(repo, "controller/src/main.rs", "fn main() {}\n", "feat: runtime")
        git(repo, "tag", "v1.3.0")
        plan = release.plan_release(
            repo,
            "v1.3.0",
            target_sha,
            base_status(),
            "sha256:" + "a" * 64,
        )

        first = release.build_release_manifest(repo, plan)
        second = release.build_release_manifest(repo, dict(reversed(list(plan.items()))))
        parsed = json.loads(first)

        self.assertEqual(first, second)
        self.assertTrue(first.endswith(b"\n"))
        self.assertEqual(parsed["repositoryVersion"], "v1.3.0")
        self.assertEqual(parsed["sourceRevision"], target_sha)
        self.assertEqual(parsed["controller"], {
            "digest": "sha256:" + "a" * 64,
            "reused": False,
            "tag": "v1.3.0",
        })
        self.assertEqual(parsed["publicSchemaVersion"], 2)

    def test_asset_reconciliation_never_clobbers_different_bytes(self):
        generated = b'{"schemaVersion":1}\n'
        self.assertEqual(release.reconcile_manifest_asset(None, generated), "create")
        self.assertEqual(release.reconcile_manifest_asset(generated, generated), "same")
        with self.assertRaisesRegex(release.ReleaseError, "conflict"):
            release.reconcile_manifest_asset(b'{"schemaVersion":2}\n', generated)

    def test_digest_validation_and_equality_fail_closed(self):
        digest = "sha256:" + "a" * 64
        release.assert_digest_matches(digest, digest)
        with self.assertRaisesRegex(release.ReleaseError, "sha256"):
            release.assert_digest_matches("latest", "latest")
        with self.assertRaisesRegex(release.ReleaseError, "does not match"):
            release.assert_digest_matches(digest, "sha256:" + "b" * 64)


class StatusTransitionTests(unittest.TestCase):
    def test_automatic_transition_records_repository_and_controller_history(self):
        status = base_status()
        updated = release.apply_deployment_status(
            status,
            manifest(),
            "automatic",
            "2026-02-01T00:00:00Z",
        )

        self.assertEqual(updated["latestRepositoryVersion"], "v1.2.3")
        self.assertEqual(updated["deployedRepositoryVersion"], "v1.2.3")
        self.assertEqual(updated["latestDeployImpactVersion"], "v1.2.3")
        self.assertEqual(updated["deployedControllerVersion"], "v1.2.3")
        self.assertEqual(updated["previousControllerVersion"], "v1.2.2")
        self.assertEqual(updated["sourceRevision"], "b" * 40)

    def test_retry_transition_is_idempotent_after_partial_failure(self):
        first = release.apply_deployment_status(
            base_status(), manifest(), "retry", "2026-02-01T00:00:00Z"
        )
        second = release.apply_deployment_status(
            first, manifest(), "retry", "2026-02-02T00:00:00Z"
        )

        self.assertEqual(second, first)

    def test_repo_only_transition_does_not_claim_production_mutation(self):
        status = base_status()
        updated = release.apply_deployment_status(
            status,
            manifest(
                impact="repo-only",
                controller_tag="v1.2.2",
                controller_digest=status["deployedControllerDigest"],
                reused=True,
            ),
            "automatic",
            "2026-02-01T00:00:00Z",
        )

        self.assertEqual(updated["latestRepositoryVersion"], "v1.2.3")
        self.assertEqual(updated["deployedRepositoryVersion"], "v1.2.2")
        self.assertEqual(updated["latestDeployImpactVersion"], "v1.2.2")
        self.assertEqual(updated["sourceRevision"], "a" * 40)

    def test_controller_rollback_changes_only_controller_mapping_and_timestamp(self):
        status = release.apply_deployment_status(
            base_status(), manifest(), "automatic", "2026-02-01T00:00:00Z"
        )
        preserved = {
            key: status[key]
            for key in (
                "latestRepositoryVersion",
                "deployedRepositoryVersion",
                "latestDeployImpactVersion",
                "lastDeployImpact",
                "sourceRevision",
            )
        }

        rolled_back = release.apply_controller_rollback(
            status,
            "v1.2.2",
            "sha256:" + "1" * 64,
            "2026-02-03T00:00:00Z",
        )

        self.assertEqual({key: rolled_back[key] for key in preserved}, preserved)
        self.assertEqual(rolled_back["deployedControllerVersion"], "v1.2.2")
        self.assertEqual(rolled_back["previousControllerVersion"], "v1.2.3")
        self.assertEqual(rolled_back["deployedControllerDigest"], "sha256:" + "1" * 64)
        self.assertEqual(rolled_back["previousControllerDigest"], "sha256:" + "2" * 64)


if __name__ == "__main__":
    unittest.main()
