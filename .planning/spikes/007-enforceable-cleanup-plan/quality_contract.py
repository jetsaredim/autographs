#!/usr/bin/env python3
"""Prototype enforceable controller contracts and emit redacted findings."""

from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path
import re
from typing import Any


ENV_READ = re.compile(r"\b(?:std::)?env::(?:var|var_os)\s*\(")
FORBIDDEN_MACRO = re.compile(r"\b(todo|unimplemented|dbg)!\s*\(")
RAW_STRING = re.compile(r'r(?P<hashes>#{0,8})"(?P<body>.*?)"(?P=hashes)', re.DOTALL)
NORMAL_STRING = re.compile(r'"(?:\\.|[^"\\])*"', re.DOTALL)
NUMERIC_BIND = re.compile(r"(?<!:):([1-9][0-9]*)\b")
SQL_START = re.compile(r"\b(?:select|insert|update|delete|merge|with)\b", re.IGNORECASE)
ENV_ASSIGNMENT = re.compile(r"^(?P<name>[A-Z][A-Z0-9_]*)=(?P<value>.*)$")
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
SECRET_REFERENCE_SUFFIXES = ("_SECRET_OCID",)


def is_secret_value_name(name: str) -> bool:
    """Match secret-bearing values while excluding non-secret Vault coordinates."""
    return not name.endswith(SECRET_REFERENCE_SUFFIXES) and any(
        term in name for term in SECRET_TERMS
    )


def rust_strings(text: str) -> list[str]:
    """Extract enough Rust string syntax to inspect SQL bind contracts."""
    strings = [match.group("body") for match in RAW_STRING.finditer(text)]
    without_raw = RAW_STRING.sub("", text)
    for match in NORMAL_STRING.finditer(without_raw):
        strings.append(match.group(0)[1:-1])
    return strings


def inspect_rust(path: Path, relative: str) -> list[dict[str, Any]]:
    text = path.read_text(encoding="utf-8")
    findings: list[dict[str, Any]] = []

    env_reads = len(ENV_READ.findall(text))
    if env_reads and relative != "controller/src/config.rs":
        findings.append(
            {
                "rule": "distributed-env-read",
                "level": "block",
                "path": relative,
                "count": env_reads,
            }
        )

    forbidden = Counter(FORBIDDEN_MACRO.findall(text))
    for macro, count in sorted(forbidden.items()):
        findings.append(
            {
                "rule": "production-placeholder-macro",
                "level": "block",
                "path": relative,
                "macro": macro,
                "count": count,
            }
        )

    unsafe_numeric_bind_statements = 0
    for literal in rust_strings(text):
        if not SQL_START.search(literal):
            continue
        binds = NUMERIC_BIND.findall(literal)
        expected = list(range(1, len(binds) + 1))
        if [int(bind) for bind in binds] != expected:
            unsafe_numeric_bind_statements += 1
    if unsafe_numeric_bind_statements:
        findings.append(
            {
                "rule": "unsafe-numeric-sql-bind-order",
                "level": "block",
                "path": relative,
                "count": unsafe_numeric_bind_statements,
            }
        )

    return findings


def inspect_env_template(path: Path, relative: str) -> list[dict[str, Any]]:
    findings = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        match = ENV_ASSIGNMENT.match(line.strip())
        if not match or not is_secret_value_name(match.group("name")):
            continue
        findings.append(
            {
                "rule": "persistent-secret-env-sink",
                "level": "block",
                "path": relative,
                "line": line_number,
                "variable": match.group("name"),
            }
        )
    return findings


def inspect(root: Path) -> dict[str, Any]:
    findings: list[dict[str, Any]] = []
    source_root = root / "controller" / "src"
    for path in sorted(source_root.rglob("*.rs")):
        relative = path.relative_to(root).as_posix()
        findings.extend(inspect_rust(path, relative))
    deploy_root = root / "deploy"
    if deploy_root.exists():
        for path in sorted(deploy_root.rglob("*.env.j2")):
            relative = path.relative_to(root).as_posix()
            findings.extend(inspect_env_template(path, relative))

    by_rule = Counter(finding["rule"] for finding in findings)
    return {
        "schema_version": 1,
        "finding_count": len(findings),
        "by_rule": dict(sorted(by_rule.items())),
        "findings": findings,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    report = inspect(args.root.resolve())
    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(encoded, encoding="utf-8")
    else:
        print(encoded, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
