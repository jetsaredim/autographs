#!/usr/bin/env python3

from pathlib import Path
import sys


DENIED_PHRASES = (
    "active Next.js runtime",
    "current Next.js runtime",
    "/api/operator/catalog current",
    "Phase 8 AI-assisted ingest",
    "Phase 8 taxonomy media cues",
)

MOVE_NOTES = ("moved to Phase 9", "moved to Phase 10")
PHASE_8_OWNER_TERMS = ("admin media", "operational posture", "security patching")


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def scan_paths(root: Path) -> list[Path]:
    paths: list[Path] = []
    for relative in ("README.md", "AGENTS.md"):
        path = root / relative
        if path.exists():
            paths.append(path)
    for directory in (root / "docs", root / ".planning" / "codebase"):
        if directory.exists():
            paths.extend(path for path in directory.rglob("*.md") if path.is_file())
    return sorted(paths)


def allowed_denied_phrase(line: str, phrase: str) -> bool:
    normalized = line.lower()
    if phrase.startswith("Phase 8 "):
        return any(note.lower() in normalized for note in MOVE_NOTES)
    if phrase == "active Next.js runtime":
        return "former active next.js runtime" in normalized and "retired" in normalized
    return False


def validate_retired_runtime_claims(root: Path) -> list[str]:
    violations: list[str] = []
    for path in scan_paths(root):
        relative = path.relative_to(root)
        for line_number, line in enumerate(read_text(path).splitlines(), start=1):
            normalized = line.lower()
            for phrase in DENIED_PHRASES:
                if phrase.lower() in normalized and not allowed_denied_phrase(line, phrase):
                    violations.append(f"{relative}:{line_number}: denied current-state claim `{phrase}`")
    return violations


def validate_phase_ownership(root: Path) -> list[str]:
    violations: list[str] = []
    codebase_dir = root / ".planning" / "codebase"
    if not codebase_dir.exists():
        return [".planning/codebase: codebase map directory is missing"]
    map_paths = sorted(path for path in codebase_dir.glob("*.md") if path.is_file())
    if not map_paths:
        return [".planning/codebase: no codebase map markdown files found"]
    for path in map_paths:
        relative = path.relative_to(root)
        text = read_text(path)
        lowered = text.lower()
        if "phase 8" not in lowered:
            violations.append(f"{relative}: missing `Phase 8` current-state ownership")
        if not any(term in lowered for term in PHASE_8_OWNER_TERMS):
            violations.append(
                f"{relative}: missing Phase 8 admin media, operational posture, or security patching ownership"
            )
    return violations


def validate_codebase_maps(root: Path) -> list[str]:
    return validate_phase_ownership(root)


def collect_violations(root: Path | None = None) -> list[str]:
    base = root or repo_root()
    violations: list[str] = []
    violations.extend(validate_retired_runtime_claims(base))
    violations.extend(validate_codebase_maps(base))
    return violations


def main() -> int:
    violations = collect_violations()
    if violations:
        print("Repository hygiene validation failed:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        return 1
    print("Repository hygiene validation passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
