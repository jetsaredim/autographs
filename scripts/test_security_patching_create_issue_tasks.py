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
CLASSIFY_FINDINGS_TASKS_PATH = TASKS_PATH.with_name("classify_findings.yml")
POST_RESULT_TASKS_PATH = TASKS_PATH.with_name("post_result.yml")
REBOOT_CLEANUP_TASKS_PATH = TASKS_PATH.with_name("reboot_cleanup.yml")
FAILED_CLEANUP_TASKS_PATH = TASKS_PATH.with_name("cleanup_failed_request.yml")
VALIDATE_REBOOT_STATE_TASKS_PATH = TASKS_PATH.with_name("validate_reboot_state.yml")
POST_REBOOT_RESULT_TASKS_PATH = TASKS_PATH.with_name("post_reboot_result.yml")
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
CREATE_ISSUE_STATUS_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-create-issue-status-validate-test.yml"
)
POST_RESULT_STATUS_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-post-result-status-validate-test.yml"
)
POST_RESULT_REFRESH_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-post-result-refresh-validate-test.yml"
)
TARGET_SCOPE_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-target-scope-validate-test.yml"
)
REBOOT_STATE_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-reboot-state-validate-test.yml"
)
REBOOT_RESULT_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-reboot-result-validate-test.yml"
)
REBOOT_PREFLIGHT_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-reboot-preflight-validate-test.yml"
)
REBOOT_KERNEL_SELECTION_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-reboot-kernel-selection-validate-test.yml"
)
CLASSIFICATION_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-finding-classification-validate-test.yml"
)
KERNEL_POSTURE_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-kernel-posture-validate-test.yml"
)
UPDATE_RECONCILIATION_TEST_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name(
    "security-update-reconciliation-validate-test.yml"
)
WORKFLOWS_PATH = Path(__file__).resolve().parents[1] / ".github" / "workflows"
SCAN_WORKFLOW_PATH = WORKFLOWS_PATH / "weekly-security-scan.yml"
UPDATE_WORKFLOW_PATH = WORKFLOWS_PATH / "apply-security-updates.yml"
REBOOT_WORKFLOW_PATH = WORKFLOWS_PATH / "reboot-security-runtime.yml"
PATCH_PLAYBOOK_PATH = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name("security-patch.yml")


def task_block(task_name: str) -> str:
    text = TASKS_PATH.read_text(encoding="utf-8")
    pattern = re.compile(rf"^- name: {re.escape(task_name)}\n(?P<body>.*?)(?=^- name: |\Z)", re.M | re.S)
    match = pattern.search(text)
    if not match:
        raise AssertionError(f"Task not found: {task_name}")
    return match.group("body")


def task_block_from(path: Path, task_name: str) -> str:
    text = path.read_text(encoding="utf-8")
    pattern = re.compile(rf"^- name: {re.escape(task_name)}\n(?P<body>.*?)(?=^- name: |\Z)", re.M | re.S)
    match = pattern.search(text)
    if not match:
        raise AssertionError(f"Task not found in {path}: {task_name}")
    return match.group("body")


def nested_task_block_from(path: Path, task_name: str) -> str:
    text = path.read_text(encoding="utf-8")
    pattern = re.compile(
        rf"^    - name: {re.escape(task_name)}\n(?P<body>.*?)(?=^    - name: |\Z)",
        re.M | re.S,
    )
    match = pattern.search(text)
    if not match:
        raise AssertionError(f"Nested task not found in {path}: {task_name}")
    return match.group("body")


