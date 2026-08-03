#!/usr/bin/env python3

import re
import unittest
from pathlib import Path


TASKS_PATH = (
    Path(__file__).resolve().parents[1]
    / "deploy"
    / "ansible"
    / "roles"
    / "security_patching"
    / "tasks"
    / "create_issue.yml"
)
DEFAULTS_PATH = TASKS_PATH.parents[1] / "defaults" / "main.yml"
REPORT_TEMPLATE_PATH = TASKS_PATH.parents[1] / "templates" / "security-report.md.j2"
VALIDATE_REQUEST_TASKS_PATH = TASKS_PATH.with_name("validate_request.yml")
EXTRACT_METADATA_TASKS_PATH = TASKS_PATH.with_name("extract_issue_metadata.yml")
PATCH_TASKS_PATH = TASKS_PATH.with_name("patch.yml")
REPORT_RENDER_TEST_PLAYBOOK_PATH = (
    Path(__file__).resolve().parents[1]
    / "deploy"
    / "ansible"
    / "playbooks"
    / "security-report-render-test.yml"
)
REQUEST_METADATA_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-request-metadata-validate-test.yml"
)


def task_block(task_name: str) -> str:
    text = TASKS_PATH.read_text(encoding="utf-8")
    pattern = re.compile(rf"^- name: {re.escape(task_name)}\n(?P<body>.*?)(?=^- name: |\Z)", re.M | re.S)
    match = pattern.search(text)
    if not match:
        raise AssertionError(f"Task not found: {task_name}")
    return match.group("body")


class SecurityPatchingCreateIssueTasksTests(unittest.TestCase):
    def test_issue_url_fact_is_available_before_open_issue_url_uses_it(self):
        build_issue_url = task_block("Build GitHub issues URLs")
        build_open_issue_url = task_block("Build open GitHub issues URL")

        self.assertIn("security_patching_issues_url:", build_issue_url)
        self.assertNotIn("security_patching_open_issues_url:", build_issue_url)
        self.assertIn("security_patching_open_issues_url:", build_open_issue_url)
        self.assertIn("{{ security_patching_issues_url }}", build_open_issue_url)

    def test_issue_report_visible_rows_are_bounded(self):
        defaults = DEFAULTS_PATH.read_text(encoding="utf-8")
        template = REPORT_TEMPLATE_PATH.read_text(encoding="utf-8")

        self.assertIn("security_patching_report_max_detail_rows: 50", defaults)
        self.assertIn("security_patching_report_max_cves_per_row: 4", defaults)
        self.assertIn("security_patching_report_max_body_bytes: 60000", defaults)
        self.assertIn(
            "shown_entries = report_entries[:security_patching_report_max_detail_rows | int]",
            template,
        )
        self.assertIn(
            "shown_cves = entry_cves[:security_patching_report_max_cves_per_row | int]",
            template,
        )
        self.assertIn(
            "complete approved advisory ID set is preserved in the hidden scan metadata",
            template,
        )
        self.assertIn("## Advisory review summary", template)
        self.assertIn("advisory_ids: {{ hostvars[host].security_patching_update_advisory_ids", template)
        self.assertIn("Ksplice-aware advisories", template)
        self.assertIn("| to_json }}", template)

    def test_issue_body_is_checked_before_github_write(self):
        body_check = task_block("Require GitHub issue body within safety limit")
        read_report = task_block("Read rendered production security update issue body")
        update_issue = task_block("Update existing production security update issue")
        create_issue = task_block("Create production security update issue")

        self.assertIn("security_patching_report_max_body_bytes", body_check)
        self.assertIn("security_patching_report_body:", read_report)
        self.assertIn("body: \"{{ security_patching_report_body }}\"", update_issue)
        self.assertIn("body: \"{{ security_patching_report_body }}\"", create_issue)

    def test_apply_request_metadata_extraction_uses_quoted_regex_result(self):
        validate_request = VALIDATE_REQUEST_TASKS_PATH.read_text(encoding="utf-8")
        extract_metadata = EXTRACT_METADATA_TASKS_PATH.read_text(encoding="utf-8")

        self.assertIn("ansible.builtin.import_tasks: extract_issue_metadata.yml", validate_request)
        self.assertNotIn("security_patching_metadata_blocks: >-", extract_metadata)
        self.assertIn('security_patching_metadata_blocks: "{{', extract_metadata)
        self.assertIn(r"[\r\n]+(.*?)[\r\n]+-->", extract_metadata)
        self.assertIn("security_patching_metadata_marker_count", extract_metadata)
        self.assertIn("parsed block(s)", extract_metadata)

    def test_metadata_validation_fixture_is_in_ci_surface(self):
        render_test = REPORT_RENDER_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        metadata_test = REQUEST_METADATA_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")

        self.assertNotIn("security_patching_render_test_metadata_blocks: >-", render_test)
        self.assertIn("tasks_from: extract_issue_metadata", metadata_test)
        self.assertIn("security_patching_request_metadata.scan_id", metadata_test)
        self.assertIn("advisory_ids", metadata_test)

    def test_apply_path_uses_ksplice_and_advisory_scoped_dnf(self):
        patch_tasks = PATCH_TASKS_PATH.read_text(encoding="utf-8")

        self.assertIn("security_patching_approved_advisory_ids", patch_tasks)
        self.assertIn("security_patching_current_advisory_ids == security_patching_approved_advisory_ids", patch_tasks)
        self.assertIn("ksplice", patch_tasks)
        self.assertIn("tasks_from: scan", patch_tasks)
        self.assertIn("upgrade-minimal", patch_tasks)
        self.assertIn("--security", patch_tasks)
        self.assertIn("--advisories={{ security_patching_remaining_approved_advisory_ids | join(',') }}", patch_tasks)
        self.assertNotIn("security_patching_approved_package_specs", patch_tasks)

    def test_scan_defaults_use_release_scoped_oval_and_configurable_ssh_target(self):
        defaults = DEFAULTS_PATH.read_text(encoding="utf-8")
        scan_tasks = TASKS_PATH.with_name("scan.yml").read_text(encoding="utf-8")

        self.assertIn("com.oracle.elsa-ol10.xml.bz2", defaults)
        self.assertNotIn("com.oracle.elsa-all.xml.bz2", defaults)
        self.assertIn("security_patching_oscap_ssh_target", defaults)
        self.assertIn('"{{ security_patching_oscap_ssh_target }}"', scan_tasks)
        self.assertNotIn("default('opc') }}@", scan_tasks)


if __name__ == "__main__":
    unittest.main()
