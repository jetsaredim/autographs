#!/usr/bin/env python3
"""Collect redacted Autographs repository and VM configuration inventories."""

from __future__ import annotations

import argparse
import datetime as dt
import grp
from itertools import combinations
import json
import os
from pathlib import Path
import pwd
import re
import stat
import subprocess
from typing import Any, Iterable


SCHEMA_VERSION = 2
KEY_PATTERN = re.compile(
    r"\b(?:AUTOGRAPHS|OCI|ORACLE_DB|TF_VAR|DEPLOY|GHCR)_[A-Z0-9_]+\b"
    r"|\b(?:DATABASE_URL|TNS_ADMIN|RUST_LOG|VM_PUBLIC_IP)\b"
)
WORKFLOW_REFERENCE_PATTERN = re.compile(r"\b(?:secrets|vars)\.([A-Z][A-Z0-9_]+)\b")
ASSIGNMENT_PATTERN = re.compile(r"^\s*(?:export\s+)?([A-Z][A-Z0-9_]+)\s*=")
SECRET_TERMS = (
    "PASSWORD",
    "TOKEN",
    "PRIVATE_KEY_PEM",
    "SSH_PRIVATE_KEY",
    "SECRET_KEY",
    "WALLET_PEM",
    "WALLET_ZIP",
    "API_KEY",
)
CONTRACT_ROLES = {
    "canonical-contract",
    "container-build",
    "deployment-contract",
    "example-contract",
    "github-workflow",
    "persistent-env-template",
    "terraform",
}
BOUNDARY_ORDER = (
    "vm-runtime",
    "vm-smoke",
    "build",
    "deployment",
    "ci",
    "maintenance",
    "infrastructure",
    "local-test",
    "documentation-only",
    "repository-other",
)
BOUNDARY_LABELS = {
    "vm-runtime": "VM Runtime",
    "vm-smoke": "VM Smoke",
    "build": "Build",
    "deployment": "Deployment",
    "ci": "CI",
    "maintenance": "Maintenance",
    "infrastructure": "Infrastructure",
    "local-test": "Local/Test",
    "documentation-only": "Documentation Only",
    "repository-other": "Repository Other",
}
SENSITIVE_TERMS = (
    "PASSWORD_HASH",
    "FINGERPRINT",
    "PRIVATE_KEY_PATH",
    "WALLET_DIR",
    "CONNECT_STRING",
    "TENANCY_OCID",
    "USER_OCID",
)
SKIP_PARTS = {".git", ".terraform", "node_modules", "target"}
GENERATED_INVENTORY_NAMES = {
    "autographs-ecosystem-comparison.md",
    "autographs-repository-inventory.json",
    "autographs-vm-inventory.json",
}
TEXT_SUFFIXES = {
    ".example",
    ".j2",
    ".md",
    ".py",
    ".rs",
    ".sh",
    ".tf",
    ".toml",
    ".yml",
    ".yaml",
}


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def classify_key(name: str) -> str:
    if any(term in name for term in SECRET_TERMS):
        return "secret-scalar"
    if any(term in name for term in SENSITIVE_TERMS):
        return "sensitive-metadata"
    return "public-config"


def source_role(path: Path) -> str:
    text = path.as_posix()
    if text.endswith(("app.env.j2", "controller.env.j2")):
        return "persistent-env-template"
    if text.endswith((".env.example", ".env.github.example")):
        return "example-contract"
    if text == "docs/configuration-contract.md":
        return "canonical-contract"
    if text.startswith("controller/src/") or text.startswith("controller/tests/"):
        return "rust"
    if text.startswith(".github/workflows/"):
        return "github-workflow"
    if text.startswith("deploy/") and any(
        part in path.parts for part in ("defaults", "tasks", "templates", "vars")
    ):
        return "deployment-contract"
    if text.startswith("deploy/"):
        return "deployment"
    if text.startswith("infra/terraform/"):
        return "terraform"
    if path.name.startswith("Dockerfile"):
        return "container-build"
    if text.startswith("docs/") or text.startswith(".planning/"):
        return "documentation"
    return "repository"


