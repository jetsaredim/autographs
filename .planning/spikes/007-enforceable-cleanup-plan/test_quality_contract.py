#!/usr/bin/env python3
"""Tests for the cleanup contract prototype."""

from __future__ import annotations

import tempfile
from pathlib import Path
import unittest

from quality_contract import inspect, rust_strings


class QualityContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / "controller" / "src").mkdir(parents=True)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write(self, relative: str, content: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def test_config_owner_and_semantic_bind_styles_are_allowed(self) -> None:
        self.write(
            "controller/src/config.rs",
            'let value = env::var("NAME");\n',
        )
        self.write(
            "controller/src/catalog.rs",
            'let one = "select * from items where id = :1 and state = :2";\n'
            'let two = r#"select * from items where id = :item_id or parent = :item_id"#;\n',
        )

        self.assertEqual(inspect(self.root)["findings"], [])

    def test_distributed_config_debug_macro_and_repeated_bind_are_found(self) -> None:
        self.write(
            "controller/src/routes.rs",
            'let value = std::env::var("NAME");\n'
            'dbg!(value);\n'
            'let sql = r#"update items set title = :1 where id = :1"#;\n',
        )

        rules = {finding["rule"] for finding in inspect(self.root)["findings"]}
        self.assertEqual(
            rules,
            {
                "distributed-env-read",
                "production-placeholder-macro",
                "unsafe-numeric-sql-bind-order",
            },
        )

    def test_out_of_order_numeric_binds_are_found(self) -> None:
        self.write(
            "controller/src/oracle_catalog.rs",
            'let sql = r#"update credits set sort_order = :3, role = :4, '
            'context = :5 where item_id = :1 and signer_id = :2"#;\n',
        )

        findings = inspect(self.root)["findings"]
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0]["rule"], "unsafe-numeric-sql-bind-order")

    def test_string_extractor_handles_raw_and_escaped_strings(self) -> None:
        values = rust_strings('let a = r##"select :1"##; let b = "update x\\\" :2";')
        self.assertEqual(values, ["select :1", 'update x\\\" :2'])

    def test_persistent_secret_values_are_found_but_secret_ocids_are_allowed(self) -> None:
        self.write(
            "deploy/role/templates/controller.env.j2",
            "ORACLE_DB_PASSWORD={{ lookup('env', 'ORACLE_DB_PASSWORD') }}\n"
            "OCI_PRIVATE_KEY_PEM={{ private_key }}\n"
            "ORACLE_DB_WALLET_ZIP_BASE64={{ wallet_zip }}\n"
            "ORACLE_DB_WALLET_PEM={{ wallet_pem }}\n"
            "PORKBUN_SECRET_KEY={{ secret_key }}\n"
            "EXAMPLE_API_KEY={{ api_key }}\n"
            "ORACLE_DB_PASSWORD_SECRET_OCID=ocid1.vaultsecret.example\n"
            "AUTOGRAPHS_PUBLIC_ORIGIN=https://example.test\n",
        )

        findings = inspect(self.root)["findings"]
        self.assertEqual(len(findings), 6)
        self.assertTrue(
            all(finding["rule"] == "persistent-secret-env-sink" for finding in findings)
        )
        self.assertEqual(
            {finding["variable"] for finding in findings},
            {
                "EXAMPLE_API_KEY",
                "OCI_PRIVATE_KEY_PEM",
                "ORACLE_DB_PASSWORD",
                "ORACLE_DB_WALLET_ZIP_BASE64",
                "ORACLE_DB_WALLET_PEM",
                "PORKBUN_SECRET_KEY",
            },
        )


if __name__ == "__main__":
    unittest.main()
