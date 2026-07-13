#!/usr/bin/env python3

import re
import sys
from pathlib import Path


ACTION_PATH = Path(".github/actions/resolve-runtime-ip/action.yml")
TERRAFORM_VERSIONS_PATH = Path("infra/terraform/versions.tf")


def main() -> int:
    required_version = read_terraform_required_version()
    minimum_version = parse_minimum_version(required_version)
    action_version = read_action_default_version()

    if compare_versions(action_version, minimum_version) < 0:
        print(
            f"{ACTION_PATH} defaults terraform-version to {action_version}, "
            f"but {TERRAFORM_VERSIONS_PATH} requires >= {minimum_version}.",
            file=sys.stderr,
        )
        return 1

    print(
        f"resolve-runtime-ip Terraform default {action_version} satisfies "
        f"infra/terraform minimum {minimum_version}."
    )
    return 0


def read_terraform_required_version() -> str:
    content = TERRAFORM_VERSIONS_PATH.read_text(encoding="utf-8")
    match = re.search(r'required_version\s*=\s*"([^"]+)"', content)
    if not match:
        raise RuntimeError(f"Could not find required_version in {TERRAFORM_VERSIONS_PATH}")
    return match.group(1)


def parse_minimum_version(required_version: str) -> str:
    match = re.search(r">=\s*([0-9]+(?:\.[0-9]+){1,2})", required_version)
    if not match:
        raise RuntimeError(f"Could not find a >= lower bound in required_version {required_version!r}")
    return match.group(1)


def read_action_default_version() -> str:
    lines = ACTION_PATH.read_text(encoding="utf-8").splitlines()
    in_terraform_version = False

    for line in lines:
        if re.match(r"^\s{2}terraform-version:\s*$", line):
            in_terraform_version = True
            continue

        if in_terraform_version:
            if re.match(r"^\s{2}[-_a-zA-Z0-9]+:\s*$", line):
                break
            match = re.match(r'^\s{4}default:\s*"?([^"\s]+)"?\s*$', line)
            if match:
                return match.group(1)

    raise RuntimeError(f"Could not find inputs.terraform-version.default in {ACTION_PATH}")


def compare_versions(left: str, right: str) -> int:
    left_parts = version_parts(left)
    right_parts = version_parts(right)
    return (left_parts > right_parts) - (left_parts < right_parts)


def version_parts(version: str) -> tuple[int, int, int]:
    match = re.fullmatch(r"([0-9]+)(?:\.([0-9]+))?(?:\.([0-9]+))?", version)
    if not match:
        raise RuntimeError(f"Unsupported Terraform version format: {version!r}")
    return tuple(int(part or "0") for part in match.groups())


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"Terraform version alignment validation failed: {error}", file=sys.stderr)
        raise SystemExit(1)