def source_boundaries(path: Path) -> set[str]:
    """Classify a source by where its configuration is consumed."""
    text = path.as_posix()
    if text.endswith(("app.env.j2", "controller.env.j2")):
        return {"vm-runtime", "deployment"}
    if text.startswith("controller/src/"):
        return {"vm-runtime"}
    if text.startswith("controller/tests/live"):
        return {"vm-smoke"}
    if text.startswith("controller/tests/"):
        return {"local-test"}
    if text == ".github/.env.github.example":
        return {"deployment"}
    if text == ".env.example":
        return {"local-test"}
    if text.startswith(".github/workflows/"):
        name = path.name
        if name == "deploy.yml":
            return {"deployment"}
        if name == "ci.yml":
            return {"ci"}
        if name in {
            "apply-security-updates.yml",
            "image-cleanup.yml",
            "reboot-security-runtime.yml",
            "weekly-security-scan.yml",
        }:
            return {"maintenance"}
        return {"build"}
    if text.startswith("deploy/"):
        return {"deployment"}
    if text.startswith("infra/terraform/"):
        return {"infrastructure"}
    if text.startswith("scripts/"):
        if path.name.startswith("test_"):
            return {"local-test"}
        if any(term in path.name for term in ("cleanup", "oscap", "advisory")):
            return {"maintenance"}
        return {"ci"}
    if path.name.startswith("Dockerfile"):
        return {"build"}
    if text.startswith("docs/") or text.startswith(".planning/"):
        return {"documentation"}
    return {"repository-other"}


def normalized_boundaries(boundaries: set[str]) -> list[str]:
    boundaries.discard("documentation")
    if len(boundaries) > 1:
        boundaries.discard("documentation-only")
        boundaries.discard("repository-other")
    if not boundaries:
        boundaries.add("documentation-only")
    return sorted(boundaries, key=boundary_sort_key)


def boundary_sort_key(boundary: str) -> tuple[int, str]:
    try:
        return (BOUNDARY_ORDER.index(boundary), boundary)
    except ValueError:
        return (len(BOUNDARY_ORDER), boundary)


def entry_boundaries(entry: dict[str, Any], name: str | None = None) -> list[str]:
    if name and name.startswith("AUTOGRAPHS_TEST_"):
        return ["local-test"]
    explicit = entry.get("boundaries")
    if explicit:
        boundaries = set(explicit)
        add_semantic_boundaries(name, boundaries)
        return normalized_boundaries(boundaries)
    boundaries: set[str] = set()
    for source in entry.get("sources", []):
        source_explicit = source.get("boundaries")
        if source_explicit:
            boundaries.update(source_explicit)
        elif source.get("path"):
            boundaries.update(source_boundaries(Path(source["path"])))
    add_semantic_boundaries(name, boundaries)
    return normalized_boundaries(boundaries)


def add_semantic_boundaries(name: str | None, boundaries: set[str]) -> None:
    """Add project contract boundaries that are mediated through workflow names."""
    if not name:
        return
    infrastructure_prefixes = (
        "AUTOGRAPHS_DNS_",
        "OCI_AUTONOMOUS_DATABASE_",
        "OCI_CREATE_",
        "OCI_RUNTIME_",
    )
    infrastructure_names = {
        "ADB_ADMIN_PASSWORD",
        "OCI_AVAILABILITY_DOMAIN",
        "OCI_COMPARTMENT_OCID",
        "OCI_HOME_REGION",
        "OCI_OBJECT_STORAGE_NAMESPACE",
        "VM_PUBLIC_IP",
    }
    if name.startswith(infrastructure_prefixes) or name in infrastructure_names:
        boundaries.add("infrastructure")


def keys_by_boundary(
    variables: dict[str, dict[str, Any]], include: set[str] | None = None
) -> dict[str, list[str]]:
    grouped: dict[str, list[str]] = {}
    for name, entry in variables.items():
        if include is not None and name not in include:
            continue
        for boundary in entry_boundaries(entry, name):
            grouped.setdefault(boundary, []).append(name)
    return {
        boundary: sorted(names)
        for boundary, names in sorted(
            grouped.items(), key=lambda item: boundary_sort_key(item[0])
        )
    }


def is_text_candidate(path: Path) -> bool:
    if any(part in SKIP_PARTS for part in path.parts):
        return False
    if path.name in GENERATED_INVENTORY_NAMES:
        return False
    if path.name.startswith("Dockerfile") or path.name in {"Caddyfile", "AGENTS.md"}:
        return True
    return path.suffix in TEXT_SUFFIXES or path.name.startswith(".env")


def read_lines(path: Path) -> list[str]:
    try:
        return path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeDecodeError):
        return []


