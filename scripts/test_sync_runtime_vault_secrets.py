"""Fail-closed tests for the GitHub-to-OCI Vault bootstrap boundary."""

import base64
import os
import unittest
from unittest.mock import patch

from scripts import sync_runtime_vault_secrets as sync


IDS = {
    "admin_password_hash": "admin-id",
    "oracle_db_wallet_password": "wallet-id",
}
GITHUB_VALUES = {
    "AUTOGRAPHS_ADMIN_PASSWORD_HASH": "argon2-hash",
    "ORACLE_DB_WALLET_PASSWORD": "wallet-password",
}


class SyncRuntimeVaultSecretsTests(unittest.TestCase):
    def run_sync(self, mode, bundles, github_values=None):
        updates = []

        def current_bundle(_oci_bin, _config_file, secret_id):
            return bundles[secret_id]

        def command(argv):
            updates.append(argv)
            secret_id = argv[argv.index("--secret-id") + 1]
            encoded = argv[argv.index("--secret-content-content") + 1]
            bundles[secret_id] = (base64.b64decode(encoded), 2)
            return b"{}"

        with patch.dict(os.environ, github_values or GITHUB_VALUES), patch.object(sync, "secret_ids", return_value=IDS), patch.object(
            sync, "current_bundle", side_effect=current_bundle
        ), patch.object(sync, "command", side_effect=command):
            sync.sync(mode, "oci", "config")
        return updates

    def test_verify_matching_values_does_not_write(self):
        bundles = {"admin-id": (b"argon2-hash", 3), "wallet-id": (b"wallet-password", 4)}
        self.assertEqual(self.run_sync("verify", bundles), [])

    def test_verify_rejects_mismatch_without_writing(self):
        bundles = {"admin-id": (b"different", 3), "wallet-id": (b"wallet-password", 4)}
        with self.assertRaisesRegex(sync.SyncError, "differs from GitHub"):
            self.run_sync("verify", bundles)

    def test_bootstrap_replaces_only_exact_version_one_placeholders(self):
        bundles = {
            "admin-id": (b"AUTOGRAPHS_UNCONFIGURED_ADMIN_PASSWORD_HASH", 1),
            "wallet-id": (b"AUTOGRAPHS_UNCONFIGURED_ORACLE_DB_WALLET_PASSWORD", 1),
        }
        updates = self.run_sync("bootstrap", bundles)
        self.assertEqual(len(updates), 2)
        self.assertEqual(bundles["admin-id"], (b"argon2-hash", 2))
        self.assertEqual(bundles["wallet-id"], (b"wallet-password", 2))

    def test_bootstrap_preflight_rejects_unexpected_value_before_any_write(self):
        bundles = {
            "admin-id": (b"AUTOGRAPHS_UNCONFIGURED_ADMIN_PASSWORD_HASH", 1),
            "wallet-id": (b"different", 2),
        }
        with self.assertRaisesRegex(sync.SyncError, "differs from GitHub"):
            self.run_sync("bootstrap", bundles)
        self.assertEqual(bundles["admin-id"][1], 1)

    def test_bootstrap_is_idempotent_after_success(self):
        bundles = {"admin-id": (b"argon2-hash", 2), "wallet-id": (b"wallet-password", 2)}
        self.assertEqual(self.run_sync("bootstrap", bundles), [])

    def test_bootstrap_rejects_placeholder_as_github_source(self):
        bundles = {
            "admin-id": (b"AUTOGRAPHS_UNCONFIGURED_ADMIN_PASSWORD_HASH", 1),
            "wallet-id": (b"AUTOGRAPHS_UNCONFIGURED_ORACLE_DB_WALLET_PASSWORD", 1),
        }
        values = dict(GITHUB_VALUES)
        values["AUTOGRAPHS_ADMIN_PASSWORD_HASH"] = "AUTOGRAPHS_UNCONFIGURED_ADMIN_PASSWORD_HASH"
        with self.assertRaisesRegex(sync.SyncError, "invalid bootstrap placeholder"):
            self.run_sync("bootstrap", bundles, values)
        self.assertEqual(bundles["admin-id"][1], 1)

    def test_bootstrap_rejects_mismatched_version_one_value(self):
        bundles = {
            "admin-id": (b"unexpected-version-one-value", 1),
            "wallet-id": (b"AUTOGRAPHS_UNCONFIGURED_ORACLE_DB_WALLET_PASSWORD", 1),
        }
        with self.assertRaisesRegex(sync.SyncError, "differs from GitHub"):
            self.run_sync("bootstrap", bundles)
        self.assertEqual(bundles["wallet-id"][1], 1)


if __name__ == "__main__":
    unittest.main()
