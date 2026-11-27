#!/usr/bin/env python3
"""Load the active OCI deploy credential and record verified slot switches."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
from urllib.error import HTTPError
from urllib.request import Request, urlopen

SELECTION_ENVIRONMENT = "oci-deploy-signer"
SELECTION_TASK = "rotate-oci-deploy-signer"
BOT = "github-actions[bot]"
FINGERPRINT = re.compile(r"(?:[0-9a-f]{2}:){15}[0-9a-f]{2}")
OCID = re.compile(r"ocid1\.(?:user|tenancy)\.[a-z0-9.]+")
REGION = "us-ashburn-1"


def api(path, data=None):
    repository = os.environ["GITHUB_REPOSITORY"]
    request = Request(
        f'{os.environ["GITHUB_API_URL"]}/repos/{repository}/{path}',
        data=json.dumps(data).encode() if data is not None else None,
        headers={
            "Authorization": f'Bearer {os.environ["GH_TOKEN"]}',
            "Accept": "application/vnd.github+json",
            "Content-Type": "application/json",
        },
    )
    with urlopen(request, timeout=30) as response:
        return json.load(response)


def selector(value):
    if (
        not isinstance(value, dict)
        or set(value) != {"slot", "fingerprint"}
        or value["slot"] not in ("legacy", "a", "b")
        or not isinstance(value["fingerprint"], str)
        or not FINGERPRINT.fullmatch(value["fingerprint"])
    ):
        raise ValueError("Invalid OCI signing-key selector.")
    return value


def signer_deployments():
    query = f"environment={SELECTION_ENVIRONMENT}&task={SELECTION_TASK}&per_page=100"
    for page in range(1, 101):
        batch = api(f"deployments?{query}&page={page}")
        yield from batch
        if len(batch) < 100:
            return
    raise ValueError("GitHub deployment history exceeded the pagination limit.")


def active_selector():
    # Pending records from interrupted runs never become active.
    for deployment in sorted(signer_deployments(), key=lambda item: item["id"], reverse=True):
        if (
            deployment["task"] != SELECTION_TASK
            or deployment["environment"] != SELECTION_ENVIRONMENT
            or (deployment["creator"] or {}).get("login") != BOT
        ):
            continue
        statuses = api(f'deployments/{deployment["id"]}/statuses?per_page=1')
        if statuses and statuses[0]["state"] == "success" and statuses[0]["creator"]["login"] == BOT:
            payload = deployment["payload"]
            return selector(json.loads(payload) if isinstance(payload, str) else payload)
    return {"slot": "legacy"}


def record_activation(config):
    if active_selector() == config:
        print(f'Slot {config["slot"]} is already active with this fingerprint.')
        return
    deployment = api("deployments", {
        "ref": os.environ["GITHUB_SHA"],
        "task": SELECTION_TASK,
        "environment": SELECTION_ENVIRONMENT,
        "auto_merge": False,
        "required_contexts": [],
        "payload": config,
        "description": f'OCI deploy signing-key slot {config["slot"]}',
    })
    api(f'deployments/{deployment["id"]}/statuses', {
        "state": "success",
        "auto_inactive": False,
        "log_url": (
            f'{os.environ["GITHUB_SERVER_URL"]}/{os.environ["GITHUB_REPOSITORY"]}'
            f'/actions/runs/{os.environ["GITHUB_RUN_ID"]}'
        ),
    })
    print(f'Activated OCI deploy signing-key slot {config["slot"]}.')


def key_fingerprint(pem):
    result = subprocess.run(
        ["openssl", "pkey", "-pubout", "-outform", "DER"],
        input=pem.encode(), stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, check=False,
    )
    if result.returncode or not result.stdout or not pem.startswith(
        ("-----BEGIN PRIVATE KEY-----", "-----BEGIN RSA PRIVATE KEY-----")
    ):
        raise ValueError("OCI signing key is missing or unreadable.")
    # OCI identifies API signing keys by the MD5 fingerprint of the DER public key.
    return hashlib.md5(result.stdout, usedforsecurity=False).digest().hex(":")


def credentials(config):
    slot = config["slot"]
    if slot == "legacy":
        pem = os.environ.get("OCI_PRIVATE_KEY_PEM", "")
        fingerprint = os.environ.get("OCI_FINGERPRINT", "")
        user = os.environ.get("OCI_CLI_USER_OCID", "")
    else:
        try:
            bundle = json.loads(os.environ.get(f"OCI_DEPLOY_KEY_{slot.upper()}", ""))
            pem, fingerprint = bundle["private_key_pem"], bundle["fingerprint"]
        except (TypeError, KeyError, ValueError):
            raise ValueError(f"OCI signing-key bundle for slot {slot} is missing or invalid.") from None
        user = os.environ.get("OCI_DEPLOY_USER_OCID", "")
    tenancy = os.environ.get("OCI_TENANCY_OCID", "")
    if not OCID.fullmatch(user) or not user.startswith("ocid1.user."):
        raise ValueError("OCI deploy user OCID is missing or invalid.")
    if not OCID.fullmatch(tenancy) or not tenancy.startswith("ocid1.tenancy."):
        raise ValueError("OCI tenancy OCID is missing or invalid.")
    if (
        not isinstance(pem, str)
        or not isinstance(fingerprint, str)
        or fingerprint != config.get("fingerprint", fingerprint)
        or fingerprint != key_fingerprint(pem)
    ):
        raise ValueError(f"OCI signing key fingerprint does not match slot {slot}.")
    return user, tenancy, fingerprint, pem


def load(config):
    user, tenancy, fingerprint, pem = credentials(config)
    os.umask(0o077)
    directory = Path(os.environ["RUNNER_TEMP"], "oci")
    directory.mkdir(mode=0o700, exist_ok=True)
    path = directory / "api_key.pem"
    path.write_text(pem.rstrip() + "\n")
    path.chmod(0o600)
    cli_config = directory / "config"
    cli_config.write_text(
        f"[DEFAULT]\nuser={user}\ntenancy={tenancy}\nfingerprint={fingerprint}\n"
        f"key_file={path}\nregion={REGION}\n"
    )
    cli_config.chmod(0o600)
    # Export identifiers and paths only. Terraform's OCI backend and provider
    # each use their own set of names.
    env = {
        "OCI_CLI_USER_OCID": user, "OCI_TENANCY_OCID": tenancy,
        "OCI_FINGERPRINT": fingerprint, "OCI_PRIVATE_KEY_PATH": path,
        "OCI_REGION": REGION,
    }
    for key, value in (
        ("tenancy_ocid", tenancy), ("user_ocid", user),
        ("fingerprint", fingerprint), ("private_key_path", path),
    ):
        env[key] = value
        env[f"TF_VAR_{key}"] = value
    with open(os.environ["GITHUB_ENV"], "a", encoding="utf-8") as target:
        for key, value in env.items():
            target.write(f"{key}={value}\n")
    print(f'Using OCI signing-key slot {config["slot"]}.')


def main():
    command = argparse.ArgumentParser()
    subcommands = command.add_subparsers(dest="command", required=True)
    subcommands.add_parser("load")
    subcommands.add_parser("check-candidate")
    subcommands.add_parser("activate")
    args = command.parse_args()
    try:
        if args.command == "load":
            load(active_selector())
        else:
            candidate = selector({
                "slot": os.environ["CANDIDATE_SLOT"],
                "fingerprint": os.environ["CANDIDATE_FINGERPRINT"],
            })
            if args.command == "check-candidate":
                active = active_selector()
                if active["slot"] == candidate["slot"] and active.get("fingerprint") not in (
                    None, candidate["fingerprint"],
                ):
                    raise ValueError("Stage the candidate in the inactive slot.")
                load(candidate)
            else:
                record_activation(candidate)
    except HTTPError as error:
        command.exit(1, f"OCI signing-key check: GitHub API returned HTTP {error.code}.\n")
    except json.JSONDecodeError:
        command.exit(1, "OCI signing-key check: invalid JSON.\n")
    except ValueError as error:
        command.exit(1, f"OCI signing-key check: {error}\n")


if __name__ == "__main__":
    main()
