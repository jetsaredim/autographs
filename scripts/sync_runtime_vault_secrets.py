#!/usr/bin/env python3
"""Verify runtime Vault values, populate bootstrap placeholders, or rotate one approved value."""

import argparse
import base64
import binascii
import hashlib
import hmac
import io
import json
import os
import re
import stat
import subprocess
import sys
import tempfile
import time
import zipfile

from pathlib import Path


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


def source_value(github_env, placeholder):
    value = os.environ.get(github_env, "").encode("utf-8")
    if not value:
        raise SyncError(f"{github_env} is missing or empty")
    if value == placeholder:
        raise SyncError(f"{github_env} still contains the invalid bootstrap placeholder")
    return value


def current_etag(oci_bin, config_file, secret_id, expected_version):
    try:
        result = json.loads(command([
            oci_bin, "vault", "secret", "get", "--config-file", config_file, "--secret-id", secret_id,
        ]))
        version = int(result["data"]["current-version-number"])
        etag = result["etag"]
        if not isinstance(etag, str) or not etag:
            raise ValueError
    except (KeyError, TypeError, ValueError) as error:
        raise SyncError("Vault did not return current version and concurrency metadata") from error
    if version != expected_version:
        raise SyncError("The Vault version changed before the update")
    return etag


def update_current(oci_bin, config_file, secret_id, name, value, previous_version, operation, *, etag=None):
    version_name = f"github-{operation}-{os.environ.get('GITHUB_RUN_ID', 'manual')}-{name}"
    arguments = [
        oci_bin, "vault", "secret", "update-base64",
        "--config-file", config_file,
        "--secret-id", secret_id,
        "--secret-content-content", base64.b64encode(value).decode("ascii"),
        "--secret-content-name", version_name,
        "--secret-content-stage", "CURRENT",
        "--wait-for-state", "ACTIVE",
        "--force",
    ]
    if etag is not None:
        arguments.extend(("--if-match", etag, "--max-wait-seconds", "180"))
    command(arguments)
    for attempt in range(6):
        actual, version = current_bundle(oci_bin, config_file, secret_id)
        if version > previous_version and hmac.compare_digest(actual, value):
            return version
        if attempt < 5:
            time.sleep(5)
    raise SyncError(f"{name} update did not become CURRENT")


def sync(mode, oci_bin, config_file):
    ids = secret_ids()
    checked = []
    for name, github_env, placeholder in MANUAL_SECRETS:
        github_value = source_value(github_env, placeholder)
        vault_value, version = current_bundle(oci_bin, config_file, ids[name])
        if hmac.compare_digest(vault_value, github_value):
            print(f"{name}: match")
        elif mode == "bootstrap" and version == 1 and vault_value == placeholder:
            checked.append((name, github_value, version))
        else:
            raise SyncError(f"{name} differs from GitHub and is not an untouched bootstrap placeholder")

    for name, github_value, version in checked:
        update_current(oci_bin, config_file, ids[name], name, github_value, version, "bootstrap")
        print(f"{name}: populated from GitHub and verified")


def wallet_archive():
    try:
        data = base64.b64decode(
            "".join(os.environ.get("ORACLE_DB_WALLET_ZIP_BASE64", "").split()), validate=True,
        )
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            entries = archive.infolist()
            names = {entry.filename for entry in entries}
            if not {"ewallet.p12", "cwallet.sso", "tnsnames.ora"} <= names or len(entries) != len(names):
                raise ValueError
            if any(
                not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", entry.filename)
                or entry.is_dir()
                or stat.S_ISLNK(entry.external_attr >> 16)
                or entry.filename in {"wallet.zip", "wallet.zip.b64"}
                for entry in entries
            ):
                raise ValueError
            if sum(entry.file_size for entry in entries) > 10 * 1024 * 1024 or archive.testzip():
                raise ValueError
            with tempfile.TemporaryDirectory() as directory:
                pkcs12_path = Path(directory) / "ewallet.p12"
                pkcs12_path.write_bytes(archive.read("ewallet.p12"))
                command([
                    "openssl", "pkcs12", "-legacy", "-in", str(pkcs12_path),
                    "-passin", "env:ORACLE_DB_WALLET_PASSWORD", "-noout",
                ])
    except (binascii.Error, OSError, RuntimeError, ValueError, zipfile.BadZipFile, SyncError) as error:
        raise SyncError("Wallet ZIP validation failed. Check the archive and its wallet password.") from error
    return data


