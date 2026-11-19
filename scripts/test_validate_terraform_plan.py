"""Behavior tests for the production Terraform plan mutation gate."""

import unittest

from scripts.validate_terraform_plan import validate


def plan(*changes):
    return {
        "format_version": "1.2",
        "resource_changes": [
            {"mode": mode, "address": address, "change": {"actions": actions}}
            for mode, address, actions in changes
        ]
    }


class TerraformPlanPolicyTests(unittest.TestCase):
    def test_allows_normal_convergence_and_data_reads(self):
        self.assertEqual(
            validate(plan(
                ("managed", "module.compute.oci_core_instance.runtime", ["update"]),
                ("managed", "oci_core_network_security_group.new", ["create"]),
                ("data", "data.oci_identity_availability_domains.current", ["read"]),
            )),
            [],
        )

    def test_rejects_deletes_replacements_and_state_removal(self):
        changes = plan(
            ("managed", "module.data_services.oci_database_autonomous_database.this", ["delete"]),
            ("managed", "oci_objectstorage_bucket.media", ["delete", "create"]),
            ("managed", "porkbun_dns_record.autographs", ["forget"]),
        )
        self.assertEqual(len(validate(changes)), 3)

    def test_retry_exception_is_only_for_runtime_replacement(self):
        runtime = ("managed", "module.compute.oci_core_instance.runtime", ["delete", "create"])
        legacy_runtime = ("managed", "module.compute.oci_core_instance.runtime[0]", ["create", "delete"])
        self.assertEqual(len(validate(plan(runtime))), 1)
        self.assertEqual(validate(plan(runtime, legacy_runtime), True), [])
        self.assertEqual(
            len(validate(plan(runtime, ("managed", "oci_core_vcn.network", ["delete"])), True)),
            1,
        )
        self.assertEqual(len(validate(plan(("managed", runtime[1], ["delete"])), True)), 1)

    def test_rejects_malformed_plan(self):
        self.assertTrue(validate({}))
        self.assertTrue(validate({"format_version": "1.2", "resource_changes": [{}]}))
        self.assertEqual(validate({"format_version": "1.2"}), [])


if __name__ == "__main__":
    unittest.main()