def collect_repo(root: Path) -> dict[str, Any]:
    variables: dict[str, dict[str, Any]] = {}
    scanned_files = 0
    for current_root, directories, filenames in os.walk(root):
        current_relative = Path(current_root).relative_to(root)
        if current_relative.parts[:2] == (".planning", "spikes"):
            directories[:] = []
            continue
        directories[:] = sorted(name for name in directories if name not in SKIP_PARTS)
        for filename in sorted(filenames):
            path = Path(current_root) / filename
            relative = path.relative_to(root)
            if not is_text_candidate(relative):
                continue
            lines = read_lines(path)
            if not lines:
                continue
            scanned_files += 1
            role = source_role(relative)
            for line_number, line in enumerate(lines, start=1):
                names = set(KEY_PATTERN.findall(line))
                names.update(WORKFLOW_REFERENCE_PATTERN.findall(line))
                assignment = ASSIGNMENT_PATTERN.match(line)
                if assignment:
                    names.add(assignment.group(1))
                for name in sorted(names):
                    if name.startswith(("GITHUB_", "RUNNER_")):
                        continue
                    entry = variables.setdefault(
                        name,
                        {"classification": classify_key(name), "sources": []},
                    )
                    source = {
                        "path": relative.as_posix(),
                        "line": line_number,
                        "role": role,
                    }
                    if source not in entry["sources"]:
                        entry["sources"].append(source)

    findings: list[dict[str, Any]] = []
    for name, entry in sorted(variables.items()):
        roles = {source["role"] for source in entry["sources"]}
        if entry["classification"] == "secret-scalar" and "persistent-env-template" in roles:
            findings.append(
                finding(
                    "persistent-plaintext-secret-contract",
                    "high",
                    name,
                    "A production env template materializes this secret on persistent storage.",
                    "Move the value to a runtime secret provider or document a narrow exception.",
                )
            )
        if "rust" in roles and not roles.intersection(
            {
                "deployment-contract",
                "example-contract",
                "github-workflow",
                "persistent-env-template",
            }
        ):
            findings.append(
                finding(
                    "undocumented-rust-config",
                    "medium",
                    name,
                    "Rust references this key without a matching example, deploy, or workflow contract.",
                    "Declare the key in the canonical configuration contract or remove the reference.",
                )
            )

    for name, entry in variables.items():
        entry["boundaries"] = entry_boundaries(entry, name)

    contract_keys = sorted(
        name
        for name, entry in variables.items()
        if any(source["role"] in CONTRACT_ROLES for source in entry["sources"])
    )
    incidental_keys = sorted(set(variables) - set(contract_keys))

    return {
        "schema_version": SCHEMA_VERSION,
        "kind": "repository",
        "generated_at": utc_now(),
        "redaction_contract": "Variable names and source locations only; values are never emitted.",
        "root": root.name,
        "scanned_files": scanned_files,
        "variables": dict(sorted(variables.items())),
        "keys_by_boundary": keys_by_boundary(variables),
        "contract_keys_by_boundary": keys_by_boundary(variables, set(contract_keys)),
        "contract_keys": contract_keys,
        "incidental_keys": incidental_keys,
        "findings": findings,
    }


def finding(kind: str, severity: str, subject: str, evidence: str, disposition: str) -> dict[str, str]:
    return {
        "kind": kind,
        "severity": severity,
        "subject": subject,
        "evidence": evidence,
        "recommended_disposition": disposition,
    }


def owner_name(uid: int) -> str:
    try:
        return pwd.getpwuid(uid).pw_name
    except KeyError:
        return str(uid)


def group_name(gid: int) -> str:
    try:
        return grp.getgrgid(gid).gr_name
    except KeyError:
        return str(gid)


def metadata(path: Path, base: Path | None = None) -> dict[str, Any]:
    info = path.lstat()
    return {
        "path": path.relative_to(base).as_posix() if base else path.as_posix(),
        "type": "symlink" if path.is_symlink() else "directory" if path.is_dir() else "file",
        "mode": stat.filemode(info.st_mode),
        "owner": owner_name(info.st_uid),
        "group": group_name(info.st_gid),
        "size_bytes": info.st_size,
        "modified_at": dt.datetime.fromtimestamp(info.st_mtime, dt.timezone.utc).isoformat(),
    }


