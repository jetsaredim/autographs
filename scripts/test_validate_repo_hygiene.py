#!/usr/bin/env python3

import importlib.util
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("validate_repo_hygiene.py")
CODEBASE_MAPS = (
    "ARCHITECTURE.md",
    "CONCERNS.md",
    "CONVENTIONS.md",
    "INTEGRATIONS.md",
    "STACK.md",
    "STRUCTURE.md",
    "TESTING.md",
)
spec = importlib.util.spec_from_file_location("validate_repo_hygiene", SCRIPT_PATH)
validate_repo_hygiene = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(validate_repo_hygiene)


def write_minimal_repo(root: Path, *, docs_text: str = "") -> None:
    (root / "docs").mkdir(parents=True, exist_ok=True)
    (root / ".planning" / "codebase").mkdir(parents=True, exist_ok=True)
    (root / "README.md").write_text("# Autographs\n\nRust/static current runtime.\n", encoding="utf-8")
    (root / "AGENTS.md").write_text(
        "Phase 8 starts with production security patching repair, operational posture, "
        "and admin media review before Phase 9 and Phase 10.\n",
        encoding="utf-8",
    )
    (root / "docs" / "phase.md").write_text(docs_text, encoding="utf-8")
    for name in CODEBASE_MAPS:
        (root / ".planning" / "codebase" / name).write_text(
            "Phase 8 admin media review and operational posture with security patching.\n",
            encoding="utf-8",
        )


class ValidateRepoHygieneTests(unittest.TestCase):
    def test_active_nextjs_runtime_current_claim_violates(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, docs_text="The Active Next.js runtime handles current traffic.\n")

            violations = validate_repo_hygiene.collect_violations(root)

            self.assertTrue(any("active Next.js runtime" in violation for violation in violations))

    def test_current_nextjs_runtime_current_claim_violates(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, docs_text="The Current Next.js runtime handles current traffic.\n")

            violations = validate_repo_hygiene.collect_violations(root)

            self.assertTrue(any("current Next.js runtime" in violation for violation in violations))

    def test_phase_8_ai_ingest_without_move_note_violates(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root, docs_text="phase 8 ai-assisted ingest is active scope.\n")

            violations = validate_repo_hygiene.collect_violations(root)

            self.assertTrue(any("Phase 8 AI-assisted ingest" in violation for violation in violations))

    def test_every_codebase_map_requires_phase_8_ownership(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_repo(root)
            (root / ".planning" / "codebase" / "CONVENTIONS.md").write_text(
                "Phase 7 taxonomy conventions remain current.\n",
                encoding="utf-8",
            )

            violations = validate_repo_hygiene.collect_violations(root)

            self.assertTrue(any("CONVENTIONS.md" in violation for violation in violations))

    def test_real_repository_validation_passes(self):
        root = Path(__file__).resolve().parents[1]

        self.assertEqual([], validate_repo_hygiene.collect_violations(root))


if __name__ == "__main__":
    unittest.main()
