import importlib.util
from pathlib import Path
import tempfile
import unittest


MODULE_PATH = Path(__file__).with_name("rust_audit.py")
SPEC = importlib.util.spec_from_file_location("rust_audit", MODULE_PATH)
audit_module = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(audit_module)


class RustAuditTests(unittest.TestCase):
    def test_audit_finds_large_modules_and_distributed_env_reads(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "src"
            tests = root / "tests"
            source.mkdir()
            tests.mkdir()
            (source / "large.rs").write_text(
                'use std::env;\n' + 'fn line() {}\n' * 1500 + 'fn config() { env::var("X"); }\n',
                encoding="utf-8",
            )
            (tests / "smoke.rs").write_text("#[test]\nfn smoke() {}\n", encoding="utf-8")

            report = audit_module.audit(root)
            kinds = {finding["kind"] for finding in report["findings"]}

            self.assertIn("large-module", kinds)
            self.assertIn("distributed-configuration-read", kinds)

    def test_config_module_owns_direct_env_reads_without_a_finding(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "tests").mkdir()
            (root / "src" / "config.rs").write_text(
                'use std::env; fn load() { env::var("X"); }\n', encoding="utf-8"
            )

            report = audit_module.audit(root)

            self.assertFalse(
                any(item["kind"] == "distributed-configuration-read" for item in report["findings"])
            )


if __name__ == "__main__":
    unittest.main()
