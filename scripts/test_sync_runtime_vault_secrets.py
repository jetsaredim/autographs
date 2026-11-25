"""Fail-closed tests for the GitHub-to-OCI Vault bootstrap boundary."""

import base64
import json
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


class RuntimeVaultRotationTests(unittest.TestCase):
    credential = "admin_password_hash"
    repository = "owner/repo"
    issue = 42
    admin_candidate = "$argon2id$v=19$m=19456,t=2,p=1$testsalt$testcandidate"

    def setUp(self):
        self.bundles = {"admin-id": (b"previous-hash", 7), "wallet-id": (b"wallet-password", 4)}
        self.enterContext(patch.dict(os.environ, {
            **GITHUB_VALUES,
            "AUTOGRAPHS_ADMIN_PASSWORD_HASH": self.admin_candidate,
            "RUNTIME_SECRET_ROTATION_AUTH_KEY": "test-only-key-" * 3,
        }))
        self.enterContext(patch.object(sync, "secret_ids", return_value=IDS))
        self.enterContext(patch.object(sync, "current_bundle", side_effect=lambda _, __, key: self.bundles[key]))
        self.metadata = self.enterContext(patch.object(sync, "current_etag", return_value="etag-7"))
        self.write = self.enterContext(patch.object(sync, "update_current", return_value=8))

    def check(self, operation="rotate"):
        return sync.preflight_rotation(self.credential, operation, self.repository, self.issue, "oci", "config")

    def apply(self, approval, issue=None):
        return sync.apply_rotation(approval, self.repository, issue or self.issue, "oci", "config")

    def test_preflight_is_read_only_and_approval_only_writes_the_requested_value(self):
        approval = self.check()
        self.assertNotIn(self.admin_candidate, json.dumps(approval))
        self.write.assert_not_called()
        self.apply(approval)
        self.write.assert_called_once_with(
            "oci", "config", "admin-id", self.credential, self.admin_candidate.encode(), 7, "rotation",
            etag="etag-7",
        )
        self.metadata.assert_called_once_with("oci", "config", "admin-id", 7)

    def test_changed_version_candidate_or_issue_prevents_write(self):
        approval = self.check()
        with patch.dict(os.environ, {"AUTOGRAPHS_ADMIN_PASSWORD_HASH": self.admin_candidate + "new"}):
            with self.assertRaisesRegex(sync.SyncError, "changed after preflight"):
                self.apply(approval)
        with self.assertRaisesRegex(sync.SyncError, "Invalid or expired"):
            self.apply(approval, issue=43)
        self.bundles["admin-id"] = (b"another-hash", 8)
        with self.assertRaisesRegex(sync.SyncError, "changed after preflight"):
            self.apply(approval)
        self.write.assert_not_called()

    def test_approval_binds_wallet_archive_as_well_as_password(self):
        self.credential = "oracle_db_wallet_password"
        self.bundles["admin-id"] = (self.admin_candidate.encode(), 7)
        with patch.dict(os.environ, {"ORACLE_DB_WALLET_PASSWORD": "new-wallet-password"}), patch.object(
            sync, "wallet_archive", side_effect=[b"checked-archive", b"changed-archive"]
        ):
            approval = self.check()
            with self.assertRaisesRegex(sync.SyncError, "changed after preflight"):
                self.apply(approval)
        self.write.assert_not_called()

    def test_reconcile_checks_matching_values_without_a_new_vault_version(self):
        self.bundles["admin-id"] = (self.admin_candidate.encode(), 8)
        self.apply(self.check(operation="reconcile"))
        self.write.assert_not_called()

    def test_preflight_rejects_drift_in_the_other_manual_credential(self):
        self.bundles["wallet-id"] = (b"unexpected", 4)
        with self.assertRaisesRegex(sync.SyncError, "stage one credential at a time"):
            self.check()
        self.write.assert_not_called()


if __name__ == "__main__":
    unittest.main()
