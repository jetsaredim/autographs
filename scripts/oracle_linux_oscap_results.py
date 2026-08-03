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


def load_true_definition_ids(results_path: Path) -> list[str]:
    root = _xml_root_from_path(results_path)
    true_definition_ids: set[str] = set()
    for element in root.iter():
        if _local_name(element.tag) != "definition":
            continue
        definition_id = element.attrib.get("definition_id") or element.attrib.get("id", "")
        if definition_id and element.attrib.get("result", "").lower() == "true":
            true_definition_ids.add(definition_id)
    return sorted(true_definition_ids)


def parse_oscap_results(
    results_path: Path,
    oval_path: Path,
    host: str,
    errata_base_url: str = DEFAULT_ERRATA_BASE_URL,
) -> dict[str, Any]:
    definitions = load_oval_definitions(oval_path, errata_base_url=errata_base_url)
    true_definition_ids = load_true_definition_ids(results_path)
    advisories = []
    unknown_definition_ids = []

    for definition_id in true_definition_ids:
        advisory = definitions.get(definition_id)
        if not advisory:
            unknown_definition_ids.append(definition_id)
            continue
        advisories.append(advisory)

    advisories.sort(key=lambda advisory: advisory["advisory_id"])
    status = "complete" if not unknown_definition_ids else "degraded"
    message = (
        "Oracle Linux OpenSCAP OVAL evaluation completed."
        if status == "complete"
        else "Oracle Linux OpenSCAP OVAL evaluation completed with unmapped true definitions."
    )
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
    return 0


if __name__ == "__main__":
    sys.exit(main())