def env_file_metadata(path: Path, base: Path) -> dict[str, Any]:
    result = metadata(path, base)
    keys = []
    if path.is_file() and not path.is_symlink():
        for line in read_lines(path):
            match = ASSIGNMENT_PATTERN.match(line)
            if match:
                keys.append(match.group(1))
    result["keys"] = sorted(set(keys))
    result["classifications"] = {
        classification: sum(classify_key(key) == classification for key in result["keys"])
        for classification in ("public-config", "sensitive-metadata", "secret-scalar")
    }
    result["boundary"] = env_file_boundary(result["path"])
    return result


def env_file_boundary(path: str) -> str:
    name = Path(path).name.lower()
    if name in {"app.env", "controller.env"}:
        return "vm-runtime"
    if "smoke" in name or "spike" in name:
        return "vm-smoke"
    return "vm-other"


def safe_children(root: Path, depth: int = 1) -> list[dict[str, Any]]:
    if not root.exists():
        return []
    output = [metadata(root)]
    current: Iterable[Path] = root.iterdir()
    for path in sorted(current):
        output.append(metadata(path))
        if depth > 1 and path.is_dir() and not path.is_symlink():
            for child in sorted(path.iterdir()):
                output.append(metadata(child))
    return output


def run_listing(command: list[str]) -> dict[str, Any]:
    try:
        result = subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        return {"command": command[0], "available": False, "error": type(error).__name__}
    return {
        "command": command[0],
        "available": result.returncode == 0,
        "return_code": result.returncode,
        "lines": [line for line in result.stdout.splitlines() if line.strip()],
    }


def collect_vm(opt_root: Path, var_root: Path, quadlet_root: Path, tmp_root: Path) -> dict[str, Any]:
    env_root = opt_root / "env"
    env_files = []
    if env_root.exists():
        env_files = [
            env_file_metadata(path, opt_root)
            for path in sorted(env_root.iterdir())
            if path.is_file() or path.is_symlink()
        ]

    findings: list[dict[str, Any]] = []
    for entry in env_files:
        if entry["classifications"]["secret-scalar"]:
            findings.append(
                finding(
                    "persistent-plaintext-secret-file",
                    "high",
                    entry["path"],
                    f"File declares {entry['classifications']['secret-scalar']} secret-like key(s).",
                    "Move secret values to OCI Vault and retain only non-secret configuration.",
                )
            )
        mode = entry["mode"]
        if len(mode) == 10 and any(char != "-" for char in mode[4:]):
            findings.append(
                finding(
                    "broad-config-file-permissions",
                    "high",
                    entry["path"],
                    f"File permissions are {mode}.",
                    "Restrict sensitive runtime files to the service owner.",
                )
            )

    tmp_entries = []
    if tmp_root.exists():
        tmp_entries = [metadata(path) for path in sorted(tmp_root.glob("*autographs*"))]

    podman = {
        "containers": run_listing(["podman", "ps", "-a", "--format", "{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}"]),
        "images": run_listing(["podman", "images", "--format", "{{.ID}}\t{{.Repository}}\t{{.Tag}}\t{{.CreatedAt}}\t{{.Size}}"]),
        "volumes": run_listing(["podman", "volume", "ls", "--format", "{{.Name}}\t{{.Driver}}"]),
        "networks": run_listing(["podman", "network", "ls", "--format", "{{.Name}}\t{{.Driver}}"]),
        "secrets": run_listing(["podman", "secret", "ls", "--format", "{{.ID}}\t{{.Name}}\t{{.Driver}}\t{{.CreatedAt}}"]),
    }
    systemd = {
        "unit_files": run_listing(["systemctl", "list-unit-files", "autographs*", "--no-legend", "--no-pager"]),
        "units": run_listing(["systemctl", "list-units", "autographs*", "--all", "--no-legend", "--no-pager"]),
    }

    return {
        "schema_version": SCHEMA_VERSION,
        "kind": "vm",
        "generated_at": utc_now(),
        "redaction_contract": "File metadata, env key names, and bounded runtime listing fields only; values and inspect payloads are never emitted.",
        "env_files": env_files,
        "wallet_metadata": safe_children(opt_root / "wallet", depth=1),
        "secret_file_metadata": safe_children(opt_root / "secrets", depth=1),
        "quadlet_metadata": safe_children(quadlet_root, depth=1),
        "static_metadata": safe_children(var_root / "static", depth=1),
        "temporary_metadata": tmp_entries,
        "podman": podman,
        "systemd": systemd,
        "findings": findings,
    }


