#!/usr/bin/env python3

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("oracle_linux_advisory_enrichment.py")
spec = importlib.util.spec_from_file_location("oracle_linux_advisory_enrichment", SCRIPT_PATH)
oracle_linux_advisory_enrichment = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(oracle_linux_advisory_enrichment)


class OracleLinuxAdvisoryEnrichmentTests(unittest.TestCase):
    def test_parse_update_entries_builds_oracle_errata_link(self):
        entries = [
            {
                "advisory_id": "ELSA-2025-20632",
                "severity": "Important",
                "package_spec": "kernel-5.15.0-300.163.18.el9uek.x86_64",
                "cves": [],
                "enrichmentStatus": "minimal",
            }
        ]

        parsed = oracle_linux_advisory_enrichment.parse_update_entries(entries)

        self.assertEqual(parsed[0]["advisory_id"], "ELSA-2025-20632")
        self.assertEqual(parsed[0]["package_spec"], "kernel-5.15.0-300.163.18.el9uek.x86_64")
        self.assertEqual(
            oracle_linux_advisory_enrichment.build_errata_link("ELSA-2025-20632"),
            "https://linux.oracle.com/errata/ELSA-2025-20632.html",
        )
        self.assertEqual(oracle_linux_advisory_enrichment.build_errata_link("RHSA-2025:20632"), "")

    def test_load_oval_enrichment_reads_cve_and_severity(self):
        oval_xml = """<?xml version="1.0" encoding="UTF-8"?>
<oval_definitions xmlns="http://oval.mitre.org/XMLSchema/oval-definitions-5">
  <definitions>
    <definition id="oval:com.oracle.elsa:def:202520632" class="patch">
      <metadata>
        <title>ELSA-2025-20632: kernel security update</title>
        <advisory from="secalert_us@oracle.com">
          <severity>Important</severity>
          <cve href="https://linux.oracle.com/cve/CVE-2025-12345.html">CVE-2025-12345</cve>
          <cve href="https://linux.oracle.com/cve/CVE-2025-67890.html">CVE-2025-67890</cve>
          <ref ref_id="ELSA-2025-20632" ref_url="https://linux.oracle.com/errata/ELSA-2025-20632.html"/>
        </advisory>
      </metadata>
    </definition>
  </definitions>
</oval_definitions>
"""
        with tempfile.TemporaryDirectory() as tmp:
            oval_path = Path(tmp) / "oval.xml"
            oval_path.write_text(oval_xml, encoding="utf-8")

            enrichment = oracle_linux_advisory_enrichment.load_oval_enrichment(oval_path.as_uri())

        self.assertEqual(enrichment["ELSA-2025-20632"]["severity"], "Important")
        self.assertEqual(
            enrichment["ELSA-2025-20632"]["cves"],
            ["CVE-2025-12345", "CVE-2025-67890"],
        )

    def test_enrich_inventory_degrades_when_oval_is_unavailable(self):
        inventory = {
            "hosts": [
                {
                    "name": "production",
                    "package_specs": ["kernel-5.15.0-300.163.18.el9uek.x86_64"],
                    "entries": [
                        {
                            "advisory_id": "ELSA-2025-20632",
                            "severity": "Important",
                            "package_spec": "kernel-5.15.0-300.163.18.el9uek.x86_64",
                            "cves": [],
                            "enrichmentStatus": "minimal",
                        }
                    ],
                }
            ]
        }

        result = oracle_linux_advisory_enrichment.enrich_inventory(
            inventory,
            oval_url="file:///tmp/autographs-missing-oval.xml",
        )

        self.assertEqual(result["status"], "degraded")
        self.assertEqual(
            result["hosts"][0]["package_specs"],
            ["kernel-5.15.0-300.163.18.el9uek.x86_64"],
        )
        self.assertEqual(
            result["hosts"][0]["entries"][0]["errata_link"],
            "https://linux.oracle.com/errata/ELSA-2025-20632.html",
        )

    def test_enrich_inventory_marks_partial_oval_matches_degraded(self):
        oval_xml = """<?xml version="1.0" encoding="UTF-8"?>
<oval_definitions xmlns="http://oval.mitre.org/XMLSchema/oval-definitions-5">
  <definitions>
    <definition id="oval:com.oracle.elsa:def:202520632" class="patch">
      <metadata>
        <title>ELSA-2025-20632: kernel security update</title>
        <advisory from="secalert_us@oracle.com">
          <severity>Important</severity>
          <cve href="https://linux.oracle.com/cve/CVE-2025-12345.html">CVE-2025-12345</cve>
          <ref ref_id="ELSA-2025-20632" ref_url="https://linux.oracle.com/errata/ELSA-2025-20632.html"/>
        </advisory>
      </metadata>
    </definition>
  </definitions>
</oval_definitions>
"""
        inventory = {
            "hosts": [
                {
                    "name": "production",
                    "package_specs": [
                        "kernel-5.15.0-300.163.18.el9uek.x86_64",
                        "openssl-3.2.2-1.el9.x86_64",
                    ],
                    "entries": [
                        {
                            "advisory_id": "ELSA-2025-20632",
                            "severity": "Important",
                            "package_spec": "kernel-5.15.0-300.163.18.el9uek.x86_64",
                        },
                        {
                            "advisory_id": "ELSA-2025-99999",
                            "severity": "Moderate",
                            "package_spec": "openssl-3.2.2-1.el9.x86_64",
                        },
                    ],
                }
            ]
        }
        with tempfile.TemporaryDirectory() as tmp:
            oval_path = Path(tmp) / "oval.xml"
            oval_path.write_text(oval_xml, encoding="utf-8")

            result = oracle_linux_advisory_enrichment.enrich_inventory(
                inventory,
                oval_url=oval_path.as_uri(),
            )

        entries = result["hosts"][0]["entries"]
        self.assertEqual(result["status"], "degraded")
        self.assertEqual(entries[0]["enrichmentStatus"], "complete")
        self.assertEqual(entries[0]["cves"], ["CVE-2025-12345"])
        self.assertEqual(entries[1]["enrichmentStatus"], "minimal")
        self.assertEqual(entries[1]["cves"], [])

    def test_main_writes_degraded_inventory_when_oval_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            input_path = root / "inventory.json"
            output_path = root / "enriched.json"
            input_path.write_text(
                json.dumps(
                    {
                        "hosts": [
                            {
                                "name": "production",
                                "package_specs": ["openssl-3.2.2-1.el9.x86_64"],
                                "entries": [
                                    {
                                        "advisory_id": "ELSA-2025-20001",
                                        "severity": "Moderate",
                                        "package_spec": "openssl-3.2.2-1.el9.x86_64",
                                    }
                                ],
                            }
                        ]
                    }
                ),
                encoding="utf-8",
            )

            exit_code = oracle_linux_advisory_enrichment.main(
                [
                    "--input",
                    str(input_path),
                    "--output",
                    str(output_path),
                    "--oval-url",
                    "file:///tmp/autographs-missing-oval.xml",
                ]
            )

            self.assertEqual(exit_code, 0)
            written = json.loads(output_path.read_text(encoding="utf-8"))
        self.assertEqual(written["status"], "degraded")
        self.assertEqual(written["hosts"][0]["entries"][0]["advisory_id"], "ELSA-2025-20001")


if __name__ == "__main__":
    unittest.main()
