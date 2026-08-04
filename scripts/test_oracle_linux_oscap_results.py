#!/usr/bin/env python3

import contextlib
import importlib.util
import io
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("oracle_linux_oscap_results.py")
spec = importlib.util.spec_from_file_location("oracle_linux_oscap_results", SCRIPT_PATH)
oracle_linux_oscap_results = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(oracle_linux_oscap_results)


OVAL_XML = """<?xml version="1.0" encoding="UTF-8"?>
<oval_definitions xmlns="http://oval.mitre.org/XMLSchema/oval-definitions-5">
  <definitions>
    <definition id="oval:com.oracle.elsa:def:2026500006" class="patch">
      <metadata>
        <title>ELSA-2026-500006: glibc security update</title>
        <reference source="elsa" ref_id="ELSA-2026-500006" ref_url="https://linux.oracle.com/errata/ELSA-2026-500006.html"/>
        <advisory>
          <severity>MODERATE</severity>
          <cve href="https://linux.oracle.com/cve/CVE-2026-4437.html">CVE-2026-4437</cve>
          <affected_cpe_list>
            <cpe>cpe:/a:oracle:linux:10::userspace_ksplice</cpe>
          </affected_cpe_list>
        </advisory>
      </metadata>
      <criteria operator="AND">
        <criterion test_ref="oval:com.oracle.elsa:tst:1" comment="glibc is earlier than 2:2.39-121.0.1.ksplice1.el10_2"/>
        <criterion test_ref="oval:com.oracle.elsa:tst:2" comment="glibc is ksplice-based"/>
      </criteria>
    </definition>
    <definition id="oval:com.oracle.elsa:def:202622963" class="patch">
      <metadata>
        <title>ELSA-2026-22963: samba security update</title>
        <reference source="elsa" ref_id="ELSA-2026-22963" ref_url="https://linux.oracle.com/errata/ELSA-2026-22963.html"/>
        <advisory>
          <severity>CRITICAL</severity>
          <cve href="https://linux.oracle.com/cve/CVE-2026-0001.html">CVE-2026-0001</cve>
        </advisory>
      </metadata>
      <criteria operator="AND">
        <criterion test_ref="oval:com.oracle.elsa:tst:3" comment="samba-common is earlier than 0:4.23.5-109.el10_2"/>
      </criteria>
    </definition>
    <definition id="oval:com.oracle.elsa:def:2026500005" class="patch">
      <metadata>
        <title>ELSA-2026-500005: openssl security update</title>
        <reference source="elsa" ref_id="ELSA-2026-500005" ref_url="https://linux.oracle.com/errata/ELSA-2026-500005.html"/>
        <advisory>
          <severity>MODERATE</severity>
          <cve href="https://linux.oracle.com/cve/CVE-2026-0002.html">CVE-2026-0002</cve>
        </advisory>
      </metadata>
      <criteria operator="AND">
        <criterion test_ref="oval:com.oracle.elsa:tst:4" comment="openssl is earlier than 1:3.5.5-5.0.1.el10_2"/>
      </criteria>
    </definition>
  </definitions>
</oval_definitions>
"""


RESULTS_XML = """<?xml version="1.0" encoding="UTF-8"?>
<oval_results xmlns="http://oval.mitre.org/XMLSchema/oval-results-5">
  <results>
    <system>
      <definitions>
        <definition definition_id="oval:com.oracle.elsa:def:2026500006" result="true"/>
        <definition definition_id="oval:com.oracle.elsa:def:202622963" result="false"/>
      </definitions>
    </system>
  </results>
</oval_results>
"""


RESULTS_WITH_EVALUATION_PROBLEMS_XML = """<?xml version="1.0" encoding="UTF-8"?>
<oval_results xmlns="http://oval.mitre.org/XMLSchema/oval-results-5">
  <results>
    <system>
      <definitions>
        <definition definition_id="oval:com.oracle.elsa:def:2026500006" result="true"/>
        <definition definition_id="oval:com.oracle.elsa:def:202622963" result="error"/>
        <definition definition_id="oval:com.oracle.elsa:def:2026500005" result="unknown"/>
      </definitions>
    </system>
  </results>
</oval_results>
"""


RESULTS_WITH_MISSING_RESULT_XML = """<?xml version="1.0" encoding="UTF-8"?>
<oval_results xmlns="http://oval.mitre.org/XMLSchema/oval-results-5">
  <results>
    <system>
      <definitions>
        <definition definition_id="oval:com.oracle.elsa:def:2026500006"/>
      </definitions>
    </system>
  </results>
</oval_results>
"""


RESULTS_WITH_EMPTY_DEFINITIONS_XML = """<?xml version="1.0" encoding="UTF-8"?>
<oval_results xmlns="http://oval.mitre.org/XMLSchema/oval-results-5">
  <results>
    <system>
      <definitions/>
    </system>
  </results>
</oval_results>
"""


