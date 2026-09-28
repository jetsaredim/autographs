#!/usr/bin/env python3

"""Structural regression tests for the privileged production release graph."""

import hashlib
import importlib.util
import json
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_PATH = ROOT / ".github/workflows/deploy.yml"
FIXTURE_PATH = ROOT / "scripts/test-fixtures/release-workflow/cases.json"
ROLE_TASKS_PATH = ROOT / "deploy/ansible/roles/autographs_deploy/tasks/main.yml"
APP_ENV_PATH = ROOT / "deploy/ansible/roles/autographs_deploy/templates/app.env.j2"
ROLLBACK_PATH = ROOT / "deploy/ansible/playbooks/controller-rollback.yml"
DOCKER_BAKE_PATH = ROOT / ".github/docker-bake.hcl"
RECOVERY_SCRIPT_PATH = ROOT / "scripts/retry_recovery.py"
RECOVERY_CONTRACT_PATH = ROOT / "scripts/retry-recovery-contracts.json"
RELEASE_SCRIPT_PATH = ROOT / "scripts/release.py"

recovery_spec = importlib.util.spec_from_file_location(
    "retry_recovery", RECOVERY_SCRIPT_PATH
)
retry_recovery = importlib.util.module_from_spec(recovery_spec)
assert recovery_spec.loader is not None
recovery_spec.loader.exec_module(retry_recovery)

release_spec = importlib.util.spec_from_file_location("release", RELEASE_SCRIPT_PATH)
release = importlib.util.module_from_spec(release_spec)
assert release_spec.loader is not None
release_spec.loader.exec_module(release)


def git(repo: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def git_bytes(repo: Path, *args: str) -> bytes:
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        capture_output=True,
    ).stdout


def write_file(repo: Path, path: str, content: str) -> None:
    target = repo / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def make_recovery_repo(allowed_path: str, extra_paths: list[str]) -> tuple[Path, str, str]:
    repo = Path(tempfile.mkdtemp())
    git(repo, "init", "-b", "main")
    git(repo, "config", "user.name", "Recovery Tests")
    git(repo, "config", "user.email", "recovery-tests@example.invalid")
    write_file(repo, allowed_path, "broken\n")
    for path in extra_paths:
        write_file(repo, path, "release\n")
    git(repo, "add", ".")
    git(repo, "commit", "-m", "release source")
    git(repo, "tag", "v1.0.0")
    source = git(repo, "rev-parse", "HEAD")
    write_file(repo, allowed_path, "fixed\n")
    for path in extra_paths:
        write_file(repo, path, "later main\n")
    git(repo, "add", ".")
    git(repo, "commit", "-m", "recovery candidate")
    recovery = git(repo, "rev-parse", "HEAD")
    git(repo, "checkout", "--detach", source)
    return repo, source, recovery