class SecurityPatchingCreateIssueTasksTests(unittest.TestCase):
    def test_report_approval_labels_are_published_after_action_label_resolution(self):
        task_pairs = (
            (
                TASKS_PATH,
                "Resolve scanner issue approval label",
                "Publish scanner issue approval label",
            ),
            (
                POST_RESULT_TASKS_PATH,
                "Resolve aggregate post-update approval label",
                "Publish aggregate post-update approval label",
            ),
            (
                POST_REBOOT_RESULT_TASKS_PATH,
                "Resolve aggregate post-reboot approval label",
                "Publish aggregate post-reboot approval label",
            ),
        )

        for path, resolve_name, publish_name in task_pairs:
            resolve_block = task_block_from(path, resolve_name)
            publish_block = task_block_from(path, publish_name)
            self.assertIn("security_patching_next_action_label:", resolve_block)
            self.assertNotIn("security_patching_report_approval_label:", resolve_block)
            self.assertIn("security_patching_report_approval_label:", publish_block)
            self.assertIn("security_patching_next_action_label | trim", publish_block)

    def test_issue_url_fact_is_available_before_open_issue_url_uses_it(self):
        build_issue_url = task_block("Build GitHub issues URLs")
        build_open_issue_url = task_block("Build open GitHub issues URL")

        self.assertIn("security_patching_issues_url:", build_issue_url)
        self.assertNotIn("security_patching_open_issues_url:", build_issue_url)
        self.assertIn("security_patching_open_issues_url:", build_open_issue_url)
        self.assertIn("{{ security_patching_issues_url }}", build_open_issue_url)

    def test_issue_report_combines_advisory_summary_and_package_sample(self):
        defaults = DEFAULTS_PATH.read_text(encoding="utf-8")
        template = REPORT_TEMPLATE_PATH.read_text(encoding="utf-8")

        self.assertNotIn("security_patching_report_max_detail_rows", defaults)
        self.assertIn("security_patching_report_max_cves_per_row: 4", defaults)
        self.assertIn("security_patching_report_max_body_bytes: 60000", defaults)
        self.assertIn('security_patching_report_approval_label: "{{ security_patching_approval_label }}"', defaults)
        self.assertIn(
            "report_approval_label = security_patching_report_approval_label | default(security_patching_approval_label)",
            template,
        )
        self.assertIn('approval_label: "{{ report_approval_label | trim }}"', template)
        self.assertIn('next_action: "{{ report_next_action }}"', template)
        self.assertIn(
            "shown_cves = advisory_cves[:security_patching_report_max_cves_per_row | int]",
            template,
        )
        self.assertIn("advisory_packages = advisory.packages | default([])", template)
        self.assertIn("shown_packages = advisory_packages[:4]", template)
        self.assertIn(
            "complete approved advisory ID set is preserved in hidden scanner metadata",
            template,
        )
        self.assertIn("## Advisory review summary", template)
        self.assertNotIn("## Advisory finding detail sample", template)
        self.assertIn("Package sample", template)
        self.assertIn("advisory_ids: {{ hostvars[host].security_patching_update_advisory_ids", template)
        self.assertIn("Ksplice-specific OVAL findings", template)
        self.assertIn("| to_json }}", template)

    def test_issue_body_is_checked_before_github_write(self):
        scan_guard = task_block("Require complete OpenSCAP scan results for all target hosts")
        body_check = task_block("Require GitHub issue body within safety limit")
        read_report = task_block("Read rendered production security update issue body")
        update_issue = task_block("Update existing production security update issue")
        create_issue = task_block("Create production security update issue")
        close_issue = task_block("Close stale production security update issue after clean scan")

        self.assertIn("security_patching_incomplete_scan_hosts | length == 0", scan_guard)
        self.assertIn("security_patching_report_max_body_bytes", body_check)
        self.assertIn("security_patching_report_body:", read_report)
        self.assertIn("body: \"{{ security_patching_report_body }}\"", update_issue)
        self.assertIn("body: \"{{ security_patching_report_body }}\"", create_issue)
        self.assertIn("method: PATCH", close_issue)
        self.assertIn("state: closed", close_issue)
        self.assertIn("patch-scan-open", close_issue)

    def test_result_publication_requires_complete_post_update_scans(self):
        post_result = POST_RESULT_TASKS_PATH.read_text(encoding="utf-8")

        self.assertIn("Require complete post-update scan results for all target hosts", post_result)
        self.assertIn("security_patching_incomplete_post_update_hosts | length == 0", post_result)
        self.assertIn("security_patching_post_update_entries is not defined", post_result)
        self.assertIn("security_patching_post_update_advisory_ids is not defined", post_result)
        self.assertIn("security_patching_post_update_next_action is not defined", post_result)
        self.assertIn("security_patching_post_update_scan_status | default('missing') != 'complete'", post_result)

    def test_partial_apply_refreshes_issue_with_remaining_findings(self):
        post_result = POST_RESULT_TASKS_PATH.read_text(encoding="utf-8")
        result_template = (TASKS_PATH.parents[1] / "templates" / "security-update-result.md.j2").read_text(
            encoding="utf-8"
        )

        self.assertIn("Prepare remaining findings issue refresh context", post_result)
        self.assertIn("Mirror remaining post-update findings into scanner report facts", post_result)
        self.assertIn("security_patching_hosts_with_findings: \"{{ security_patching_remaining_hosts }}\"", post_result)
        self.assertIn("Render refreshed production security update issue body", post_result)
        self.assertIn("src: security-report.md.j2", post_result)
        self.assertIn("Require refreshed GitHub issue body within safety limit", post_result)
        self.assertIn("Reconcile production security update issue with authoritative findings", post_result)
        self.assertIn("url: \"{{ security_patching_issue_url }}\"", post_result)
        self.assertIn("'body': security_patching_refreshed_report_body", post_result)
        self.assertIn('body: "{{ security_patching_reconciled_issue_body }}"', post_result)
        self.assertIn("security_patching_next_action_label", post_result)
        self.assertIn("security_patching_reconciled_issue_body", post_result)
        self.assertIn("else {'state_reason': 'completed'}", post_result)
        self.assertIn("This issue has been refreshed with the remaining findings", result_template)
        self.assertIn("Approval drift reconciled", result_template)
        self.assertIn("No package changes were attempted", result_template)
        self.assertIn("Advisory-scoped DNF still reports package work", result_template)
        self.assertIn("remaining packages are reboot/installonly candidates", result_template)
        self.assertIn("Do not apply an approval label", result_template)

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
        create_status_test = CREATE_ISSUE_STATUS_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        post_result_status_test = POST_RESULT_STATUS_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        post_result_refresh_test = POST_RESULT_REFRESH_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        reboot_state_test = REBOOT_STATE_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        reboot_result_test = REBOOT_RESULT_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        reboot_preflight_test = REBOOT_PREFLIGHT_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        reboot_kernel_selection_test = REBOOT_KERNEL_SELECTION_TEST_PLAYBOOK_PATH.read_text(
            encoding="utf-8"
        )
        non_kernel_fixture = nested_task_block_from(
            REBOOT_STATE_TEST_PLAYBOOK_PATH,
            "Record non-kernel approved reboot issue metadata",
        )
        classification_test = CLASSIFICATION_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        kernel_posture_test = KERNEL_POSTURE_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        update_reconciliation_test = UPDATE_RECONCILIATION_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        target_scope_test = TARGET_SCOPE_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        ci = (Path(__file__).resolve().parents[1] / ".github" / "workflows" / "ci.yml").read_text(
            encoding="utf-8"
        )

        self.assertNotIn("security_patching_render_test_metadata_blocks: >-", render_test)
        self.assertIn("tasks_from: extract_issue_metadata", metadata_test)
        self.assertIn("security_patching_request_metadata.scan_id", metadata_test)
        self.assertIn("advisory_ids", metadata_test)
        self.assertIn("tasks_from: create_issue", create_status_test)
        self.assertIn("degraded-host", create_status_test)
        self.assertIn("security-create-issue-status-validate-test.yml", ci)
        self.assertIn("tasks_from: post_result", post_result_status_test)
        self.assertIn("security-post-result-status-validate-test.yml", ci)
        self.assertIn("tasks_from: post_result", post_result_refresh_test)
        self.assertIn("security_patching_refreshed_report_body", post_result_refresh_test)
        self.assertIn("stale_advisory_id not in security_patching_refreshed_report_body", post_result_refresh_test)
        self.assertIn("security-post-result-refresh-validate-test.yml", ci)
        self.assertIn("tasks_from: validate_reboot_state", reboot_state_test)
        self.assertIn("drifted reboot state blocks mutation", reboot_state_test.lower())
        self.assertIn("non-kernel security reboot state blocks mutation", reboot_state_test.lower())
        self.assertIn("openssl", reboot_state_test)
        self.assertIn("security_patching_scan_status: complete", non_kernel_fixture)
        self.assertIn("security_patching_next_action: reboot", non_kernel_fixture)
        self.assertIn(
            "security_patching_next_action_label: approved-production-reboot",
            non_kernel_fixture,
        )
        self.assertIn("security-reboot-state-validate-test.yml", ci)
        self.assertIn(
            '--start-at-task "Record non-kernel approved reboot issue metadata"',
            ci,
        )
        self.assertIn("tasks_from: post_reboot_result", reboot_result_test)
        self.assertIn("security_patching_reboot_result_comment_body", reboot_result_test)
        self.assertIn("security-reboot-result-validate-test.yml", ci)
        self.assertIn("tasks_from: classify_reboot_request", reboot_preflight_test)
        self.assertIn("late drift prevents cleanup on every target", reboot_preflight_test.lower())
        self.assertIn("security-reboot-preflight-validate-test.yml", ci)
        self.assertIn("RPM-newest installed UEK target", reboot_kernel_selection_test)
        self.assertIn("missing selected UEK initramfs fails closed", reboot_kernel_selection_test)
        self.assertIn("mismatched selected UEK boot entry fails closed", reboot_kernel_selection_test)
        self.assertIn("stale but otherwise valid post-reboot UEK", reboot_kernel_selection_test)
        self.assertIn("security-reboot-kernel-selection-validate-test.yml", ci)
        self.assertIn("tasks_from: classify_findings", classification_test)
        self.assertIn("DNF-applicable", classification_test)
        self.assertIn("reboot-only", classification_test)
        self.assertIn("mixed unclassifiable", classification_test)
        self.assertIn("security-finding-classification-validate-test.yml", ci)
        self.assertIn("failed default-kernel probe remains actionable", kernel_posture_test)
        self.assertIn("security-kernel-posture-validate-test.yml", ci)
        self.assertIn("tasks_from: patch", update_reconciliation_test)
        self.assertIn("security_patching_reconciliation_only", update_reconciliation_test)
        self.assertIn("security-update-reconciliation-validate-test.yml", ci)
        self.assertIn("tasks_from: validate_target_scope", target_scope_test)
        self.assertIn("missing target group", target_scope_test.lower())
        self.assertIn("empty target group", target_scope_test.lower())
        self.assertIn("mismatched hosts", target_scope_test.lower())
        self.assertIn("security-target-scope-validate-test.yml", ci)

    def test_issue_writers_fail_closed_on_empty_or_mismatched_target_scope(self):
        create_issue = TASKS_PATH.read_text(encoding="utf-8")
        classify_update = TASKS_PATH.with_name("classify_update_request.yml").read_text(encoding="utf-8")
        post_result = POST_RESULT_TASKS_PATH.read_text(encoding="utf-8")
        post_reboot = POST_REBOOT_RESULT_TASKS_PATH.read_text(encoding="utf-8")
        validate_request = VALIDATE_REQUEST_TASKS_PATH.read_text(encoding="utf-8")
        target_scope = TASKS_PATH.with_name("validate_target_scope.yml").read_text(encoding="utf-8")

        self.assertIn("security_patching_target_group in groups", target_scope)
        self.assertIn("security_patching_target_hosts | length > 0", target_scope)
        self.assertIn(
            "security_patching_request_instance_hosts == security_patching_target_hosts",
            target_scope,
        )
        for task_text in (create_issue, classify_update, post_result, post_reboot, validate_request):
            self.assertIn("validate_target_scope.yml", task_text)

    def test_apply_path_reports_ksplice_and_uses_advisory_scoped_dnf(self):
        patch_tasks = PATCH_TASKS_PATH.read_text(encoding="utf-8")
        classify_tasks = CLASSIFY_FINDINGS_TASKS_PATH.read_text(encoding="utf-8")
        patch_playbook = PATCH_PLAYBOOK_PATH.read_text(encoding="utf-8")
        update_reconciliation_test = UPDATE_RECONCILIATION_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")

        self.assertIn("security_patching_approved_advisory_ids", patch_tasks)
        self.assertIn("security_patching_update_drift_detected", patch_tasks)
        self.assertIn("security_patching_reconciliation_only", patch_tasks)
        self.assertIn("Preserve authoritative scan state when no package mutation is allowed", patch_tasks)
        self.assertIn("ksplice", patch_tasks)
        self.assertIn("security_patching_ksplice_apply_mode: report_only", patch_tasks)
        self.assertIn("security_patching_ksplice_available", patch_tasks)
        self.assertNotIn("- all\n          - upgrade", patch_tasks)
        self.assertIn("tasks_from: scan", patch_tasks)
        self.assertIn("upgrade-minimal", patch_tasks)
        self.assertIn("--security", patch_tasks)
        self.assertIn("--advisories={{ security_patching_remaining_approved_advisory_ids | join(',') }}", patch_tasks)
        self.assertNotIn("security_patching_approved_package_specs", patch_tasks)
        self.assertIn("--assumeno", classify_tasks)
        self.assertIn("security_patching_classification_dnf_work", classify_tasks)
        self.assertIn("security_patching_reboot_approval_label", classify_tasks)
        self.assertIn("Preflight approved production security updates", patch_playbook)
        self.assertIn("Require every target ready before package mutation", patch_playbook)
        self.assertIn("tasks_from: classify_update_request", patch_playbook)
        self.assertIn("one drifted host disables all package mutation", update_reconciliation_test.lower())
        self.assertLess(
            patch_playbook.index("Scan current host security update state"),
            patch_playbook.index("Apply approved host security updates"),
        )

    def test_reboot_workflow_uses_separate_label_and_installonly_cleanup(self):
        defaults = DEFAULTS_PATH.read_text(encoding="utf-8")
        workflow = REBOOT_WORKFLOW_PATH.read_text(encoding="utf-8")
        reboot_tasks = REBOOT_CLEANUP_TASKS_PATH.read_text(encoding="utf-8")
        failed_cleanup_tasks = FAILED_CLEANUP_TASKS_PATH.read_text(encoding="utf-8")
        validate_reboot_tasks = VALIDATE_REBOOT_STATE_TASKS_PATH.read_text(encoding="utf-8")
        post_reboot_tasks = POST_REBOOT_RESULT_TASKS_PATH.read_text(encoding="utf-8")
        reboot_state_test = REBOOT_STATE_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        reboot_preflight_test = REBOOT_PREFLIGHT_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        reboot_result_test = REBOOT_RESULT_TEST_PLAYBOOK_PATH.read_text(encoding="utf-8")
        classify_reboot_tasks = TASKS_PATH.with_name("classify_reboot_request.yml").read_text(
            encoding="utf-8"
        )
        reboot_playbook = REPORT_RENDER_TEST_PLAYBOOK_PATH.with_name("security-reboot.yml").read_text(
            encoding="utf-8"
        )
        result_template = (TASKS_PATH.parents[1] / "templates" / "security-reboot-result.md.j2").read_text(
            encoding="utf-8"
        )

        self.assertIn("security_patching_reboot_approval_label: approved-production-reboot", defaults)
        self.assertIn("security_patching_reboot_validate_dnf_noop: true", defaults)
        self.assertIn("security_patching_reboot_allowed_package_regex", defaults)
        self.assertIn("security_patching_failure_context_path", defaults)
        self.assertNotIn("security_patching_failure_refresh", defaults)
        self.assertIn("security_patching_failure_context_max_chars", defaults)
        self.assertIn("security_patching_failure_context_stream_max_chars", defaults)
        self.assertIn('security_patching_report_approval_label: "{{ security_patching_approval_label }}"', defaults)
        self.assertIn("name: approved-production-reboot", defaults)
        self.assertIn("github.event.label.name == 'approved-production-reboot'", workflow)
        self.assertIn("SECURITY_PATCHING_FAILURE_CONTEXT_PATH", workflow)
        self.assertNotIn("SECURITY_PATCHING_FAILURE_REFRESH_PATH", workflow)
        self.assertIn("--extra-vars security_patching_approval_label=approved-production-reboot", workflow)
        self.assertIn(
            """--extra-vars '{"security_patching_failed_request_message":"Production security reboot workflow did not complete successfully."}'""",
            workflow,
        )
        self.assertIn("playbook: playbooks/security-reboot.yml", workflow)
        self.assertIn("tasks_from: validate_request", reboot_playbook)
        self.assertIn("tasks_from: validate_reboot_state", reboot_playbook)
        self.assertIn("Preflight every production security reboot target", reboot_playbook)
        self.assertIn("Require every target ready before reboot mutation", reboot_playbook)
        self.assertIn("tasks_from: classify_reboot_request", reboot_playbook)
        self.assertIn("tasks_from: reboot_cleanup", reboot_playbook)
        self.assertGreaterEqual(reboot_playbook.count("tasks_from: scan"), 2)
        self.assertIn("tasks_from: post_reboot_result", reboot_playbook)
        self.assertIn("security_patching_reboot_mutation_allowed", reboot_playbook)
        self.assertIn("security_patching_reboot_preflight_ready", validate_reboot_tasks)
        self.assertIn("security_patching_reboot_reclassified", validate_reboot_tasks)
        self.assertIn("Preserve authoritative preflight state before any reboot mutation", validate_reboot_tasks)
        self.assertIn("security_patching_reboot_added_advisory_ids", validate_reboot_tasks)
        self.assertIn("security_patching_reboot_removed_advisory_ids", validate_reboot_tasks)
        self.assertIn("Write reboot package eligibility failure context", validate_reboot_tasks)
        self.assertIn("Write reboot DNF no-op failure context", validate_reboot_tasks)
        self.assertIn("Build bounded reboot DNF no-op failure evidence", validate_reboot_tasks)
        self.assertIn("security_patching_reboot_dnf_noop_stdout_excerpt", validate_reboot_tasks)
        self.assertIn("security_patching_reboot_allowed_package_regex", validate_reboot_tasks)
        self.assertIn("security_patching_reboot_disallowed_packages", validate_reboot_tasks)
        self.assertIn("dnf", validate_reboot_tasks)
        self.assertIn("--assumeno", validate_reboot_tasks)
        self.assertIn("upgrade-minimal", validate_reboot_tasks)
        self.assertIn("--advisories={{ security_patching_reboot_approved_advisory_ids | join(',') }}", validate_reboot_tasks)
        self.assertIn("Record reboot DNF no-op proof", validate_reboot_tasks)
        self.assertIn("security_patching_reboot_completed: false", validate_reboot_tasks)
        self.assertIn("resolve_reboot_kernel.yml", validate_reboot_tasks)
        self.assertIn("security_patching_reboot_target_kernel_valid", validate_reboot_tasks)
        self.assertIn("security_patching_reboot_reclassified_hosts", classify_reboot_tasks)
        self.assertIn("security_patching_reboot_reconciliation_hosts", classify_reboot_tasks)
        self.assertIn("reclassified for update reconciles without mutation", reboot_preflight_test.lower())
        self.assertIn("reclassified for investigation reconciles without mutation", reboot_preflight_test.lower())
        self.assertIn("reclassified for update refreshes every target", reboot_result_test.lower())
        self.assertIn("reclassified for investigation refreshes issue", reboot_result_test.lower())
        self.assertIn("Reboot approval reclassified", result_template)
        self.assertIn("No reboot or installonly cleanup was attempted", result_template)
        self.assertIn("Validate failed reboot cleanup truncates oversized failure context", reboot_state_test)
        self.assertNotIn("failed request issue refresh", failed_cleanup_tasks.lower())
        self.assertNotIn("security_patching_failure_refresh", failed_cleanup_tasks)
        self.assertNotIn("reboot_advisory_drift", reboot_state_test)
        self.assertIn("Bound failed request details for GitHub comment", failed_cleanup_tasks)
        self.assertIn("Comment failed production security update result", failed_cleanup_tasks)
        self.assertIn("security_patching_failed_request_label_removal_response", failed_cleanup_tasks)
        self.assertIn("ansible.builtin.reboot", reboot_tasks)
        self.assertIn("Select verified newest installed UEK as default", reboot_tasks)
        self.assertIn("Refuse reboot unless newest installed UEK is the verified default", reboot_tasks)
        self.assertIn("--oldinstallonly", reboot_tasks)
        self.assertIn("--setopt=installonly_limit={{ security_patching_installonly_limit | int }}", reboot_tasks)
        self.assertIn("Verify Caddy-fronted admin health after reboot", reboot_tasks)
        self.assertIn("Reconcile production security update issue with authoritative post-reboot findings", post_reboot_tasks)
        self.assertIn("security_patching_reconciled_issue_body", post_reboot_tasks)
        self.assertIn("else {'state_reason': 'completed'}", post_reboot_tasks)
        self.assertIn("security_patching_reboot_result_comment_body", post_reboot_tasks)
        self.assertIn("Collect hosts whose approved reboot advisory set drifted", post_reboot_tasks)
        self.assertIn("Kernel before", result_template)
        self.assertIn("Selected UEK target", result_template)
        self.assertIn("Initramfs verified", result_template)
        self.assertIn("Default changed", result_template)
        self.assertIn("Installonly cleanup", result_template)
        self.assertIn("Approval drift reconciled", result_template)
        self.assertIn("No reboot or installonly cleanup was attempted", result_template)

    def test_scan_defaults_use_release_scoped_oval_and_configurable_ssh_target(self):
        defaults = DEFAULTS_PATH.read_text(encoding="utf-8")
        scan_tasks = TASKS_PATH.with_name("scan.yml").read_text(encoding="utf-8")

        self.assertIn("com.oracle.elsa-ol10.xml.bz2", defaults)
        self.assertNotIn("com.oracle.elsa-all.xml.bz2", defaults)
        self.assertIn("security_patching_oscap_ssh_target", defaults)
        self.assertIn('"{{ security_patching_oscap_ssh_target }}"', scan_tasks)
        self.assertNotIn("default('opc') }}@", scan_tasks)

    def test_all_security_issue_writers_share_one_concurrency_group(self):
        workflows = [
            SCAN_WORKFLOW_PATH.read_text(encoding="utf-8"),
            UPDATE_WORKFLOW_PATH.read_text(encoding="utf-8"),
            REBOOT_WORKFLOW_PATH.read_text(encoding="utf-8"),
        ]

        for workflow in workflows:
            self.assertIn("group: production-security-patching", workflow)
        self.assertNotIn("group: production-security-scan", workflows[0])
        self.assertNotIn("group: production-security-updates", "\n".join(workflows))

    def test_success_paths_reconcile_issue_state_with_idempotent_patch(self):
        create_issue = TASKS_PATH.read_text(encoding="utf-8")
        post_result = POST_RESULT_TASKS_PATH.read_text(encoding="utf-8")
        post_reboot = POST_REBOOT_RESULT_TASKS_PATH.read_text(encoding="utf-8")

        self.assertIn("Close stale production security update issue after clean scan", create_issue)
        self.assertIn("method: PATCH", task_block("Update existing production security update issue"))
        self.assertIn("method: PATCH", task_block("Close stale production security update issue after clean scan"))
        self.assertIn("method: PATCH", post_result)
        self.assertIn("method: PATCH", post_reboot)
        self.assertNotIn("method: DELETE", post_result)
        self.assertNotIn("method: DELETE", post_reboot)


if __name__ == "__main__":
    unittest.main()
