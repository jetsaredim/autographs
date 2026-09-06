#!/usr/bin/env python3
"""Validate PR and commit subjects used by release-please."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


CONVENTIONAL_SUBJECT = re.compile(
    r"^(?P<type>[a-z][a-z0-9-]*)"
    r"(?:\((?P<scope>[A-Za-z0-9][A-Za-z0-9._/-]*)\))?"
    r"(?P<breaking>!)?: (?P<description>\S.*)$"
)
GIT_OBJECT_ID = re.compile(r"^[0-9a-fA-F]{40}(?:[0-9a-fA-F]{24})?$")


def load_release_types(config_path: Path) -> tuple[str, ...]:
    """Return the Conventional Commit types configured for the root package."""
    try:
        config = json.loads(config_path.read_text(encoding="utf-8"))
        sections = config["packages"]["."]["changelog-sections"]
        release_types = tuple(
            dict.fromkeys(
                section["type"]
                for section in sections
                if isinstance(section, dict)
                and isinstance(section.get("type"), str)
                and section["type"]
            )
        )
    except (OSError, json.JSONDecodeError, KeyError, TypeError) as error:
        raise ValueError(f"cannot read release types from {config_path}: {error}") from error

    if not release_types:
        raise ValueError(f"no changelog section types configured in {config_path}")
    return release_types


def validate_object_id(value: str, name: str) -> None:
    if GIT_OBJECT_ID.fullmatch(value) is None:
        raise ValueError(f"{name} must be a 40- or 64-character hexadecimal Git object ID")


def validate_subject(subject: str, release_types: tuple[str, ...]) -> str | None:
    """Return an error for a subject release-please cannot classify, if any."""
    match = CONVENTIONAL_SUBJECT.fullmatch(subject)
    if match is None:
        return "must match <type>(optional-scope)[!]: <description>"
    if match.group("type") not in release_types:
        allowed = ", ".join(release_types)
        return f"type '{match.group('type')}' is not configured; allowed types: {allowed}"
    return None


def load_pull_request_title(event_path: Path) -> str:
    try:
        event = json.loads(event_path.read_text(encoding="utf-8"))
        title = event["pull_request"]["title"]
    except (OSError, json.JSONDecodeError, KeyError, TypeError) as error:
        raise ValueError(f"cannot read pull request title from {event_path}: {error}") from error
    if not isinstance(title, str) or not title:
        raise ValueError(f"pull request title in {event_path} is empty or invalid")
    return title


def run_git(repository: Path, *arguments: str) -> str:
    try:
        result = subprocess.run(
            ["git", *arguments],
            cwd=repository,
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        detail = getattr(error, "stderr", "") or str(error)
        raise ValueError(f"git {' '.join(arguments)} failed: {detail.strip()}") from error
    return result.stdout


def pull_request_commits(
    repository: Path, base: str, head: str
) -> tuple[list[tuple[str, str]], list[str]]:
    """Return non-merge commit subjects and skipped merge commit SHAs."""
    revision_lines = run_git(repository, "rev-list", "--reverse", "--parents", f"{base}..{head}")
    commits: list[tuple[str, str]] = []
    skipped_merges: list[str] = []
    for line in revision_lines.splitlines():
        fields = line.split()
        if not fields:
            continue
        commit = fields[0]
        if len(fields) > 2:
            skipped_merges.append(commit)
            continue
        subject = run_git(repository, "show", "-s", "--format=%s", commit).rstrip("\n")
        commits.append((commit, subject))
    return commits, skipped_merges


def github_escape(message: str) -> str:
    return message.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    title_source = parser.add_mutually_exclusive_group(required=True)
    title_source.add_argument("--event", type=Path, help="GitHub pull_request event JSON")
    title_source.add_argument("--title", help="Pull request title (for local use)")
    parser.add_argument("--base", required=True, help="Pull request base SHA")
    parser.add_argument("--head", required=True, help="Pull request head SHA")
    parser.add_argument("--config", type=Path, default=Path("release-please-config.json"))
    parser.add_argument("--repository", type=Path, default=Path("."))
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        release_types = load_release_types(args.config)
        title = args.title if args.title is not None else load_pull_request_title(args.event)
        validate_object_id(args.base, "base")
        validate_object_id(args.head, "head")
        commits, skipped_merges = pull_request_commits(args.repository, args.base, args.head)
    except ValueError as error:
        print(f"::error title=Release input validation failed::{github_escape(str(error))}")
        return 2

    failures: list[str] = []
    title_error = validate_subject(title, release_types)
    if title_error:
        failures.append(f"PR title {title!r} {title_error}")

    for commit, subject in commits:
        subject_error = validate_subject(subject, release_types)
        if subject_error:
            failures.append(f"commit {commit[:12]} subject {subject!r} {subject_error}")

    if failures:
        for failure in failures:
            print(f"::error title=Invalid Conventional Commit::{github_escape(failure)}")
        print("\nUse a release-please type and this format:")
        print("  feat(catalog): add signer filters")
        print("  fix!: reject private media URLs")
        return 1

    print(f"Validated PR title and {len(commits)} non-merge commit subject(s).")
    if skipped_merges:
        print(f"Skipped {len(skipped_merges)} merge commit(s).")
    print(f"Configured release-please types: {', '.join(release_types)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
