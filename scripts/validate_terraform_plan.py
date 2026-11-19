#!/usr/bin/env python3
"""Reject destructive runtime Terraform plans before CI or release applies them."""

import argparse
import json
import sys


RUNTIME_INSTANCE_ADDRESSES = {
    "module.compute.oci_core_instance.runtime",
    "module.compute.oci_core_instance.runtime[0]",
}
REPLACEMENT_ACTIONS = {("delete", "create"), ("create", "delete")}
NON_DESTRUCTIVE_ACTIONS = {
    ("no-op",),
    ("create",),
    ("update",),
    ("read",),
}


def validate(plan: object, allow_runtime_replacement: bool = False) -> list[str]:
    if not isinstance(plan, dict) or not isinstance(plan.get("format_version"), str):
        return ["Terraform plan JSON has no format_version"]
    changes = plan.get("resource_changes", [])
    if not isinstance(changes, list):
        return ["Terraform plan JSON has an invalid resource_changes array"]

    failures = []
    for resource in changes:
        if not isinstance(resource, dict):
            failures.append("Terraform plan contains an invalid resource change")
            continue
        mode = resource.get("mode")
        if mode == "data":
            continue
        if mode != "managed":
            failures.append("Terraform plan contains a resource change with an unknown mode")
            continue
        address = resource.get("address")
        change = resource.get("change")
        actions = change.get("actions") if isinstance(change, dict) else None
        if not isinstance(address, str) or not isinstance(actions, list):
            failures.append("Terraform plan contains a resource change without address/actions")
            continue
        action_tuple = tuple(actions)
        if action_tuple in NON_DESTRUCTIVE_ACTIONS:
            continue
        if (
            allow_runtime_replacement
            and address in RUNTIME_INSTANCE_ADDRESSES
            and action_tuple in REPLACEMENT_ACTIONS
        ):
            continue
        failures.append(f"{address}: {', '.join(str(action) for action in actions)}")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--allow-runtime-replacement", action="store_true",
        help="Allow only the runtime VM replacement requested by an explicit retry",
    )
    args = parser.parse_args()
    try:
        plan = json.load(sys.stdin)
    except json.JSONDecodeError as error:
        print(f"Invalid Terraform plan JSON: {error}", file=sys.stderr)
        return 1
    failures = validate(plan, args.allow_runtime_replacement)
    if failures:
        print("Terraform plan contains prohibited managed-resource actions:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        return 1
    print("Terraform managed-resource actions are permitted.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
