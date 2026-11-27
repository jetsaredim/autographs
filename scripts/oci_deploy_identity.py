#!/usr/bin/env python3
"""Load the reviewed OCI deploy credential and check signing-key PRs."""

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
from urllib.error import HTTPError
from urllib.request import Request, urlopen

SELECTOR = ".github/oci-deploy-identity.json"
LABEL = "approved-oci-signing-key-rotation"
STATUS = "OCI signing key rotation"
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


def pages(path):
    for page in range(1, 101):
        batch = api(f"{path}?per_page=100&page={page}")
        yield from batch
        if len(batch) < 100:
            return
    raise ValueError("GitHub response exceeded the pagination limit.")


def selector(value):
    if value == {"slot": "legacy"}:
        return value
    if (
        not isinstance(value, dict)
        or set(value) != {"slot", "fingerprint", "approval_pr"}
        or value["slot"] not in ("legacy", "a", "b")
        or not isinstance(value["fingerprint"], str)
        or not FINGERPRINT.fullmatch(value["fingerprint"])
        or type(value["approval_pr"]) is not int
        or value["approval_pr"] < 1
    ):
        raise ValueError("Invalid OCI signing-key selector.")
    return value


def read_selector(ref, allow_missing=False):
    try:
        content = api(f"contents/{SELECTOR}?ref={ref}")
    except HTTPError as error:
        if allow_missing and error.code == 404:
            return {"slot": "legacy"}
        raise
    return selector(json.loads(base64.b64decode(content["content"])))


def same_repo_pr(pr):
    return (
        pr["base"]["ref"] == "main"
        and (pr["head"]["repo"] or {}).get("full_name") == os.environ["GITHUB_REPOSITORY"]
        and pr["user"]["login"] != "dependabot[bot]"
    )


def latest_label(number, through=None):
    events = (
        event for event in pages(f"issues/{number}/events")
        if event["event"] in ("labeled", "unlabeled")
        and event["label"]["name"] == LABEL
        and (through is None or event["created_at"] <= through)
    )
    return max(events, key=lambda event: event["id"], default=None)


def owner_approval(event):
    return (
        event is not None
        and event["event"] == "labeled"
        and event["actor"]["login"] == os.environ["REPOSITORY_OWNER"]
    )


def description(config, event_id):
    return f'{config["slot"]} {config["fingerprint"]} approval {event_id}'


def require_merged_approval(config):
    number = config["approval_pr"]
    pr = api(f"pulls/{number}")
    merged = pr["merged_at"]
    if not merged or not same_repo_pr(pr):
        raise ValueError("Signing-key selector must reference a merged rotation PR.")
    event = latest_label(number, merged)
    if not owner_approval(event):
        raise ValueError("Rotation PR did not have owner approval at merge.")
    head = pr["head"]["sha"]
    if read_selector(head) != config:
        raise ValueError("Signing-key selector does not match the reviewed PR head.")
    approved_status = next((
        entry for entry in pages(f"commits/{head}/statuses")
        if entry["context"] == STATUS and entry["created_at"] <= merged
    ), None)
    if (
        not approved_status
        or approved_status["state"] != "success"
        or (approved_status["creator"] or {}).get("login") != "github-actions[bot]"
        or approved_status["description"] != description(config, event["id"])
    ):
        raise ValueError("No successful signing-key check matched the merged PR head and approval.")


def set_status(head, state, summary):
    api(f"statuses/{head}", {
        "context": STATUS,
        "state": state,
        "description": summary,
        "target_url": (
            f'{os.environ["GITHUB_SERVER_URL"]}/{os.environ["GITHUB_REPOSITORY"]}'
            f'/actions/runs/{os.environ["GITHUB_RUN_ID"]}'
        ),
    })


def prepare(metadata_file, candidate_file):
    with open(os.environ["GITHUB_EVENT_PATH"], encoding="utf-8") as source:
        payload = json.load(source)
    number = payload["number"]
    head = payload["pull_request"]["head"]["sha"]
    pr = api(f"pulls/{number}")
    if pr["state"] != "open" or pr["head"]["sha"] != head:
        return
    set_status(head, "pending", "Checking proposed signing key")
    metadata = {"number": number, "head": head}
    metadata_file.write_text(json.dumps(metadata))
    if not same_repo_pr(pr):
        raise ValueError("Signing-key changes require a branch in this repository.")
    files = list(pages(f"pulls/{number}/files"))
    if [item["filename"] for item in files] != [SELECTOR] or files[0]["status"] != "modified":
        raise ValueError("Signing-key rotation PRs must change only the selector.")
    config = read_selector(head)
    if config.get("approval_pr") != number:
        raise ValueError("Set approval_pr to the number of this rotation PR.")
    candidate_file.write_text(json.dumps(config))
    if (
        payload["action"] == "labeled"
        and payload["label"]["name"] == LABEL
        and payload["sender"]["login"] == os.environ["REPOSITORY_OWNER"]
        and os.environ["GITHUB_RUN_ATTEMPT"] == "1"
    ):
        event = latest_label(number)
        if owner_approval(event) and event["commit_id"] in (None, head):
            metadata["approval"] = event["id"]
            metadata_file.write_text(json.dumps(metadata))


def report(metadata_file, candidate_file, passed):
    if not metadata_file.exists():
        return
    metadata = json.loads(metadata_file.read_text())
    number, head = metadata["number"], metadata["head"]
    pr = api(f"pulls/{number}")
    if pr["state"] != "open" or pr["head"]["sha"] != head:
        return
    if not passed:
        set_status(head, "failure", "Candidate signing-key check failed")
        return
    event = latest_label(number)
    if owner_approval(event) and event["id"] == metadata.get("approval"):
        config = selector(json.loads(candidate_file.read_text()))
        set_status(head, "success", description(config, event["id"]))
    else:
        set_status(head, "failure", f"Candidate passed. Re-apply {LABEL} to approve this head.")
        raise ValueError(f"Candidate passed. The owner must apply {LABEL} to this PR head.")


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


def load(config, candidate=False):
    if not candidate and "approval_pr" in config:
        require_merged_approval(config)
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
    loader = subcommands.add_parser("load")
    source = loader.add_mutually_exclusive_group(required=True)
    source.add_argument("--ref")
    source.add_argument("--candidate", type=Path)
    loader.add_argument("--allow-missing", action="store_true")
    for name in ("prepare", "report"):
        handler = subcommands.add_parser(name)
        handler.add_argument("metadata", type=Path)
        handler.add_argument("candidate_file", type=Path)
        if name == "report":
            handler.add_argument("--passed", action="store_true")
    args = command.parse_args()
    try:
        if args.command == "prepare":
            prepare(args.metadata, args.candidate_file)
        elif args.command == "report":
            report(args.metadata, args.candidate_file, args.passed)
        elif args.ref:
            load(read_selector(args.ref, args.allow_missing))
        else:
            load(selector(json.loads(args.candidate.read_text())), candidate=True)
    except HTTPError as error:
        command.exit(1, f"OCI signing-key check: GitHub API returned HTTP {error.code}.\n")
    except json.JSONDecodeError:
        command.exit(1, "OCI signing-key check: invalid JSON.\n")
    except ValueError as error:
        command.exit(1, f"OCI signing-key check: {error}\n")


if __name__ == "__main__":
    main()