class OracleLinuxOscapResultsTests(unittest.TestCase):
    def test_parse_oscap_results_returns_true_elsa_findings(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            oval_path = root / "oval.xml"
            results_path = root / "results.xml"
            oval_path.write_text(OVAL_XML, encoding="utf-8")
            results_path.write_text(RESULTS_XML, encoding="utf-8")

            parsed = oracle_linux_oscap_results.parse_oscap_results(results_path, oval_path, "production")

        self.assertEqual(parsed["status"], "complete")
        self.assertEqual(parsed["advisory_ids"], ["ELSA-2026-500006"])
        self.assertEqual(parsed["ksplice_aware_advisory_ids"], ["ELSA-2026-500006"])
        advisory = parsed["advisories"][0]
        self.assertEqual(advisory["severity"], "MODERATE")
        self.assertEqual(advisory["cves"], ["CVE-2026-4437"])
        self.assertEqual(advisory["packages"], ["glibc"])
        self.assertEqual(advisory["package_count"], 1)
        self.assertEqual(advisory["errata_link"], "https://linux.oracle.com/errata/ELSA-2026-500006.html")
        self.assertTrue(advisory["ksplice_aware"])

    def test_main_writes_parseable_json(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            oval_path = root / "oval.xml"
            results_path = root / "results.xml"
            output_path = root / "parsed.json"
            oval_path.write_text(OVAL_XML, encoding="utf-8")
            results_path.write_text(RESULTS_XML, encoding="utf-8")

            rc = oracle_linux_oscap_results.main(
                [
                    "--results",
                    str(results_path),
                    "--oval-definitions",
                    str(oval_path),
                    "--output",
                    str(output_path),
                    "--host",
                    "production",
                ]
            )

            parsed = json.loads(output_path.read_text(encoding="utf-8"))

        self.assertEqual(rc, 0)
        self.assertEqual(parsed["host"], "production")
        self.assertEqual(parsed["entries"][0]["advisory_id"], "ELSA-2026-500006")

    def test_parse_oscap_results_degrades_on_evaluation_errors(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            oval_path = root / "oval.xml"
            results_path = root / "results.xml"
            oval_path.write_text(OVAL_XML, encoding="utf-8")
            results_path.write_text(RESULTS_WITH_EVALUATION_PROBLEMS_XML, encoding="utf-8")

            parsed = oracle_linux_oscap_results.parse_oscap_results(results_path, oval_path, "production")

        self.assertEqual(parsed["status"], "degraded")
        self.assertEqual(parsed["advisory_ids"], ["ELSA-2026-500006"])
        self.assertEqual(
            parsed["evaluation_problem_definition_results"],
            [
                {
                    "definition_id": "oval:com.oracle.elsa:def:202622963",
                    "result": "error",
                    "advisory_id": "ELSA-2026-22963",
                    "summary": "ELSA-2026-22963: samba security update",
                },
                {
                    "definition_id": "oval:com.oracle.elsa:def:2026500005",
                    "result": "unknown",
                    "advisory_id": "ELSA-2026-500005",
                    "summary": "ELSA-2026-500005: openssl security update",
                }
            ],
        )

    def test_main_returns_failure_for_degraded_results_after_writing_json(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            oval_path = root / "oval.xml"
            results_path = root / "results.xml"
            output_path = root / "parsed.json"
            oval_path.write_text(OVAL_XML, encoding="utf-8")
            results_path.write_text(RESULTS_WITH_EVALUATION_PROBLEMS_XML, encoding="utf-8")

            with contextlib.redirect_stderr(io.StringIO()):
                rc = oracle_linux_oscap_results.main(
                    [
                        "--results",
                        str(results_path),
                        "--oval-definitions",
                        str(oval_path),
                        "--output",
                        str(output_path),
                        "--host",
                        "production",
                    ]
                )

            parsed = json.loads(output_path.read_text(encoding="utf-8"))

        self.assertEqual(rc, 4)
        self.assertEqual(parsed["status"], "degraded")
        self.assertEqual(len(parsed["evaluation_problem_definition_results"]), 2)

    def test_parse_oscap_results_degrades_on_missing_definition_result(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            oval_path = root / "oval.xml"
            results_path = root / "results.xml"
            oval_path.write_text(OVAL_XML, encoding="utf-8")
            results_path.write_text(RESULTS_WITH_MISSING_RESULT_XML, encoding="utf-8")

            parsed = oracle_linux_oscap_results.parse_oscap_results(results_path, oval_path, "production")

        self.assertEqual(parsed["status"], "degraded")
        self.assertEqual(
            parsed["evaluation_problem_definition_results"],
            [
                {
                    "definition_id": "oval:com.oracle.elsa:def:2026500006",
                    "result": "",
                    "advisory_id": "",
                    "summary": "",
                },
                {
                    "definition_id": "",
                    "result": "missing",
                    "advisory_id": "",
                    "summary": "OpenSCAP results contained no evaluated definitions with result states.",
                },
            ],
        )

    def test_main_returns_failure_when_results_contain_no_evaluated_definitions(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            oval_path = root / "oval.xml"
            results_path = root / "results.xml"
            output_path = root / "parsed.json"
            oval_path.write_text(OVAL_XML, encoding="utf-8")
            results_path.write_text(RESULTS_WITH_EMPTY_DEFINITIONS_XML, encoding="utf-8")

            with contextlib.redirect_stderr(io.StringIO()):
                rc = oracle_linux_oscap_results.main(
                    [
                        "--results",
                        str(results_path),
                        "--oval-definitions",
                        str(oval_path),
                        "--output",
                        str(output_path),
                        "--host",
                        "production",
                    ]
                )

            parsed = json.loads(output_path.read_text(encoding="utf-8"))

        self.assertEqual(rc, 4)
        self.assertEqual(parsed["status"], "degraded")
        self.assertEqual(
            parsed["evaluation_problem_definition_results"],
            [
                {
                    "definition_id": "",
                    "result": "missing",
                    "advisory_id": "",
                    "summary": "OpenSCAP results contained no evaluated definitions with result states.",
                }
            ],
        )


if __name__ == "__main__":
    unittest.main()
