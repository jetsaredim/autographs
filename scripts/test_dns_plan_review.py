import copy
import base64
import json
import unittest
from unittest.mock import patch

from scripts import dns_plan_review


STATE = {
    "lineage": "production-lineage",
    "serial": 42,
    "cloudflare_zone_present": True,
}
PLAN = {
    "resource_changes": [
        {
            "address": "cloudflare_dns_record.autographs[0]",
            "mode": "managed",
            "change": {
                "actions": ["update"],
                "before": {"proxied": False},
                "after": {"proxied": True},
            },
        },
        {
            "address": "oci_core_instance.runtime",
            "mode": "managed",
            "change": {"actions": ["no-op"]},
        },
    ]
}


class DnsPlanReviewTests(unittest.TestCase):
    def test_review_record_covers_dns_values_and_state(self):
        record = dns_plan_review.review_record(
            PLAN, STATE, "tree-a", "head-a", 100, "proxied"
        )
        self.assertEqual(record["dns_change_count"], 1)
        changed = copy.deepcopy(PLAN)
        changed["resource_changes"][0]["change"]["after"]["proxied"] = False
        self.assertNotEqual(
            record["dns_change_sha256"],
            dns_plan_review.review_record(
                changed, STATE, "tree-a", "head-a", 100, "proxied"
            )["dns_change_sha256"],
        )
        with self.assertRaisesRegex(ValueError, "mode off"):
            dns_plan_review.review_record(PLAN, STATE, "tree-a", "head-a", 100, "off")

    def test_deploy_rejects_post_review_dns_or_state_drift(self):
        reviewed = dns_plan_review.review_record(
            PLAN, STATE, "tree-a", "head-a", 100, "proxied"
        )
        with patch.object(dns_plan_review, "reviewed_ci_record", return_value=reviewed):
            dns_plan_review.verify(
                PLAN, STATE, "tree-a", "proxied", "owner/repo", "merge-a"
            )
            drifted = copy.deepcopy(PLAN)
            drifted["resource_changes"][0]["change"]["after"]["proxied"] = False
            with self.assertRaisesRegex(ValueError, "differ"):
                dns_plan_review.verify(
                    drifted, STATE, "tree-a", "proxied", "owner/repo", "merge-a"
                )
            changed_state = {**STATE, "serial": 43}
            with self.assertRaisesRegex(ValueError, "differ"):
                dns_plan_review.verify(
                    PLAN, changed_state, "tree-a", "proxied", "owner/repo", "merge-a"
                )
            with self.assertRaisesRegex(ValueError, "differ"):
                dns_plan_review.verify(
                    PLAN, STATE, "tree-b", "proxied", "owner/repo", "merge-a"
                )
        with self.assertRaisesRegex(ValueError, "mode off"):
            dns_plan_review.verify(PLAN, STATE, "tree-a", "off", "owner/repo", "merge-a")

    def test_no_dns_change_needs_no_release_lookup(self):
        plan = {"resource_changes": [{"address": "oci_core_instance.runtime", "mode": "managed", "change": {"actions": ["update"]}}]}
        with patch.object(dns_plan_review, "reviewed_ci_record") as lookup:
            dns_plan_review.verify(
                plan, STATE, "tree-a", "proxied", "owner/repo", "merge-a"
            )
            lookup.assert_not_called()

    def test_only_latest_matching_release_pr_ci_can_authorize(self):
        pr = {
            "number": 100,
            "merged_at": "now",
            "merge_commit_sha": "merge-a",
            "base": {"ref": "main"},
            "head": {"sha": "head-a"},
        }
        runs = {"workflow_runs": [
            {"id": 2, "path": ".github/workflows/ci.yml", "status": "completed", "conclusion": "failure", "pull_requests": [{"number": 100}]},
            {"id": 1, "path": ".github/workflows/ci.yml", "status": "completed", "conclusion": "success", "pull_requests": [{"number": 100}]},
        ]}
        with patch.object(dns_plan_review, "gh_api", side_effect=[[pr], runs]):
            with self.assertRaisesRegex(ValueError, "latest Release PR CI run did not pass"):
                dns_plan_review.reviewed_ci_record("owner/repo", "merge-a")

    def test_successful_release_pr_ci_record_is_bound_to_head(self):
        pr = {
            "number": 100,
            "merged_at": "now",
            "merge_commit_sha": "merge-a",
            "base": {"ref": "main"},
            "head": {"sha": "head-a"},
        }
        run = {"id": 2, "path": ".github/workflows/ci.yml", "status": "completed", "conclusion": "success", "pull_requests": [{"number": 100}]}
        record = dns_plan_review.review_record(PLAN, STATE, "tree-a", "head-a", 100, "proxied")
        encoded = base64.urlsafe_b64encode(json.dumps(record).encode()).decode().rstrip("=")
        responses = [
            [pr],
            {"workflow_runs": [run]},
            {"jobs": [{"id": 3, "name": "Terraform checks", "conclusion": "success"}]},
            f"2026-11-21T00:00:00Z {dns_plan_review.MARKER}{encoded}\n",
        ]
        with patch.object(dns_plan_review, "gh_api", side_effect=responses) as api:
            self.assertEqual(dns_plan_review.reviewed_ci_record("owner/repo", "merge-a"), record)
            self.assertEqual(api.call_count, 4)
        stale = {**record, "head": "head-old"}
        responses[-1] = f"2026-11-21T00:00:00Z {dns_plan_review.MARKER}{base64.urlsafe_b64encode(json.dumps(stale).encode()).decode().rstrip('=')}\n"
        with patch.object(dns_plan_review, "gh_api", side_effect=responses):
            with self.assertRaisesRegex(ValueError, "does not match"):
                dns_plan_review.reviewed_ci_record("owner/repo", "merge-a")


if __name__ == "__main__":
    unittest.main()
