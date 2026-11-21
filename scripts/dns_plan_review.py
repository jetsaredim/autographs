#!/usr/bin/env python3
"""Bind ordinary release DNS actions to the successful Release PR Terraform plan."""

import argparse
import base64
import hashlib
import json
import re
import subprocess
import sys

MARKER = "AUTOGRAPHS_DNS_REVIEW="
DNS_ADDRESS = re.compile(r"^(cloudflare_|porkbun_dns_record\.)")


def dns_changes(plan):
    changes = [
        {"address": item["address"], "mode": item["mode"], "change": item["change"]}
        for item in plan.get("resource_changes", [])
        if item.get("mode") == "managed"
        and DNS_ADDRESS.match(item.get("address", ""))
        and item.get("change", {}).get("actions") != ["no-op"]
    ]
    return sorted(changes, key=lambda item: item["address"])


def review_record(plan, state, tree, head, pr, mode):
    if mode == "off" and state["cloudflare_zone_present"]:
        raise ValueError("Cloudflare mode off cannot remove the managed production zone")
    changes = dns_changes(plan)
    serialized = json.dumps(changes, sort_keys=True, separators=(",", ":"))
    return {
        "version": 1,
        "pr": int(pr),
        "head": head,
        "terraform_tree": tree,
        "state_lineage": state["lineage"],
        "state_serial": state["serial"],
        "cloudflare_mode": mode,
        "dns_change_count": len(changes),
        "dns_change_sha256": hashlib.sha256(serialized.encode()).hexdigest(),
    }


def gh_api(path, *, text=False):
    result = subprocess.run(
        ["gh", "api", path], check=True, capture_output=True, text=True
    )
    return result.stdout if text else json.loads(result.stdout)


def reviewed_ci_record(repo, source_revision):
    prs = gh_api(f"repos/{repo}/commits/{source_revision}/pulls")
    merged = [
        pr for pr in prs
        if pr.get("merged_at")
        and pr.get("merge_commit_sha") == source_revision
        and pr.get("base", {}).get("ref") == "main"
    ]
    if len(merged) != 1:
        raise ValueError("release source must identify exactly one merged main PR")
    pr = merged[0]
    head = pr["head"]["sha"]
    number = pr["number"]
    runs = gh_api(
        f"repos/{repo}/actions/runs?event=pull_request&head_sha={head}&per_page=100"
    )["workflow_runs"]
    matching = [
        run for run in runs
        if run.get("path") == ".github/workflows/ci.yml"
        and any(item.get("number") == number for item in run.get("pull_requests", []))
    ]
    if not matching:
        raise ValueError("no CI run found for the merged Release PR head")
    run = max(matching, key=lambda item: item["id"])
    if run.get("status") != "completed" or run.get("conclusion") != "success":
        raise ValueError("latest Release PR CI run did not pass")
    jobs = gh_api(f"repos/{repo}/actions/runs/{run['id']}/jobs?per_page=100")["jobs"]
    terraform_jobs = [job for job in jobs if job.get("name") == "Terraform checks"]
    if len(terraform_jobs) != 1 or terraform_jobs[0].get("conclusion") != "success":
        raise ValueError("successful Release PR Terraform job is required")
    log = gh_api(f"repos/{repo}/actions/jobs/{terraform_jobs[0]['id']}/logs", text=True)
    encoded = re.findall(r"(?:^|\s)" + MARKER + r"([A-Za-z0-9_-]+)(?:\s|$)", log)
    if len(encoded) != 1:
        raise ValueError("Release PR Terraform log must contain one DNS review record")
    record = json.loads(base64.urlsafe_b64decode(encoded[0] + "=" * (-len(encoded[0]) % 4)))
    if record.get("pr") != number or record.get("head") != head:
        raise ValueError("CI DNS review record does not match the merged Release PR")
    return record


def verify(plan, state, tree, mode, repo, source_revision):
    if mode == "off" and state["cloudflare_zone_present"]:
        raise ValueError("Cloudflare mode off cannot remove the managed production zone")
    changes = dns_changes(plan)
    if not changes:
        print("No Cloudflare or Porkbun DNS changes in the saved release plan.")
        return
    expected = reviewed_ci_record(repo, source_revision)
    actual = review_record(
        plan, state, tree, expected["head"], expected["pr"], mode
    )
    if actual != expected:
        raise ValueError("DNS changes, mode, Terraform tree, or state differ from Release PR CI")
    print(f"Verified {len(changes)} DNS change(s) against Release PR CI plan.")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["emit", "verify"])
    parser.add_argument("--state-meta", required=True)
    parser.add_argument("--tree", required=True)
    parser.add_argument("--mode", required=True)
    parser.add_argument("--head")
    parser.add_argument("--pr", type=int)
    parser.add_argument("--repo")
    parser.add_argument("--source-revision")
    args = parser.parse_args()
    plan = json.load(sys.stdin)
    with open(args.state_meta, encoding="utf-8") as handle:
        state = json.load(handle)
    try:
        if args.command == "emit":
            if not args.head or not args.pr:
                raise ValueError("PR head and number are required for CI evidence")
            record = review_record(plan, state, args.tree, args.head, args.pr, args.mode)
            encoded = base64.urlsafe_b64encode(
                json.dumps(record, sort_keys=True, separators=(",", ":")).encode()
            ).decode().rstrip("=")
            print(f"{MARKER}{encoded}")
            print(f"DNS change count: {record['dns_change_count']}")
            print(f"DNS change SHA-256: {record['dns_change_sha256']}")
        else:
            if not args.repo or not args.source_revision:
                raise ValueError("repository and release source are required")
            verify(plan, state, args.tree, args.mode, args.repo, args.source_revision)
    except (ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"DNS plan review gate: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
