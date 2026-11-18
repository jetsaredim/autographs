#!/usr/bin/env python3
"""Compare runtime Vault values with GitHub Secrets, or fill exact bootstrap placeholders."""

import argparse
import base64
import hmac
import json
import os
import subprocess
import sys
import time


MANUAL_SECRETS = (
    ("admin_password_hash", "AUTOGRAPHS_ADMIN_PASSWORD_HASH", b"AUTOGRAPHS_UNCONFIGURED_ADMIN_PASSWORD_HASH"),
    ("oracle_db_wallet_password", "ORACLE_DB_WALLET_PASSWORD", b"AUTOGRAPHS_UNCONFIGURED_ORACLE_DB_WALLET_PASSWORD"),
)


class SyncError(Exception):
    pass


def command(argv):
    result = subprocess.run(argv, capture_output=True, check=False)
    if result.returncode != 0:
        raise SyncError("Required Terraform or OCI command failed; no secret content was logged")
    return result.stdout


def secret_ids():
    try:
        ids = json.loads(command(["terraform", "output", "-json", "runtime_secret_ids"]))
        return {name: ids[name] for name, _, _ in MANUAL_SECRETS}
    except (ValueError, KeyError, TypeError) as error:
        raise SyncError("Runtime Terraform state does not contain both manual Vault secret IDs") from error


def current_bundle(oci_bin, config_file, secret_id):
    try:
        data = json.loads(command([
            oci_bin, "secrets", "secret-bundle", "get",
            "--config-file", config_file,
            "--secret-id", secret_id, "--stage", "CURRENT",
        ]))["data"]
        content = base64.b64decode(data["secret-bundle-content"]["content"], validate=True)
        version = int(data["version-number"])
        return content, version
    except (ValueError, KeyError, TypeError) as error:
        raise SyncError("OCI returned an invalid CURRENT secret bundle") from error


def sync(mode, oci_bin, config_file):
    ids = secret_ids()
    checked = []
    for name, github_env, placeholder in MANUAL_SECRETS:
        github_value = os.environ.get(github_env, "").encode("utf-8")
        if not github_value:
            raise SyncError(f"{github_env} is missing or empty")
        if github_value == placeholder:
            raise SyncError(f"{github_env} still contains the invalid bootstrap placeholder")
        vault_value, version = current_bundle(oci_bin, config_file, ids[name])
        matched = hmac.compare_digest(vault_value, github_value)
        if not matched and (mode != "bootstrap" or version != 1 or vault_value != placeholder):
            raise SyncError(f"{name} differs from GitHub and is not an untouched bootstrap placeholder")
        checked.append((name, github_value, ids[name], matched))

    for name, github_value, secret_id, matched in checked:
        if matched:
            print(f"{name}: match")
            continue
        encoded = base64.b64encode(github_value).decode("ascii")
        version_name = f"github-bootstrap-{os.environ.get('GITHUB_RUN_ID', 'manual')}-{name}"
        command([
            oci_bin, "vault", "secret", "update-base64",
            "--config-file", config_file,
            "--secret-id", secret_id,
            "--secret-content-content", encoded,
            "--secret-content-name", version_name,
            "--secret-content-stage", "CURRENT",
            "--wait-for-state", "ACTIVE",
            "--force",
        ])
        for attempt in range(6):
            actual, version = current_bundle(oci_bin, config_file, secret_id)
            if version >= 2 and hmac.compare_digest(actual, github_value):
                break
            if attempt < 5:
                time.sleep(5)
        else:
            raise SyncError(f"{name} bootstrap update did not become CURRENT")
        print(f"{name}: populated from GitHub and verified")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("verify", "bootstrap"))
    parser.add_argument("--oci-bin", required=True)
    parser.add_argument("--oci-config-file", required=True)
    args = parser.parse_args()
    try:
        sync(args.mode, args.oci_bin, args.oci_config_file)
    except SyncError as error:
        print(f"Vault secret check failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