def rotation_candidate(name, operation, oci_bin, config_file):
    ids = secret_ids()
    selected = None
    for secret_name, github_env, placeholder in MANUAL_SECRETS:
        candidate = source_value(github_env, placeholder)
        actual, version = current_bundle(oci_bin, config_file, ids[secret_name])
        matches = hmac.compare_digest(actual, candidate)
        if version < 2 or actual == placeholder:
            raise SyncError(f"{secret_name} requires initial bootstrap before rotation")
        if secret_name == name:
            if matches != (operation == "reconcile"):
                raise SyncError("Rotate requires a changed value; reconcile requires a matching value")
            selected = (ids[secret_name], candidate, version)
        elif not matches:
            raise SyncError(f"{secret_name} also differs from GitHub; stage one credential at a time")
    if selected is None:
        raise SyncError("Unknown manual rotation credential")
    secret_id, candidate, version = selected
    if name == "admin_password_hash":
        if not re.fullmatch(rb"\$argon2id\$v=19\$m=[0-9]+,t=[0-9]+,p=[0-9]+\$[A-Za-z0-9+/]+\$[A-Za-z0-9+/]+", candidate):
            raise SyncError("The admin credential must be an Argon2id password hash")
        archive = b""
    else:
        archive = wallet_archive()
    return secret_id, candidate, version, archive


def rotation_signature(record, secret_id, candidate, archive):
    key = os.environ.get("RUNTIME_SECRET_ROTATION_AUTH_KEY", "").encode("utf-8")
    if len(key) < 32:
        raise SyncError("Configure a random RUNTIME_SECRET_ROTATION_AUTH_KEY of at least 32 bytes")
    payload = json.dumps(
        [record, secret_id, base64.b64encode(candidate).decode("ascii"), hashlib.sha256(archive).hexdigest()],
        sort_keys=True, separators=(",", ":"),
    ).encode("utf-8")
    return hmac.digest(key, payload, "sha256").hex()


def preflight_rotation(name, operation, repository, issue, oci_bin, config_file):
    secret_id, candidate, version, archive = rotation_candidate(name, operation, oci_bin, config_file)
    record = {"credential": name, "operation": operation, "repository": repository, "issue": issue,
              "version": version, "checked_at": int(time.time()), "run_id": os.environ.get("GITHUB_RUN_ID", "")}
    return {"request": record, "signature": rotation_signature(record, secret_id, candidate, archive)}


def apply_rotation(approval, repository, issue, oci_bin, config_file):
    try:
        record = approval["request"]
        signature = approval["signature"]
        name = record["credential"]
        operation = record["operation"]
        if (
            record["repository"] != repository or record["issue"] != issue
            or not isinstance(signature, str) or operation not in ("rotate", "reconcile")
            or not 0 <= time.time() - record["checked_at"] <= 86400
        ):
            raise ValueError
    except (KeyError, TypeError, ValueError) as error:
        raise SyncError("Invalid or expired secret rotation preflight") from error

    secret_id, candidate, version, archive = rotation_candidate(name, operation, oci_bin, config_file)
    expected = rotation_signature(record, secret_id, candidate, archive)
    if version != record["version"] or not hmac.compare_digest(signature, expected):
        raise SyncError("The Vault version or GitHub candidate changed after preflight")
    if operation == "rotate":
        etag = current_etag(oci_bin, config_file, secret_id, version)
        version = update_current(
            oci_bin, config_file, secret_id, name, candidate, version, "rotation", etag=etag,
        )
    print(f"{name}: CURRENT version {version} verified")
    return archive


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("verify", "bootstrap", "rotation-preflight", "rotation-apply"))
    parser.add_argument("--oci-bin", required=True)
    parser.add_argument("--oci-config-file", required=True)
    parser.add_argument("--repository", default=os.environ.get("GITHUB_REPOSITORY"))
    parser.add_argument("--issue", type=int)
    parser.add_argument("--credential", choices=[name for name, _, _ in MANUAL_SECRETS])
    parser.add_argument("--operation", choices=("rotate", "reconcile"), default="rotate")
    parser.add_argument("--approval", type=Path)
    parser.add_argument("--wallet-output", type=Path)
    args = parser.parse_args()
    try:
        if args.mode in ("bootstrap", "verify"):
            sync(args.mode, args.oci_bin, args.oci_config_file)
        else:
            if not args.repository or not args.issue or args.issue < 1 or args.approval is None:
                parser.error("rotation requires a repository, issue number and preflight file")
            if args.mode == "rotation-preflight":
                if args.credential is None:
                    parser.error("rotation-preflight requires a credential")
                record = preflight_rotation(args.credential, args.operation, args.repository, args.issue,
                                            args.oci_bin, args.oci_config_file)
                args.approval.write_text(json.dumps(record) + "\n", encoding="utf-8")
            else:
                try:
                    approval = json.loads(args.approval.read_text(encoding="utf-8"))
                except (OSError, ValueError) as error:
                    raise SyncError("Could not read the rotation preflight") from error
                archive = apply_rotation(approval, args.repository, args.issue, args.oci_bin, args.oci_config_file)
                if archive and args.wallet_output:
                    args.wallet_output.write_bytes(base64.b64encode(archive))
    except SyncError as error:
        print(f"Vault secret check failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