def write_recovery_contract(
    repo: Path, recovery_revision: str, allowed_path: str
) -> Path:
    digest = hashlib.sha256(
        git_bytes(repo, "show", f"{recovery_revision}:{allowed_path}")
    ).hexdigest()
    contract = {
        "schemaVersion": 1,
        "releases": {
            "v1.0.0": {
                "recoveryRevision": recovery_revision,
                "approvedFiles": {allowed_path: f"sha256:{digest}"},
            }
        },
    }
    path = repo / "recovery-contract.json"
    path.write_text(json.dumps(contract), encoding="utf-8")
    return path


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
        self.assertNotIn("release", self.workflow["on"])

    def test_automatic_release(self):
        self.assert_ordered(self.cases["automatic_release"]["ordered_steps"])
        release_names = [step["name"] for step in named_steps(self.release_job)]
        token_name = "Generate short-lived release token"
        token = self.release_steps[token_name]
        self.assertEqual(
            token["uses"],
            "actions/create-github-app-token@bcd2ba49218906704ab6c1aa796996da409d3eb1",
        )
        self.assertEqual(
            token["with"]["client-id"],
            "${{ vars.RELEASE_PLEASE_APP_CLIENT_ID }}",
        )
        self.assertEqual(
            token["with"]["private-key"],
            "${{ secrets.RELEASE_PLEASE_APP_PRIVATE_KEY }}",
        )
        self.assertEqual(token["with"]["permission-contents"], "write")
        self.assertEqual(token["with"]["permission-pull-requests"], "write")
        self.assertNotIn("owner", token["with"])
        self.assertNotIn("repositories", token["with"])
        self.assertEqual(self.release_job["permissions"], {"contents": "read"})
        self.assertLess(
            release_names.index(token_name),
            release_names.index("Check unresolved draft releases"),
        )
        action = self.release_steps["Run release-please"]
        self.assertEqual(
            action["uses"],
            "googleapis/release-please-action@45996ed1f6d02564a971a2fa1b5860e934307cf7",
        )
        app_token = "${{ steps.release_app_token.outputs.token }}"
        self.assertEqual(action["with"]["token"], app_token)
        self.assertEqual(
            self.release_steps["Check unresolved draft releases"]["env"]["GH_TOKEN"],
            app_token,
        )
        retired_token = "RELEASE_PLEASE_" + "TOKEN"
        self.assertNotIn(retired_token, WORKFLOW_PATH.read_text(encoding="utf-8"))
        self.assertNotIn("continue-on-error", action)

    def test_release_and_production_state_machine_is_serialized(self):
        concurrency = self.workflow["concurrency"]
        self.assertEqual(concurrency["group"], "release-production")
        self.assertEqual(concurrency["cancel-in-progress"], "false")
        self.assertNotIn("concurrency", self.release_job)
        self.assertNotEqual(
            concurrency["group"], self.production_job["concurrency"]["group"]
        )

    def test_retry_uses_current_main_automation_without_relabeling_release_content(self):
        case = self.cases["retry_automation_checkout"]
        release_source = self.production_steps[case["release_source_step"]]
        self.assertEqual(release_source["with"]["fetch-depth"], "0")
        self.assertEqual(
            release_source["with"]["ref"],
            "${{ steps.request.outputs.release_tag }}",
        )
        self.assertNotIn("path", release_source["with"])
        self.assertIn(
            "steps.request.outputs.operation != 'rollback'", release_source["if"]
        )

        retry_automation = self.production_steps[case["retry_automation_step"]]
        self.assertEqual(
            retry_automation["if"], "steps.request.outputs.operation == 'retry'"
        )
        self.assertEqual(retry_automation["with"]["fetch-depth"], "0")
        self.assertEqual(retry_automation["with"]["ref"], "${{ github.sha }}")
        self.assertEqual(retry_automation["with"]["path"], case["automation_path"])
        self.assertEqual(
            retry_automation["with"]["sparse-checkout"].splitlines(),
            case["sparse_paths"],
        )
        self.assert_ordered(
            [
                case["release_source_step"],
                case["retry_automation_step"],
                "Validate selected release source",
                case["recovery_step"],
                "Terraform apply",
                "Run full deployment",
            ]
        )

        recovery = self.production_steps[case["recovery_step"]]
        self.assertEqual(recovery["id"], "retry_recovery")
        self.assertIn("scripts/retry_recovery.py", recovery["run"])
        self.assertIn("scripts/retry-recovery-contracts.json", recovery["run"])
        self.assertIn("retry-recovery-audit.json", recovery["run"])
        self.assertIn(
            "retry-recovery-attempt-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}.json",
            recovery["run"],
        )
        self.assertIn("--attempt-output", recovery["run"])
        self.assertIn("scripts/release.py reconcile-asset", recovery["run"])
        self.assertIn("gh release upload", recovery["run"])

        deploy = self.production_steps["Run full deployment"]
        self.assertEqual(deploy["with"]["directory"], "deploy/ansible")
        self.assertEqual(
            deploy["env"]["ANSIBLE_CONFIG"],
            "${{ format('{0}/deploy/ansible/ansible.cfg', github.workspace) }}",
        )
        for release_identity in (
            "steps.request.outputs.release_tag",
            "steps.manifest.outputs.controller_tag",
            "steps.manifest.outputs.controller_digest",
            "steps.source.outputs.source_revision",
        ):
            self.assertIn(release_identity, deploy["with"]["options"])

        self.assertEqual(
            self.production_steps["Build and publish semantic controller image"]["with"][
                "files"
            ].strip(),
            ".github/docker-bake.hcl",
        )
        self.assertEqual(
            self.production_steps["Terraform apply"]["working-directory"],
            "infra/terraform",
        )
        for step_name in (
            "Resolve controller image plan",
            "Build and publish semantic controller image",
            "Reconcile release manifest",
            "Terraform apply",
            "Verify deployment health",
            "Commit production release status",
            "Publish GitHub Release",
        ):
            self.assertNotIn(
                case["automation_path"], json.dumps(self.production_steps[step_name])
            )

        gate = self.production_steps["Validate release completion gate"]
        self.assertEqual(
            gate["env"]["RETRY_RECOVERY_VERIFIED"],
            "${{ steps.retry_recovery.outputs.verified }}",
        )
        self.assertIn("Retry completion requires a verified recovery audit", gate["run"])
        status = self.production_steps["Commit production release status"]["run"]
        self.assertIn("--retry-recovery-audit", status)

    def test_status_commit_uses_clean_current_main_checkout_after_recovery(self):
        checkout_name = "Checkout current main for production status"
        checkout = self.production_steps[checkout_name]
        status = self.production_steps["Commit production release status"]
        status_condition = (
            "steps.request.outputs.operation == 'rollback' || "
            "steps.release_gate.outputs.ready == 'true'"
        )

        self.assertEqual(checkout["if"], status_condition)
        self.assertEqual(checkout["uses"], "actions/checkout@v7")
        self.assertEqual(checkout["with"]["fetch-depth"], "0")
        self.assertEqual(checkout["with"]["ref"], "main")
        self.assertEqual(checkout["with"]["path"], ".production-status")
        self.assertEqual(status["if"], status_condition)
        self.assertEqual(status["working-directory"], ".production-status")
        self.assertNotIn("git restore", status["run"])
        self.assertNotIn("git reset", status["run"])
        self.assert_ordered(
            [
                "Validate release completion gate",
                checkout_name,
                "Commit production release status",
                "Publish GitHub Release",
            ]
        )

    def test_retry_recovery_two_revision_allowlist_accepts_only_approved_bytes(self):
        case = self.cases["retry_automation_checkout"]
        allowed_path = case["allowed_path"]
        repo, source, recovery = make_recovery_repo(allowed_path, [])
        contract = write_recovery_contract(repo, recovery, allowed_path)

        audit = retry_recovery.apply_recovery(
            repo,
            contract,
            "v1.0.0",
            source,
        )

        self.assertEqual((repo / allowed_path).read_text(encoding="utf-8"), "fixed\n")
        self.assertTrue(audit["recoveryApplied"])
        self.assertEqual(audit["releaseSourceRevision"], source)
        self.assertEqual(audit["recoveryRevision"], recovery)
        self.assertEqual(audit["approvedFiles"][0]["path"], allowed_path)
        self.assertNotIn("automationRevision", audit)

    def test_retry_recovery_existing_asset_survives_a_later_dispatch(self):
        case = self.cases["retry_automation_checkout"]
        allowed_path = case["allowed_path"]
        repo, source, recovery = make_recovery_repo(allowed_path, [])
        contract = write_recovery_contract(repo, recovery, allowed_path)

        audit_a = retry_recovery.apply_recovery(repo, contract, "v1.0.0", source)
        attempt_a = retry_recovery.build_attempt_record(
            audit_a, "a" * 40, "100", "1"
        )
        existing_asset = retry_recovery._encode_json(audit_a)

        git(repo, "checkout", "--force", "--detach", source)
        audit_b = retry_recovery.apply_recovery(repo, contract, "v1.0.0", source)
        attempt_b = retry_recovery.build_attempt_record(
            audit_b, "b" * 40, "101", "1"
        )
        generated_asset = retry_recovery._encode_json(audit_b)

        self.assertEqual(
            release.reconcile_manifest_asset(existing_asset, generated_asset), "same"
        )
        self.assertEqual(audit_b, audit_a)
        self.assertNotEqual(
            attempt_b["automationRevision"], attempt_a["automationRevision"]
        )
        self.assertNotEqual(attempt_b["workflowRunId"], attempt_a["workflowRunId"])
        self.assertEqual(
            attempt_b["recoveryAuditSha256"], attempt_a["recoveryAuditSha256"]
        )

        git(repo, "checkout", "--force", "main")
        write_file(repo, allowed_path, "revised recovery\n")
        git(repo, "add", allowed_path)
        git(repo, "commit", "-m", "revise recovery payload")
        revised_recovery = git(repo, "rev-parse", "HEAD")
        revised_contract = write_recovery_contract(
            repo, revised_recovery, allowed_path
        )
        git(repo, "checkout", "--force", "--detach", source)
        revised_audit = retry_recovery.apply_recovery(
            repo, revised_contract, "v1.0.0", source
        )

        with self.assertRaisesRegex(release.ReleaseError, "conflict"):
            release.reconcile_manifest_asset(
                existing_asset, retry_recovery._encode_json(revised_audit)
            )

    def test_repository_recovery_contract_pins_revision_and_file_hashes(self):
        contract = json.loads(RECOVERY_CONTRACT_PATH.read_text(encoding="utf-8"))
        recovery = contract["releases"]["v0.2.4"]
        self.assertRegex(recovery["recoveryRevision"], r"^[0-9a-f]{40}$")
        self.assertEqual(
            sorted(recovery["approvedFiles"]),
            [
                "deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml",
                "deploy/ansible/roles/autographs_deploy/tasks/validate_kernel_boot_entry.yml",
            ],
        )
        for digest in recovery["approvedFiles"].values():
            self.assertRegex(digest, r"^sha256:[0-9a-f]{64}$")

    def test_retry_recovery_rejects_each_non_allowlisted_desired_state_path(self):
        case = self.cases["retry_automation_checkout"]
        allowed_path = case["allowed_path"]
        for forbidden_path in case["forbidden_paths"]:
            with self.subTest(forbidden_path=forbidden_path):
                repo, source, recovery = make_recovery_repo(
                    allowed_path, [forbidden_path]
                )
                contract = write_recovery_contract(repo, recovery, allowed_path)

                with self.assertRaisesRegex(
                    retry_recovery.RecoveryError,
                    "unexpected=.*" + re.escape(forbidden_path),
                ):
                    retry_recovery.apply_recovery(
                        repo,
                        contract,
                        "v1.0.0",
                        source,
                    )

                self.assertEqual(
                    (repo / allowed_path).read_text(encoding="utf-8"), "broken\n"
                )

    def test_manual_production_operations_require_main(self):
        condition = self.production_job["if"]
        self.assertIn("github.event_name == 'workflow_dispatch'", condition)
        self.assertNotIn("github.ref", condition)
        run = self.production_steps["Resolve release request"]["run"]
        guard = '$GITHUB_REF" != "refs/heads/main'
        self.assertIn(guard, run)
        self.assertLess(run.index(guard), run.index('gh release view "$release_tag"'))

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

    def test_draft_manifest_asset_lookup(self):
        case = self.cases["draft_manifest_asset_lookup"]
        for step_name in case["steps"]:
            run = self.production_steps[step_name]["run"]
            for term in case["required_run_terms"]:
                self.assertIn(term, run)
            for term in case["forbidden_run_terms"]:
                self.assertNotIn(term, run)

    def test_published_manifest_rollback(self):
        self.assert_ordered(self.cases["published_manifest_rollback"]["ordered_steps"])
        load = self.production_steps["Load published rollback manifest"]
        self.assertIn("steps.request.outputs.operation == 'rollback'", load["if"])
        self.assertIn("gh release download", load["run"])

    def test_rollback_authenticates_before_private_image_inspection(self):
        release_login = self.production_steps["Log in to ghcr.io"]
        self.assertIn(
            "steps.request.outputs.operation != 'rollback'", release_login["if"]
        )
        self.assertEqual(release_login["with"]["password"], "${{ secrets.GITHUB_TOKEN }}")
        login_name = "Log in to ghcr.io for rollback inspection"
        login = self.production_steps[login_name]
        self.assertIn("steps.request.outputs.operation == 'rollback'", login["if"])
        self.assertEqual(login["uses"], "docker/login-action@v4")
        self.assertEqual(login["with"]["registry"], "ghcr.io")
        self.assertEqual(login["with"]["password"], "${{ secrets.GHCR_TOKEN }}")
        self.assertLess(
            self.production_step_names.index(login_name),
            self.production_step_names.index("Recheck controller digest before rollback"),
        )

    def test_identical_versus_conflicting_manifest_assets(self):
        case = self.cases["identical_versus_conflicting_manifest_assets"]
        run = self.production_steps[case["step"]]["run"]
        for term in case["required_run_terms"]:
            self.assertIn(term, run)
        for term in case["forbidden_run_terms"]:
            self.assertNotIn(term, run)

    def test_release_completion_fails_closed_before_status_and_publication(self):
        manifest_run = self.production_steps["Reconcile release manifest"]["run"]
        mutation_assignment = (
            'production_mutation_required="$(jq -r \'.impact != "repo-only"\' '
            '"$manifest")"'
        )
        self.assertIn(mutation_assignment, manifest_run)
        self.assertIn(f"if ! {mutation_assignment}; then", manifest_run)
        self.assertNotIn('echo "production_mutation_required=$(jq', manifest_run)
        self.assertIn('case "$production_mutation_required" in', manifest_run)
        self.assertLess(
            manifest_run.index(mutation_assignment),
            manifest_run.index(
                'echo "production_mutation_required=${production_mutation_required}"'
            ),
        )

        health = self.production_steps["Verify deployment health"]
        self.assertEqual(health["id"], "deployment_health")
        self.assertEqual(
            health["env"]["EXPECTED_REPO_VERSION"],
            "${{ steps.request.outputs.release_tag }}",
        )
        self.assertEqual(
            health["env"]["EXPECTED_CONTROLLER_VERSION"],
            "${{ steps.manifest.outputs.controller_tag }}",
        )
        self.assertEqual(
            health["env"]["EXPECTED_SOURCE_REVISION"],
            "${{ steps.source.outputs.source_revision }}",
        )
        for release_field in (
            ".release.repoVersion == $repo",
            ".release.controllerVersion == $controller",
            ".release.sourceRevision == $source",
        ):
            self.assertIn(release_field, health["run"])
        self.assertIn('echo "verified=true" >> "$GITHUB_OUTPUT"', health["run"])

        gate_name = "Validate release completion gate"
        gate = self.production_steps[gate_name]
        self.assertEqual(gate["id"], "release_gate")
        self.assertEqual(
            gate["if"], "steps.request.outputs.operation != 'rollback'"
        )
        self.assertEqual(
            gate["env"]["PRODUCTION_MUTATION_REQUIRED"],
            "${{ steps.manifest.outputs.production_mutation_required }}",
        )
        self.assertEqual(
            gate["env"]["DEPLOYMENT_VERIFIED"],
            "${{ steps.deployment_health.outputs.verified }}",
        )
        self.assertIn('case "$PRODUCTION_MUTATION_REQUIRED" in', gate["run"])
        self.assertIn('[ "$DEPLOYMENT_VERIFIED" != "true" ]', gate["run"])
        self.assertIn('echo "ready=true" >> "$GITHUB_OUTPUT"', gate["run"])

        status = self.production_steps["Commit production release status"]
        publish = self.production_steps["Publish GitHub Release"]
        ready = "steps.release_gate.outputs.ready == 'true'"
        self.assertEqual(
            status["if"],
            "steps.request.outputs.operation == 'rollback' || " + ready,
        )
        self.assertEqual(
            publish["if"],
            "steps.request.outputs.operation != 'rollback' && " + ready,
        )
        self.assertLess(
            self.production_step_names.index(gate_name),
            self.production_step_names.index("Commit production release status"),
        )
        self.assertLess(
            self.production_step_names.index("Commit production release status"),
            self.production_step_names.index("Publish GitHub Release"),
        )

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

    def test_full_deploy_preserves_controller_rollback_metadata(self):
        env_template = APP_ENV_PATH.read_text(encoding="utf-8")
        for field in (
            "AUTOGRAPHS_CONTROLLER_DIGEST",
            "AUTOGRAPHS_PREVIOUS_CONTROLLER_IMAGE",
            "AUTOGRAPHS_PREVIOUS_CONTROLLER_VERSION",
            "AUTOGRAPHS_PREVIOUS_CONTROLLER_DIGEST",
        ):
            self.assertIn(field, env_template)
        tasks = ROLE_TASKS_PATH.read_text(encoding="utf-8")
        self.assertLess(
            tasks.index("Resolve existing controller release metadata"),
            tasks.index("Write app environment file"),
        )
        self.assertIn("autographs_deploy_controller_digest", tasks)
        self.assertIn("@{{ autographs_deploy_controller_digest }}", tasks)

    def test_image_probe_fails_closed_and_bake_uses_only_semantic_tag(self):
        image_plan = self.production_steps["Resolve controller image plan"]["run"]
        self.assertIn("probe_controller_image()", image_plan)
        self.assertGreaterEqual(image_plan.count('probe_controller_image "$image"'), 2)
        self.assertIn("manifest unknown|not found|404", image_plan)
        self.assertIn("Unable to determine whether controller image", image_plan)
        self.assertNotIn("imagetools inspect \"$image\" >/dev/null 2>&1", image_plan)
        metadata_paths = re.findall(
            r"--format '\{\{json (\.[A-Za-z.]+)\}\}'", image_plan
        )
        self.assertEqual(metadata_paths, [".Image.Config.Labels"] * 2)
        self.assertNotIn(".Image.config.Labels", image_plan)
        self.assertIn("scripts/release.py validate-reused-controller", image_plan)
        self.assertLess(
            image_plan.index("scripts/release.py validate-reused-controller"),
            image_plan.index('probe_controller_image "$image"'),
        )
        representative_inspection = {
            "Image": {
                "Config": {
                    "Labels": {"org.opencontainers.image.revision": "source-sha"}
                }
            }
        }
        for metadata_path in metadata_paths:
            value = representative_inspection
            for component in metadata_path.removeprefix(".").split("."):
                value = value[component]
            self.assertEqual(
                value["org.opencontainers.image.revision"], "source-sha"
            )
        bake = DOCKER_BAKE_PATH.read_text(encoding="utf-8")
        self.assertEqual(bake.count("${GHCR_CONTROLLER_IMAGE_REPOSITORY}:"), 1)
        self.assertNotIn(":latest", bake)
        self.assertNotIn(":production", bake)

    def test_rollback_playbook_mutates_controller_only_and_verifies_pull(self):
        rollback = ROLLBACK_PATH.read_text(encoding="utf-8")
        self.assertIn("app.env.rollback", rollback)
        self.assertIn("autographs-controller.container.rollback", rollback)
        self.assertIn("@{{ autographs_rollback_controller_digest }}", rollback)
        self.assertIn("AUTOGRAPHS_PREVIOUS_CONTROLLER_DIGEST", rollback)
        self.assertIn("Restart controller after verified rollback", rollback)
        self.assertIn("Verify controller health after rollback", rollback)
        self.assertIn("Verify Caddy health after rollback", rollback)
        self.assertNotIn("terraform apply", rollback)
        self.assertNotIn("AUTOGRAPHS_REPO_VERSION", rollback)
        self.assertNotIn("AUTOGRAPHS_SOURCE_REVISION", rollback)


if __name__ == "__main__":
    unittest.main()
