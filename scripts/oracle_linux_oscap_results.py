#!/usr/bin/env python3

import argparse
import bz2
import json
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any


DEFAULT_ERRATA_BASE_URL = "https://linux.oracle.com/errata/"
ADVISORY_RE = re.compile(r"\bELSA-\d{4}-\d+\b")
PACKAGE_COMMENT_RE = re.compile(r"^(?P<name>[A-Za-z0-9_.+-]+) is earlier than ")
FINDING_RESULTS = {"true"}
CLEAN_RESULTS = {"false", "not applicable"}


def _local_name(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def _xml_root_from_path(path: Path) -> ET.Element:
    if path.suffix == ".bz2":
        return ET.fromstring(bz2.decompress(path.read_bytes()))
    return ET.parse(path).getroot()


def build_errata_link(advisory_id: str, errata_base_url: str = DEFAULT_ERRATA_BASE_URL) -> str:
    if not advisory_id.startswith("ELSA-"):
        return ""
    return f"{errata_base_url.rstrip('/')}/{advisory_id}.html"


def _definition_advisory_id(definition: ET.Element) -> str | None:
    for element in definition.iter():
        if _local_name(element.tag) not in {"ref", "reference"}:
            continue
        match = ADVISORY_RE.search(element.attrib.get("ref_id", ""))
        if match:
            return match.group(0)
    for element in definition.iter():
        match = ADVISORY_RE.search(element.text or "")
        if match:
            return match.group(0)
    return None


def _definition_packages(definition: ET.Element) -> list[str]:
    packages: set[str] = set()
    for element in definition.iter():
        if _local_name(element.tag) != "criterion":
            continue
        match = PACKAGE_COMMENT_RE.match(element.attrib.get("comment", ""))
        if match:
            packages.add(match.group("name"))
    return sorted(packages)


def load_oval_definitions(oval_path: Path, errata_base_url: str = DEFAULT_ERRATA_BASE_URL) -> dict[str, dict[str, Any]]:
    root = _xml_root_from_path(oval_path)
    definitions: dict[str, dict[str, Any]] = {}
    for definition in root.iter():
        if _local_name(definition.tag) != "definition":
            continue
        definition_id = definition.attrib.get("id", "")
        advisory_id = _definition_advisory_id(definition)
        if not definition_id or not advisory_id:
            continue
        severity = "unknown"
        cves: set[str] = set()
        summary = ""
        affected_cpes: set[str] = set()
        ksplice_aware = False
        for element in definition.iter():
            name = _local_name(element.tag)
            text = (element.text or "").strip()
            if name == "title" and text and not summary:
                summary = text
            elif name == "severity" and text:
                severity = text
            elif name == "cve" and text:
                cves.add(text)
            elif name == "cpe" and text:
                affected_cpes.add(text)
                if "ksplice" in text.lower():
                    ksplice_aware = True
            elif name == "criterion" and "ksplice" in element.attrib.get("comment", "").lower():
                ksplice_aware = True

        packages = _definition_packages(definition)
        definitions[definition_id] = {
            "definition_id": definition_id,
            "advisory_id": advisory_id,
            "severity": severity,
            "cves": sorted(cves),
            "summary": summary,
            "errata_link": build_errata_link(advisory_id, errata_base_url),
            "packages": packages,
            "package_count": len(packages),
            "affected_cpes": sorted(affected_cpes),
            "ksplice_aware": ksplice_aware,
        }
    return definitions


def load_definition_results(results_path: Path) -> dict[str, str]:
    definition_results, _ = load_evaluated_definition_results(results_path)
    return definition_results


def load_evaluated_definition_results(results_path: Path) -> tuple[dict[str, str], list[dict[str, str]]]:
    root = _xml_root_from_path(results_path)
    definition_results: dict[str, str] = {}
    malformed_definition_results: list[dict[str, str]] = []
    for system in root.iter():
        if _local_name(system.tag) != "system":
            continue
        for child in list(system):
            if _local_name(child.tag) != "definitions":
                continue
            for element in list(child):
                if _local_name(element.tag) != "definition":
                    continue
                definition_id = element.attrib.get("definition_id") or element.attrib.get("id", "")
                result = element.attrib.get("result", "").strip().lower()
                if not definition_id or not result:
                    malformed_definition_results.append(
                        {
                            "definition_id": definition_id,
                            "result": result,
                            "advisory_id": "",
                            "summary": "",
                        }
                    )
                else:
                    definition_results[definition_id] = result
    return definition_results, malformed_definition_results


def load_true_definition_ids(results_path: Path) -> list[str]:
    return sorted(
        definition_id
        for definition_id, result in load_definition_results(results_path).items()
        if result in FINDING_RESULTS
    )


def parse_oscap_results(
    results_path: Path,
    oval_path: Path,
    host: str,
    errata_base_url: str = DEFAULT_ERRATA_BASE_URL,
) -> dict[str, Any]:
    definitions = load_oval_definitions(oval_path, errata_base_url=errata_base_url)
    definition_results, malformed_definition_results = load_evaluated_definition_results(results_path)
    true_definition_ids = sorted(
        definition_id for definition_id, result in definition_results.items() if result in FINDING_RESULTS
    )
    advisories = []
    unknown_definition_ids = []
    evaluation_problem_definition_results = malformed_definition_results

    for definition_id in true_definition_ids:
        advisory = definitions.get(definition_id)
        if not advisory:
            unknown_definition_ids.append(definition_id)
            continue
        advisories.append(advisory)

    for definition_id, result in sorted(definition_results.items()):
        if result in FINDING_RESULTS or result in CLEAN_RESULTS:
            continue
        advisory = definitions.get(definition_id, {})
        evaluation_problem_definition_results.append(
            {
                "definition_id": definition_id,
                "result": result,
                "advisory_id": advisory.get("advisory_id", ""),
                "summary": advisory.get("summary", ""),
            }
        )

    if not definition_results:
        evaluation_problem_definition_results.append(
            {
                "definition_id": "",
                "result": "missing",
                "advisory_id": "",
                "summary": "OpenSCAP results contained no evaluated definitions with result states.",
            }
        )

    advisories.sort(key=lambda advisory: advisory["advisory_id"])
    status = "complete" if not unknown_definition_ids and not evaluation_problem_definition_results else "degraded"
    if status == "complete":
        message = "Oracle Linux OpenSCAP OVAL evaluation completed."
    elif evaluation_problem_definition_results:
        message = (
            "Oracle Linux OpenSCAP OVAL evaluation completed with "
            f"{len(evaluation_problem_definition_results)} non-complete definition result(s)."
        )
    else:
        message = "Oracle Linux OpenSCAP OVAL evaluation completed with unmapped true definitions."
    advisory_ids = sorted({advisory["advisory_id"] for advisory in advisories})
    return {
        "host": host,
        "status": status,
        "message": message,
        "advisory_ids": advisory_ids,
        "ksplice_aware_advisory_ids": sorted(
            {advisory["advisory_id"] for advisory in advisories if advisory.get("ksplice_aware")}
        ),
        "unknown_definition_ids": unknown_definition_ids,
        "evaluation_problem_definition_results": evaluation_problem_definition_results,
        "advisories": advisories,
        "entries": advisories,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Parse Oracle Linux OpenSCAP OVAL results")
    parser.add_argument("--results", required=True, help="OpenSCAP OVAL results XML path")
    parser.add_argument("--oval-definitions", required=True, help="Oracle Linux OVAL definitions XML or XML.bz2 path")
    parser.add_argument("--output", required=True, help="Path to write parsed JSON")
    parser.add_argument("--host", required=True, help="Inventory host name")
    parser.add_argument("--errata-base-url", default=DEFAULT_ERRATA_BASE_URL, help="Oracle Linux errata base URL")
    args = parser.parse_args(argv)

    try:
        parsed = parse_oscap_results(
            Path(args.results),
            Path(args.oval_definitions),
            args.host,
            errata_base_url=args.errata_base_url,
        )
    except (OSError, EOFError, ET.ParseError, ValueError) as error:
        print(f"Failed to parse OpenSCAP results: {error}", file=sys.stderr)
        return 2

    try:
        Path(args.output).write_text(json.dumps(parsed, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    except OSError as error:
        print(f"Failed to write parsed OpenSCAP results: {error}", file=sys.stderr)
        return 3
    if parsed.get("status") != "complete":
        print(parsed.get("message", "OpenSCAP result parsing was degraded."), file=sys.stderr)
        return 4
    return 0


if __name__ == "__main__":
    sys.exit(main())
