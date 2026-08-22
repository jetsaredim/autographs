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
            self.assertIn("DEPLOY_REGION", report["variables"])

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
                "## VM Keys Missing From Repository Contract", 1
            )[1].split("##", 1)[0]
            self.assertIn("AUTOGRAPHS_DOCS_ONLY", missing_section)

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