def render_comparison(repo: dict[str, Any], vm: dict[str, Any]) -> str:
    variables = repo.get("variables", {})
    contract_keys = set(repo.get("contract_keys", []))
    if "contract_keys" not in repo:
        contract_keys = {
            name
            for name, entry in variables.items()
            if any(
                source.get("role") in CONTRACT_ROLES
                for source in entry.get("sources", [])
            )
        }
    incidental_keys = set(repo.get("incidental_keys", set(variables) - contract_keys))
    vm_files = vm.get("env_files", [])
    for entry in vm_files:
        entry.setdefault("boundary", env_file_boundary(entry.get("path", "")))
    vm_keys = {key for entry in vm_files for key in entry.get("keys", [])}
    runtime_files = [entry for entry in vm_files if entry["boundary"] == "vm-runtime"]
    smoke_files = [entry for entry in vm_files if entry["boundary"] == "vm-smoke"]
    runtime_vm_keys = {key for entry in runtime_files for key in entry.get("keys", [])}
    smoke_vm_keys = {key for entry in smoke_files for key in entry.get("keys", [])}

    runtime_template_keys = {
        name
        for name, entry in variables.items()
        if any(
            source.get("role") == "persistent-env-template"
            for source in entry.get("sources", [])
        )
    }
    runtime_consumer_keys = {
        name
        for name, entry in variables.items()
        if any(
            source.get("path", "").startswith("controller/src/")
            for source in entry.get("sources", [])
        )
        and not name.startswith("AUTOGRAPHS_TEST_")
    }
    runtime_known_keys = runtime_template_keys | runtime_consumer_keys

    visible_boundary_keys = contract_keys | runtime_consumer_keys
    visible_boundary_keys.update(
        name
        for name, entry in variables.items()
        if "vm-smoke" in entry_boundaries(entry, name)
    )
    repo_boundaries = keys_by_boundary(variables, visible_boundary_keys)
    smoke_repo_keys = set(repo_boundaries.get("vm-smoke", []))

    unknown_runtime_keys = sorted(runtime_vm_keys - runtime_known_keys)
    missing_template_keys = sorted(runtime_template_keys - runtime_vm_keys)
    defaulted_runtime_keys = sorted(
        runtime_consumer_keys - runtime_vm_keys - runtime_template_keys
    )
    unknown_smoke_keys = sorted(
        smoke_vm_keys - smoke_repo_keys - runtime_consumer_keys - runtime_template_keys
    )
    absent_smoke_keys = sorted(smoke_repo_keys - smoke_vm_keys)

    lines = [
        "# Ecosystem Inventory Comparison",
        "",
        f"- Repository declared contract keys: {len(contract_keys)}",
        f"- Repository incidental/historical mentions: {len(incidental_keys)}",
        f"- VM env keys: {len(vm_keys)}",
        f"- VM env files: {len(vm_files)}",
        "",
        "## Repository Variables By Boundary",
        "",
    ]
    lines.extend(
        [
            "Variables may appear in multiple boundaries when they are handed from one system to another.",
            "Only the VM Runtime and VM Smoke boundaries participate in VM env comparisons.",
            "",
            "| Boundary | Keys |",
            "|----------|-----:|",
        ]
    )
    for boundary, names in repo_boundaries.items():
        lines.append(f"| {BOUNDARY_LABELS.get(boundary, boundary)} | {len(names)} |")
    for boundary, names in repo_boundaries.items():
        lines.extend(["", f"### {BOUNDARY_LABELS.get(boundary, boundary)}", ""])
        append_key_list(lines, names)

    lines.extend(["", "## Persistent Secret-Like Keys By VM Boundary", ""])
    for boundary in ("vm-runtime", "vm-smoke", "vm-other"):
        matching = [entry for entry in vm_files if entry["boundary"] == boundary]
        secret_locations: dict[str, list[str]] = {}
        for entry in matching:
            for key in entry.get("keys", []):
                if classify_key(key) == "secret-scalar":
                    secret_locations.setdefault(key, []).append(entry["path"])
        lines.extend([f"### {BOUNDARY_LABELS.get(boundary, 'VM Other')}", ""])
        if secret_locations:
            for key, paths in sorted(secret_locations.items()):
                rendered_paths = ", ".join(f"`{path}`" for path in sorted(paths))
                lines.append(f"- `{key}`: {rendered_paths}")
        else:
            lines.append("- None")
        lines.append("")

    lines.extend(["## VM Runtime Contract Drift", ""])
    lines.extend(
        ["### VM Runtime Keys Without a Runtime Template or Controller Consumer", ""]
    )
    append_key_list(lines, unknown_runtime_keys)
    lines.extend(
        ["", "### Deployed Runtime Template Keys Absent From Runtime Env Files", ""]
    )
    append_key_list(lines, missing_template_keys)
    lines.extend(
        [
            "",
            "### Controller Consumers Not Materialized in Runtime Env",
            "",
            "These keys are not emitted by the deployed runtime templates. They may intentionally use code defaults or be optional overrides; review them rather than adding them automatically.",
            "",
        ]
    )
    append_key_list(lines, defaulted_runtime_keys)

    lines.extend(["", "## VM Smoke Contract Review", ""])
    lines.extend(["### Smoke Env Keys Without a Repository Smoke Consumer", ""])
    append_key_list(lines, unknown_smoke_keys)
    lines.extend(
        [
            "",
            "### Repository Smoke Keys Absent From Retained Smoke Env Files",
            "",
            "These may be optional gates, command-line injections, or task-specific settings. They are not production-runtime drift.",
            "",
        ]
    )
    append_key_list(lines, absent_smoke_keys)

    lines.extend(["", "## Actual Env File Key Overlap", ""])
    overlaps = []
    for left, right in combinations(vm_files, 2):
        shared = sorted(set(left.get("keys", [])) & set(right.get("keys", [])))
        if shared:
            overlaps.append((left["path"], right["path"], shared))
    if overlaps:
        for left, right, shared in overlaps:
            rendered_keys = ", ".join(f"`{key}`" for key in shared)
            lines.append(f"- `{left}` and `{right}` ({len(shared)}): {rendered_keys}")
    else:
        lines.append("- None")

    lines.extend(["", "## Incidental or Historical Repository Mentions", ""])
    append_key_list(lines, sorted(incidental_keys))
    lines.extend(
        [
            "",
            "Runtime drift sections compare only like-for-like boundaries. External-boundary and defaulted variables remain evidence for review, not instructions to add them to VM env files.",
            "",
        ]
    )
    return "\n".join(lines)


