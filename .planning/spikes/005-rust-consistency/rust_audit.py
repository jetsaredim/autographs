#!/usr/bin/env python3
"""Emit structural Rust consistency metrics without modifying source."""

from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path
import re
from typing import Any


PATTERNS = {
    "async_functions": re.compile(r"\basync\s+fn\b"),
    "clones": re.compile(r"\.clone\s*\("),
    "direct_env_reads": re.compile(r"\b(?:std::)?env::var\s*\("),
    "expects": re.compile(r"\.expect\s*\("),
    "panics": re.compile(r"\bpanic!\s*\("),
    "result_string": re.compile(r"Result\s*<[^\n;]+String\s*>") ,
    "spawn_blocking": re.compile(r"\bspawn_blocking\s*\("),
    "unwraps": re.compile(r"\.unwrap\s*\("),
}
SQL_CONST = re.compile(r"\bconst\s+[A-Z][A-Z0-9_]*SQL\s*:")
NAMED_BIND_CALL = re.compile(r"\.(?:query|execute)_named\s*\(")
POSITIONAL_BIND_CALL = re.compile(r"\.(?:query|execute)\s*\(")


def count_lines(text: str) -> int:
    return text.count("\n") + (0 if not text or text.endswith("\n") else 1)


def audit(root: Path) -> dict[str, Any]:
    files = []
    totals: Counter[str] = Counter()
    for area in (root / "src", root / "tests"):
        for path in sorted(area.rglob("*.rs")):
            text = path.read_text(encoding="utf-8")
            metrics = {name: len(pattern.findall(text)) for name, pattern in PATTERNS.items()}
            plain_fs_operations = len(
                re.findall(r"\bfs::(?:read|read_dir|read_link|write)\s*\(", text)
            )
            uses_tokio_fs = "use tokio::fs" in text
            uses_std_fs = bool(re.search(r"use\s+std(?:::\{[^}]*\bfs\b|::fs)", text, re.DOTALL))
            metrics["std_fs_operations"] = len(
                re.findall(r"\bstd::fs::(?:read|read_dir|read_link|write)\s*\(", text)
            ) + (plain_fs_operations if uses_std_fs else 0)
            metrics["tokio_fs_operations"] = len(re.findall(r"\btokio::fs::", text)) + (
                plain_fs_operations if uses_tokio_fs else 0
            )
            metrics.update(
                {
                    "sql_constants": len(SQL_CONST.findall(text)),
                    "named_bind_calls": len(NAMED_BIND_CALL.findall(text)),
                    "positional_bind_calls": len(POSITIONAL_BIND_CALL.findall(text)),
                }
            )
            totals.update(metrics)
            files.append(
                {
                    "path": path.relative_to(root).as_posix(),
                    "bytes": len(text.encode("utf-8")),
                    "lines": count_lines(text),
                    **metrics,
                }
            )

    source_files = [entry for entry in files if entry["path"].startswith("src/")]
    findings = []
    for entry in source_files:
        if entry["lines"] >= 1500:
            findings.append(
                {
                    "kind": "large-module",
                    "severity": "high",
                    "path": entry["path"],
                    "evidence": f"{entry['lines']} lines",
                    "disposition": "Split by existing domain responsibility during touched-feature work.",
                }
            )
        if entry["direct_env_reads"] and entry["path"] != "src/config.rs":
            findings.append(
                {
                    "kind": "distributed-configuration-read",
                    "severity": "medium",
                    "path": entry["path"],
                    "evidence": f"{entry['direct_env_reads']} direct env read(s)",
                    "disposition": "Move runtime configuration parsing into typed configuration owners.",
                }
            )
        if entry["std_fs_operations"] and entry["async_functions"] and not entry["spawn_blocking"]:
            findings.append(
                {
                    "kind": "blocking-io-review",
                    "severity": "medium",
                    "path": entry["path"],
                    "evidence": f"{entry['std_fs_operations']} std fs operation(s), {entry['async_functions']} async function(s)",
                    "disposition": "Measure call context; use spawn_blocking or async fs only where request-path blocking is demonstrated.",
                }
            )

    return {
        "schema_version": 1,
        "root": root.name,
        "source_file_count": len(source_files),
        "test_file_count": len(files) - len(source_files),
        "source_lines": sum(entry["lines"] for entry in source_files),
        "test_lines": sum(entry["lines"] for entry in files if entry["path"].startswith("tests/")),
        "totals": dict(sorted(totals.items())),
        "files": sorted(files, key=lambda entry: (-entry["lines"], entry["path"])),
        "findings": findings,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("controller"))
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    report = audit(args.root)
    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(encoded, encoding="utf-8")
    else:
        print(encoded, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
