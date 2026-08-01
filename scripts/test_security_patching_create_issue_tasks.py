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


if __name__ == "__main__":
    unittest.main()
