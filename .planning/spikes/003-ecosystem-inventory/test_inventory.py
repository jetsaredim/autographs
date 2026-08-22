import importlib.util
import json
import os
from pathlib import Path
import stat
import tempfile
import unittest


MODULE_PATH = Path(__file__).with_name("inventory.py")
SPEC = importlib.util.spec_from_file_location("ecosystem_inventory", MODULE_PATH)
inventory = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(inventory)


class InventoryTests(unittest.TestCase):
    def test_repository_inventory_never_emits_values(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".env.example").write_text(
                "ORACLE_DB_PASSWORD=super-secret-value\nAUTOGRAPHS_HTTP_PORT=8080\n",
                encoding="utf-8",
            )
            source = root / "controller" / "src"
            source.mkdir(parents=True)
            (source / "config.rs").write_text(
                'std::env::var("ORACLE_DB_PASSWORD");\n', encoding="utf-8"
            )

            report = inventory.collect_repo(root)
            encoded = json.dumps(report)

            self.assertIn("ORACLE_DB_PASSWORD", report["variables"])
            self.assertNotIn("super-secret-value", encoded)

    def test_repository_inventory_finds_github_secret_and_variable_references(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflows = root / ".github" / "workflows"
            workflows.mkdir(parents=True)
            (workflows / "deploy.yml").write_text(
                "token: ${{ secrets.PORKBUN_API_KEY }}\n"
                "region: ${{ vars.DEPLOY_REGION }}\n",
                encoding="utf-8",
            )

            report = inventory.collect_repo(root)

            self.assertEqual(
                report["variables"]["PORKBUN_API_KEY"]["classification"],
                "secret-scalar",
            )
            self.assertEqual(
                report["variables"]["PORKBUN_API_KEY"]["boundaries"],
                ["deployment"],
            )
            self.assertIn("DEPLOY_REGION", report["variables"])

    def test_repository_inventory_classifies_dockerfile_variants_as_build_contracts(
        self,
    ):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            controller = root / "controller"
            controller.mkdir()
            (controller / "Dockerfile.smoke").write_text(
                "ARG AUTOGRAPHS_SMOKE_IMAGE_VERSION=unknown\n",
                encoding="utf-8",
            )

            report = inventory.collect_repo(root)
            variable = report["variables"]["AUTOGRAPHS_SMOKE_IMAGE_VERSION"]

            self.assertEqual(variable["boundaries"], ["build"])
            self.assertIn("AUTOGRAPHS_SMOKE_IMAGE_VERSION", report["contract_keys"])
            self.assertEqual(variable["sources"][0]["role"], "container-build")

    def test_vm_inventory_extracts_names_and_flags_persistent_secrets(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            opt_root = root / "opt" / "autographs"
            env_root = opt_root / "env"
            env_root.mkdir(parents=True)
            env_file = env_root / "controller.env"
            env_file.write_text(
                "AUTOGRAPHS_HTTP_PORT=8080\nAUTOGRAPHS_ADMIN_PASSWORD=do-not-emit\n",
                encoding="utf-8",
            )
            env_file.chmod(0o600)

            report = inventory.collect_vm(
                opt_root,
                root / "var" / "lib" / "autographs",
                root / "etc" / "containers" / "systemd",
                root / "tmp",
            )
            encoded = json.dumps(report)

            self.assertEqual(
                report["env_files"][0]["keys"],
                ["AUTOGRAPHS_ADMIN_PASSWORD", "AUTOGRAPHS_HTTP_PORT"],
            )
            self.assertEqual(report["env_files"][0]["boundary"], "vm-runtime")
            self.assertNotIn("do-not-emit", encoded)
            self.assertTrue(
                any(
                    item["kind"] == "persistent-plaintext-secret-file"
                    for item in report["findings"]
                )
            )

    def test_comparison_reports_names_without_values(self):
        repo = {
            "contract_keys": ["AUTOGRAPHS_HTTP_PORT", "ORACLE_DB_PASSWORD"],
            "incidental_keys": [],
            "variables": {
                "AUTOGRAPHS_HTTP_PORT": {},
                "ORACLE_DB_PASSWORD": {},
            }
        }
        vm = {
            "env_files": [
                {
                    "path": "env/controller.env",
                    "keys": ["ORACLE_DB_PASSWORD", "UNTRACKED_SETTING"],
                }
            ]
        }

        rendered = inventory.render_comparison(repo, vm)

        self.assertIn("ORACLE_DB_PASSWORD", rendered)
        self.assertIn("UNTRACKED_SETTING", rendered)
        self.assertIn("AUTOGRAPHS_HTTP_PORT", rendered)

    def test_docs_only_mention_does_not_mask_undocumented_vm_key(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".env.example").write_text(
                "AUTOGRAPHS_HTTP_PORT=8080\n", encoding="utf-8"
            )
            docs = root / ".planning" / "history"
            docs.mkdir(parents=True)
            (docs / "old-plan.md").write_text(
                "Retired AUTOGRAPHS_DOCS_ONLY setting.\n", encoding="utf-8"
            )

            repo = inventory.collect_repo(root)
            vm = {
                "env_files": [
                    {
                        "path": "env/controller.env",
                        "keys": ["AUTOGRAPHS_DOCS_ONLY", "AUTOGRAPHS_HTTP_PORT"],
                    }
                ]
            }
            rendered = inventory.render_comparison(repo, vm)

            self.assertIn("AUTOGRAPHS_DOCS_ONLY", repo["incidental_keys"])
            missing_section = rendered.split(
                "### VM Runtime Keys Without a Runtime Template or Controller Consumer", 1
            )[1].split("##", 1)[0]
            self.assertIn("AUTOGRAPHS_DOCS_ONLY", missing_section)

    def test_comparison_keeps_external_and_defaulted_variables_out_of_runtime_drift(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            template = root / "deploy" / "role" / "templates"
            template.mkdir(parents=True)
            (template / "app.env.j2").write_text(
                "AUTOGRAPHS_RUNTIME_REQUIRED=value\n", encoding="utf-8"
            )
            source = root / "controller" / "src"
            source.mkdir(parents=True)
            (source / "config.rs").write_text(
                'env::var("AUTOGRAPHS_RUNTIME_REQUIRED");\n'
                'env::var("AUTOGRAPHS_OPTIONAL_DEFAULT");\n'
                '#[cfg(test)] env::var("AUTOGRAPHS_TEST_ONLY_SETTING");\n',
                encoding="utf-8",
            )
            smoke = root / "controller" / "tests"
            smoke.mkdir(parents=True)
            (smoke / "live_persistence_smoke.rs").write_text(
                'env::var("AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE");\n',
                encoding="utf-8",
            )
            workflows = root / ".github" / "workflows"
            workflows.mkdir(parents=True)
            (workflows / "deploy.yml").write_text(
                "user: ${{ vars.DEPLOY_SSH_USER }}\n", encoding="utf-8"
            )
            (workflows / "image-cleanup.yml").write_text(
                "retain: ${{ vars.GHCR_CLEANUP_RETAIN_TAGGED }}\n", encoding="utf-8"
            )
            terraform = root / "infra" / "terraform"
            terraform.mkdir(parents=True)
            (terraform / "variables.tf").write_text(
                'variable "OCI_RUNTIME_SHAPE" {}\n', encoding="utf-8"
            )

            repo = inventory.collect_repo(root)
            vm = {
                "env_files": [
                    {
                        "path": "env/app.env",
                        "keys": ["AUTOGRAPHS_RUNTIME_REQUIRED"],
                    },
                    {
                        "path": "env/live-persistence-smoke.env",
                        "keys": ["AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE"],
                    },
                ]
            }
            rendered = inventory.render_comparison(repo, vm)

            self.assertEqual(
                repo["variables"]["DEPLOY_SSH_USER"]["boundaries"], ["deployment"]
            )
            self.assertEqual(
                repo["variables"]["GHCR_CLEANUP_RETAIN_TAGGED"]["boundaries"],
                ["maintenance"],
            )
            self.assertEqual(
                repo["variables"]["OCI_RUNTIME_SHAPE"]["boundaries"],
                ["infrastructure"],
            )
            defaulted_section = rendered.split(
                "### Controller Consumers Not Materialized in Runtime Env", 1
            )[1].split("##", 1)[0]
            self.assertIn("AUTOGRAPHS_OPTIONAL_DEFAULT", defaulted_section)
            self.assertNotIn("DEPLOY_SSH_USER", defaulted_section)
            self.assertNotIn("AUTOGRAPHS_TEST_ONLY_SETTING", defaulted_section)
            runtime_drift = rendered.split("## VM Runtime Contract Drift", 1)[1].split(
                "## VM Smoke Contract Review", 1
            )[0]
            self.assertNotIn("GHCR_CLEANUP_RETAIN_TAGGED", runtime_drift)
            self.assertNotIn("OCI_RUNTIME_SHAPE", runtime_drift)

    def test_generated_inventory_reports_do_not_feed_back_into_repository_scan(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".env.example").write_text(
                "AUTOGRAPHS_REAL_SETTING=value\n", encoding="utf-8"
            )
            (root / "autographs-ecosystem-comparison.md").write_text(
                "- `AUTOGRAPHS_REPORT_FEEDBACK`\n", encoding="utf-8"
            )

            report = inventory.collect_repo(root)

            self.assertIn("AUTOGRAPHS_REAL_SETTING", report["variables"])
            self.assertNotIn("AUTOGRAPHS_REPORT_FEEDBACK", report["variables"])

    def test_comparison_reports_actual_key_intersections(self):
        repo = {"variables": {}, "contract_keys": [], "incidental_keys": []}
        vm = {
            "env_files": [
                {
                    "path": "env/app.env",
                    "keys": ["AUTOGRAPHS_PUBLIC_ORIGIN", "ORACLE_DB_USER"],
                },
                {
                    "path": "env/controller.env",
                    "keys": ["OCI_AUTH_MODE"],
                },
                {
                    "path": "env/live-smoke.env",
                    "keys": ["AUTOGRAPHS_PUBLIC_ORIGIN", "ORACLE_DB_USER"],
                },
            ]
        }

        rendered = inventory.render_comparison(repo, vm)
        overlap = rendered.split("## Actual Env File Key Overlap", 1)[1].split(
            "##", 1
        )[0]

        self.assertIn("`env/app.env` and `env/live-smoke.env` (2)", overlap)
        self.assertIn("`AUTOGRAPHS_PUBLIC_ORIGIN`", overlap)
        self.assertNotIn("env/controller.env", overlap)

    def test_schema_one_repository_sources_are_classified_by_path(self):
        repo = {
            "schema_version": 1,
            "contract_keys": ["DEPLOY_SSH_USER", "ORACLE_DB_USER"],
            "incidental_keys": [],
            "variables": {
                "DEPLOY_SSH_USER": {
                    "sources": [
                        {
                            "path": ".github/workflows/deploy.yml",
                            "role": "workflow",
                        }
                    ]
                },
                "ORACLE_DB_USER": {
                    "sources": [
                        {
                            "path": "controller/src/config.rs",
                            "role": "rust-consumer",
                        }
                    ]
                },
            },
        }
        vm = {
            "schema_version": 1,
            "env_files": [
                {
                    "path": "env/app.env",
                    "keys": ["ORACLE_DB_USER"],
                }
            ],
        }

        rendered = inventory.render_comparison(repo, vm)
        deployment = rendered.split("### Deployment", 1)[1].split("###", 1)[0]
        runtime = rendered.split("### VM Runtime", 1)[1].split("###", 1)[0]
        drift = rendered.split("## VM Runtime Contract Drift", 1)[1].split(
            "## VM Smoke Contract Review", 1
        )[0]

        self.assertIn("DEPLOY_SSH_USER", deployment)
        self.assertIn("ORACLE_DB_USER", runtime)
        self.assertNotIn("DEPLOY_SSH_USER", drift)

    def test_private_writer_rejects_symlink_and_forces_mode_0600(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            victim = root / "victim.json"
            victim.write_text("unchanged\n", encoding="utf-8")
            output_link = root / "inventory.json"
            output_link.symlink_to(victim)

            with self.assertRaises(FileExistsError):
                inventory.write_json(output_link, {"kind": "vm"})

            self.assertEqual(victim.read_text(encoding="utf-8"), "unchanged\n")
            output = root / "safe.json"
            old_umask = os.umask(0)
            try:
                inventory.write_json(output, {"kind": "vm"})
            finally:
                os.umask(old_umask)
            self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o600)


if __name__ == "__main__":
    unittest.main()
