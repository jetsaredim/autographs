#!/usr/bin/env python3

import argparse
import bz2
import json
import re
import sys
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any


DEFAULT_ERRATA_BASE_URL = "https://linux.oracle.com/errata/"
DEFAULT_OVAL_URL = "https://linux.oracle.com/security/oval/com.oracle.elsa-all.xml.bz2"
ADVISORY_RE = re.compile(r"\bELSA-\d{4}-\d+\b")


def parse_update_entries(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    parsed = []
    for entry in entries:
        advisory_id = str(entry.get("advisory_id", "")).strip()
        package_spec = str(entry.get("package_spec", "")).strip()
        if not advisory_id or not package_spec:
            continue
        cves = sorted({str(cve).strip() for cve in entry.get("cves", []) if str(cve).strip()})
        parsed.append(
            {
                "advisory_id": advisory_id,
                "severity": str(entry.get("severity", "") or "unknown").strip() or "unknown",
                "package_spec": package_spec,
                "cves": cves,
                "enrichmentStatus": str(entry.get("enrichmentStatus", "") or "minimal"),
            }
        )
    return parsed


def build_errata_link(advisory_id: str, errata_base_url: str = DEFAULT_ERRATA_BASE_URL) -> str:
    if not advisory_id.startswith("ELSA-"):
        return ""
    base_url = errata_base_url.rstrip("/") + "/"
    return f"{base_url}{advisory_id}.html"


def _local_name(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def _read_url(url: str) -> bytes:
    with urllib.request.urlopen(url, timeout=30) as response:
        return response.read()


def _xml_bytes_from_url(url: str) -> bytes:
    payload = _read_url(url)
    if url.endswith(".bz2"):
        return bz2.decompress(payload)
    return payload


def _definition_advisory_id(definition: ET.Element) -> str | None:
    for element in definition.iter():
        if _local_name(element.tag) != "ref":
            continue
        ref_id = element.attrib.get("ref_id", "")
        match = ADVISORY_RE.search(ref_id)
        if match:
            return match.group(0)
    for element in definition.iter():
        text = element.text or ""
        match = ADVISORY_RE.search(text)
        if match:
            return match.group(0)
    return None


def load_oval_enrichment(oval_url: str) -> dict[str, dict[str, Any]]:
    root = ET.fromstring(_xml_bytes_from_url(oval_url))
    enrichment: dict[str, dict[str, Any]] = {}
    for definition in root.iter():
        if _local_name(definition.tag) != "definition":
            continue
        advisory_id = _definition_advisory_id(definition)
        if not advisory_id:
            continue
        severity = "unknown"
        cves: set[str] = set()
        summary = ""
        for element in definition.iter():
            name = _local_name(element.tag)
            text = (element.text or "").strip()
            if name == "title" and text and not summary:
                summary = text
            elif name == "severity" and text:
                severity = text
            elif name == "cve" and text:
                cves.add(text)
        enrichment[advisory_id] = {
            "severity": severity,
            "cves": sorted(cves),
            "summary": summary,
        }
    return enrichment


def summarize_advisories(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    summaries: dict[str, dict[str, Any]] = {}
    for entry in entries:
        advisory_id = str(entry.get("advisory_id", "")).strip()
        if not advisory_id:
            continue
        summary = summaries.setdefault(
            advisory_id,
            {
                "advisory_id": advisory_id,
                "severity": str(entry.get("severity", "") or "unknown"),
                "cves": set(),
                "errata_link": str(entry.get("errata_link", "") or ""),
                "summary": str(entry.get("summary", "") or ""),
                "package_specs": set(),
            },
        )
        summary["package_specs"].add(str(entry.get("package_spec", "") or ""))
        summary["cves"].update(str(cve).strip() for cve in entry.get("cves", []) if str(cve).strip())
        if summary["severity"] in {"", "unknown"} and entry.get("severity"):
            summary["severity"] = str(entry["severity"])
        if not summary["summary"] and entry.get("summary"):
            summary["summary"] = str(entry["summary"])
        if not summary["errata_link"] and entry.get("errata_link"):
            summary["errata_link"] = str(entry["errata_link"])

    return [
        {
            "advisory_id": advisory_id,
            "severity": str(summary["severity"] or "unknown"),
            "cves": sorted(summary["cves"]),
            "errata_link": summary["errata_link"],
            "summary": summary["summary"],
            "package_count": len({spec for spec in summary["package_specs"] if spec}),
        }
        for advisory_id, summary in sorted(summaries.items())
    ]


def enrich_inventory(
    inventory: dict[str, Any],
    oval_url: str = DEFAULT_OVAL_URL,
    errata_base_url: str = DEFAULT_ERRATA_BASE_URL,
) -> dict[str, Any]:
    try:
        oval_enrichment = load_oval_enrichment(oval_url) if oval_url else {}
        status = "complete" if oval_enrichment else "minimal"
        message = (
            "Oracle Linux OVAL advisory enrichment completed."
            if oval_enrichment
            else "Runtime package inventory was collected without advisory enrichment."
        )
    except (OSError, ValueError, ET.ParseError, urllib.error.URLError, EOFError) as error:
        oval_enrichment = {}
        status = "degraded"
        message = f"Oracle Linux OVAL enrichment failed; package inventory and errata links were preserved: {error}"

    enriched_hosts = []
    total_entries = 0
    matched_entries = 0
    for host in inventory.get("hosts", []):
        host_entries = []
        for entry in parse_update_entries(host.get("entries", [])):
            total_entries += 1
            advisory = oval_enrichment.get(entry["advisory_id"], {})
            if advisory:
                matched_entries += 1
            cves = advisory.get("cves") or entry.get("cves", [])
            severity = advisory.get("severity") or entry.get("severity", "unknown")
            entry_status = "complete" if advisory else ("minimal" if oval_enrichment else status)
            host_entries.append(
                {
                    **entry,
                    "severity": severity,
                    "cves": cves,
                    "summary": advisory.get("summary", ""),
                    "errata_link": build_errata_link(entry["advisory_id"], errata_base_url),
                    "enrichmentStatus": entry_status,
                }
            )
        enriched_hosts.append(
            {
                "name": host.get("name"),
                "package_specs": host.get("package_specs", []),
                "advisories": summarize_advisories(host_entries),
                "entries": host_entries,
            }
        )

    if oval_enrichment and matched_entries < total_entries:
        status = "degraded"
        message = "Oracle Linux OVAL enrichment was partial; package inventory and errata links were preserved."

    return {
        "status": status,
        "message": message,
        "hosts": enriched_hosts,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Best-effort Oracle Linux advisory enrichment")
    parser.add_argument("--input", required=True, help="Path to scanner inventory JSON")
    parser.add_argument("--output", required=True, help="Path to write enrichment JSON")
    parser.add_argument("--oval-url", default=DEFAULT_OVAL_URL, help="Oracle Linux OVAL XML or XML.bz2 URL")
    parser.add_argument("--errata-base-url", default=DEFAULT_ERRATA_BASE_URL, help="Oracle Linux errata base URL")
    args = parser.parse_args(argv)

    input_path = Path(args.input)
    output_path = Path(args.output)
    try:
        inventory = json.loads(input_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        print(f"Failed to read enrichment input: {error}", file=sys.stderr)
        return 2

    result = enrich_inventory(inventory, oval_url=args.oval_url, errata_base_url=args.errata_base_url)
    try:
        output_path.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    except OSError as error:
        print(f"Failed to write enrichment output: {error}", file=sys.stderr)
        return 3
    return 0


if __name__ == "__main__":
    sys.exit(main())