def append_key_list(lines: list[str], keys: Iterable[str]) -> None:
    rendered = list(keys)
    if rendered:
        lines.extend(f"- `{key}`" for key in rendered)
    else:
        lines.append("- None")


def write_private_text(path: Path, content: str) -> None:
    """Create a private output without following or replacing an existing path."""
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    descriptor = os.open(path, flags, 0o600)
    try:
        os.fchmod(descriptor, 0o600)
        with os.fdopen(descriptor, "w", encoding="utf-8") as output:
            descriptor = -1
            output.write(content)
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def write_json(path: Path, payload: dict[str, Any]) -> None:
    write_private_text(path, json.dumps(payload, indent=2, sort_keys=True) + "\n")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    repo = subparsers.add_parser("repo", help="inventory repository configuration names")
    repo.add_argument("--root", type=Path, default=Path.cwd())
    repo.add_argument("--output", type=Path, required=True)

    vm = subparsers.add_parser("vm", help="inventory VM metadata without secret values")
    vm.add_argument("--opt-root", type=Path, default=Path("/opt/autographs"))
    vm.add_argument("--var-root", type=Path, default=Path("/var/lib/autographs"))
    vm.add_argument("--quadlet-root", type=Path, default=Path("/etc/containers/systemd"))
    vm.add_argument("--tmp-root", type=Path, default=Path("/tmp"))
    vm.add_argument("--output", type=Path, required=True)

    compare = subparsers.add_parser("compare", help="compare repository and VM inventories")
    compare.add_argument("--repo", type=Path, required=True)
    compare.add_argument("--vm", type=Path, required=True)
    compare.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.command == "repo":
        write_json(args.output, collect_repo(args.root.resolve()))
    elif args.command == "vm":
        write_json(
            args.output,
            collect_vm(args.opt_root, args.var_root, args.quadlet_root, args.tmp_root),
        )
    else:
        repo = json.loads(args.repo.read_text(encoding="utf-8"))
        vm = json.loads(args.vm.read_text(encoding="utf-8"))
        write_private_text(args.output, render_comparison(repo, vm))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
