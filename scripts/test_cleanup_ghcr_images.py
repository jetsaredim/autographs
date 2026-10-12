#!/usr/bin/env python3

import base64
import contextlib
import importlib.util
import io
import json
import os
import tempfile
import unittest
from unittest.mock import patch
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("cleanup-ghcr-images.py")
spec = importlib.util.spec_from_file_location("cleanup_ghcr_images", SCRIPT_PATH)
cleanup_ghcr_images = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(cleanup_ghcr_images)


def no_children(_version):
    return []


def make_version(version_id, name, tags, created_at="2020-01-01T00:00:00Z"):
    return {
        "id": version_id,
        "name": name,
        "created_at": created_at,
        "metadata": {"container": {"tags": tags}},
    }


def ids_to_delete(result):
    return sorted(version["id"] for version, reasons in result if not reasons)


def reasons_for(result, version_id):
    return next(reasons for version, reasons in result if version["id"] == version_id)


class CleanupGhcrImagesTests(unittest.TestCase):
    def test_missing_status_stops_cleanup(self):
        with tempfile.TemporaryDirectory() as tmp:
            with patch.dict(os.environ, {"GHCR_CLEANUP_RELEASE_STATUS_PATH": str(Path(tmp) / "missing")}):
                with self.assertRaises(FileNotFoundError):
                    cleanup_ghcr_images.read_release_status()

    def test_remote_cleanup_defaults_to_inventory(self):
        version = {"id": 1, "created_at": "2020-01-01T00:00:00Z", "metadata": {"container": {"tags": ["v1.0.0"]}}}
        with patch.dict(os.environ, {"GITHUB_TOKEN": "test", "GHCR_IMAGE_REPOSITORY": "ghcr.io/example/controller"}, clear=True):
            with patch.object(cleanup_ghcr_images, "read_release_status", return_value={"deployedControllerVersion": "v2.0.0"}), patch.object(cleanup_ghcr_images, "list_package_versions", return_value=[version]), patch.object(cleanup_ghcr_images, "select_versions", return_value=[(version, [])]), patch.object(cleanup_ghcr_images, "delete_package_version") as delete:
                cleanup_ghcr_images.main()
                delete.assert_not_called()

    def test_remote_delete_succeeds_and_counts_deletions(self):
        version = {"id": 1, "created_at": "2020-01-01T00:00:00Z", "metadata": {"container": {"tags": ["v1.0.0"]}}}
        env = {
            "GITHUB_TOKEN": "test",
            "GHCR_IMAGE_REPOSITORY": "ghcr.io/example/controller",
            "GHCR_CLEANUP_DRY_RUN": "false",
        }
        output = io.StringIO()
        with patch.dict(os.environ, env, clear=True):
            with patch.object(cleanup_ghcr_images, "read_release_status", return_value={"deployedControllerVersion": "v2.0.0"}), patch.object(cleanup_ghcr_images, "list_package_versions", return_value=[version]), patch.object(cleanup_ghcr_images, "select_versions", return_value=[(version, [])]), patch.object(cleanup_ghcr_images, "delete_package_version", return_value=True), contextlib.redirect_stdout(output):
                exit_code = cleanup_ghcr_images.main()
        self.assertEqual(exit_code, 0)
        self.assertIn("Deleted GHCR version 1", output.getvalue())
        self.assertIn("1 deleted, 0 kept", output.getvalue())

    def test_remote_delete_reports_404_without_claiming_deletion(self):
        version = {"id": 1, "created_at": "2020-01-01T00:00:00Z", "metadata": {"container": {"tags": ["v1.0.0"]}}}
        env = {
            "GITHUB_TOKEN": "test",
            "GHCR_IMAGE_REPOSITORY": "ghcr.io/example/controller",
            "GHCR_CLEANUP_DRY_RUN": "false",
        }
        output = io.StringIO()
        with patch.dict(os.environ, env, clear=True):
            with patch.object(cleanup_ghcr_images, "read_release_status", return_value={"deployedControllerVersion": "v2.0.0"}), patch.object(cleanup_ghcr_images, "list_package_versions", return_value=[version]), patch.object(cleanup_ghcr_images, "select_versions", return_value=[(version, [])]), patch.object(cleanup_ghcr_images, "delete_package_version", return_value=False) as delete, contextlib.redirect_stdout(output):
                exit_code = cleanup_ghcr_images.main()
        delete.assert_called_once()
        self.assertEqual(exit_code, 1)
        self.assertIn("returned 404", output.getvalue())
        self.assertIn("0 deleted, 0 kept, 1 not deleted (404)", output.getvalue())
        self.assertNotIn("Deleted GHCR version", output.getvalue())

    def test_active_previous_and_digest_are_protected(self):
        versions = [{"id": n, "name": digest, "created_at": "2020-01-01T00:00:00Z",
                     "metadata": {"container": {"tags": tags}}}
                    for n, digest, tags in [(1, "a", ["v1.2.0"]), (2, "b", ["v1.1.0"]),
                                            (3, "c", ["other"]), (4, "d", ["v1.0.0"])]]
        status = {"deployedControllerVersion": "v1.2.0", "previousControllerVersion": "v1.1.0",
                  "previousControllerDigest": "c"}
        result = cleanup_ghcr_images.select_versions(versions, status, 0, 0, float("inf"), set(), no_children)
        self.assertEqual(ids_to_delete(result), [4])

    def test_protected_tags_are_retained(self):
        versions = [
            make_version(1, "a", ["v2.0.0"]),
            make_version(2, "b", ["keepme"]),
            make_version(3, "c", ["v1.0.0"]),
        ]
        status = {"deployedControllerVersion": "v2.0.0"}
        result = cleanup_ghcr_images.select_versions(
            versions, status, 0, 0, float("inf"), {"keepme"}, no_children
        )
        self.assertEqual(ids_to_delete(result), [3])

    def test_recent_versions_are_retained_by_minimum_age(self):
        versions = [
            make_version(1, "a", ["v2.0.0"]),
            make_version(2, "b", ["v1.0.0"]),
            make_version(3, "c", []),
        ]
        status = {"deployedControllerVersion": "v2.0.0"}
        # cutoff of 0 means every 2020-dated version is "newer than the minimum age"
        result = cleanup_ghcr_images.select_versions(versions, status, 0, 0, 0, set(), no_children)
        self.assertEqual(ids_to_delete(result), [])
        self.assertIn("newer than minimum age", reasons_for(result, 3))

    def test_untagged_manifest_referenced_by_retained_index_is_retained(self):
        versions = [
            make_version(1, "sha256:index", ["v1.0.0"]),
            make_version(2, "sha256:image", []),
            make_version(3, "sha256:attestation", []),
        ]
        children = {"sha256:index": ["sha256:image", "sha256:attestation"]}
        status = {"deployedControllerVersion": "v1.0.0"}
        result = cleanup_ghcr_images.select_versions(
            versions, status, 1, 1, float("inf"), set(), lambda v: children.get(v["name"], [])
        )
        self.assertEqual(ids_to_delete(result), [])
        self.assertIn("referenced by retained version 1", reasons_for(result, 2))
        self.assertIn("referenced by retained version 1", reasons_for(result, 3))

    def test_unreferenced_untagged_manifest_is_deleted(self):
        versions = [
            make_version(1, "sha256:index", ["v1.0.0"]),
            make_version(2, "sha256:orphan", []),
        ]
        status = {"deployedControllerVersion": "v1.0.0"}
        result = cleanup_ghcr_images.select_versions(
            versions, status, 1, 1, float("inf"), set(), no_children
        )
        self.assertEqual(ids_to_delete(result), [2])

    def test_children_of_a_deleted_index_are_deleted_with_it(self):
        versions = [
            make_version(1, "sha256:new-index", ["v2.0.0"], "2020-03-01T00:00:00Z"),
            make_version(2, "sha256:new-image", [], "2020-03-01T00:00:00Z"),
            make_version(3, "sha256:new-attestation", [], "2020-03-01T00:00:00Z"),
            make_version(4, "sha256:old-index", ["v1.0.0"], "2020-01-01T00:00:00Z"),
            make_version(5, "sha256:old-image", [], "2020-01-01T00:00:00Z"),
            make_version(6, "sha256:old-attestation", [], "2020-01-01T00:00:00Z"),
            make_version(7, "sha256:orphan-a", [], "2019-12-01T00:00:00Z"),
            make_version(8, "sha256:orphan-b", [], "2019-12-01T00:00:00Z"),
        ]
        children = {
            "sha256:new-index": ["sha256:new-image", "sha256:new-attestation"],
            "sha256:old-index": ["sha256:old-image", "sha256:old-attestation"],
        }
        status = {"deployedControllerVersion": "v2.0.0"}
        result = cleanup_ghcr_images.select_versions(
            versions, status, 1, 1, float("inf"), set(), lambda v: children.get(v["name"], [])
        )
        self.assertEqual(ids_to_delete(result), [4, 5, 6, 7, 8])

    def test_retention_count_only_counts_tagged_versions(self):
        # Untagged children are the newest versions; they must not use up the retention count.
        versions = [
            make_version(1, "sha256:untagged-a", [], "2020-04-01T00:00:00Z"),
            make_version(2, "sha256:untagged-b", [], "2020-04-01T00:00:00Z"),
            make_version(3, "sha256:v3", ["v3.0.0"], "2020-03-01T00:00:00Z"),
            make_version(4, "sha256:v2", ["v2.0.0"], "2020-02-01T00:00:00Z"),
            make_version(5, "sha256:v1", ["v1.0.0"], "2020-01-01T00:00:00Z"),
        ]
        status = {"deployedControllerVersion": "v3.0.0"}
        result = cleanup_ghcr_images.select_versions(
            versions, status, 2, 2, float("inf"), set(), no_children
        )
        self.assertEqual(ids_to_delete(result), [1, 2, 5])
        self.assertIn("among 2 newest tagged versions", reasons_for(result, 4))

    def test_unreadable_manifest_aborts_before_any_selection(self):
        versions = [make_version(1, "sha256:index", ["v1.0.0"]), make_version(2, "sha256:other", [])]

        def unreadable(_version):
            raise RuntimeError("Could not read manifest")

        with self.assertRaises(RuntimeError):
            cleanup_ghcr_images.select_versions(
                versions, {"deployedControllerVersion": "v1.0.0"}, 1, 1, float("inf"), set(), unreadable
            )

    def test_stale_deployed_status_stops_cleanup(self):
        versions = [make_version(1, "sha256:index", ["v1.0.0"])]
        with self.assertRaisesRegex(RuntimeError, "v9.9.9"):
            cleanup_ghcr_images.select_versions(
                versions, {"deployedControllerVersion": "v9.9.9"}, 1, 1, float("inf"), set(), no_children
            )

    def test_semver_tags_only_match_release_versions(self):
        self.assertEqual(
            cleanup_ghcr_images.semver_tags(["v0.7.1", "latest", "v1.2", "v1.2.3-extra"]),
            ["v0.7.1"],
        )

    def test_delete_package_version_distinguishes_deleted_missing_and_failed(self):
        package_info = {"owner": "example", "package_name": "controller", "owner_kind": "users"}
        with patch.object(cleanup_ghcr_images, "github_request", return_value=(204, {}, "")):
            self.assertTrue(cleanup_ghcr_images.delete_package_version("t", package_info, "1"))
        with patch.object(cleanup_ghcr_images, "github_request", return_value=(404, {}, "")):
            self.assertFalse(cleanup_ghcr_images.delete_package_version("t", package_info, "1"))
        with patch.object(cleanup_ghcr_images, "github_request", return_value=(500, {}, "boom")):
            with self.assertRaises(RuntimeError):
                cleanup_ghcr_images.delete_package_version("t", package_info, "1")

    def test_child_resolver_reads_index_children_from_registry(self):
        package_info = {"owner": "Example", "package_name": "autographs/controller", "owner_kind": "users"}
        index = {"manifests": [{"digest": "sha256:image"}, {"digest": "sha256:attestation"}]}
        calls = []

        def fake_get(url, headers):
            calls.append((url, headers))
            if url.startswith("https://ghcr.io/token"):
                return 200, json.dumps({"token": "registry-token"})
            return 200, json.dumps(index)

        with patch.dict(os.environ, {"GITHUB_ACTOR": "octo"}, clear=True), patch.object(cleanup_ghcr_images, "http_get", side_effect=fake_get):
            resolve = cleanup_ghcr_images.make_child_resolver("gh-token", package_info)
            children = resolve({"name": "sha256:index"})
            resolve({"name": "sha256:index"})

        self.assertEqual(children, ["sha256:image", "sha256:attestation"])
        token_url, token_headers = calls[0]
        self.assertIn("repository%3Aexample%2Fautographs%2Fcontroller%3Apull", token_url)
        expected_basic = base64.b64encode(b"octo:gh-token").decode()
        self.assertEqual(token_headers["Authorization"], f"Basic {expected_basic}")
        manifest_url, manifest_headers = calls[1]
        self.assertEqual(manifest_url, "https://ghcr.io/v2/example/autographs/controller/manifests/sha256:index")
        self.assertEqual(manifest_headers["Authorization"], "Bearer registry-token")
        self.assertIn("application/vnd.oci.image.index.v1+json", manifest_headers["Accept"])
        # the registry token is fetched once and reused
        self.assertEqual(sum(1 for url, _ in calls if url.startswith("https://ghcr.io/token")), 1)

    def test_child_resolver_returns_nothing_for_a_plain_image_manifest(self):
        package_info = {"owner": "example", "package_name": "controller", "owner_kind": "users"}

        def fake_get(url, headers):
            if url.startswith("https://ghcr.io/token"):
                return 200, json.dumps({"token": "t"})
            return 200, json.dumps({"schemaVersion": 2, "layers": []})

        with patch.object(cleanup_ghcr_images, "http_get", side_effect=fake_get):
            resolve = cleanup_ghcr_images.make_child_resolver("gh-token", package_info)
            self.assertEqual(resolve({"name": "sha256:image"}), [])

    def test_child_resolver_raises_when_manifest_cannot_be_read(self):
        package_info = {"owner": "example", "package_name": "controller", "owner_kind": "users"}

        def fake_get(url, headers):
            if url.startswith("https://ghcr.io/token"):
                return 200, json.dumps({"token": "t"})
            return 404, "not found"

        with patch.object(cleanup_ghcr_images, "http_get", side_effect=fake_get):
            resolve = cleanup_ghcr_images.make_child_resolver("gh-token", package_info)
            with self.assertRaises(RuntimeError):
                resolve({"name": "sha256:missing"})

    def test_child_resolver_raises_when_registry_token_is_refused(self):
        package_info = {"owner": "example", "package_name": "controller", "owner_kind": "users"}
        with patch.object(cleanup_ghcr_images, "http_get", return_value=(401, "denied")):
            resolve = cleanup_ghcr_images.make_child_resolver("gh-token", package_info)
            with self.assertRaises(RuntimeError):
                resolve({"name": "sha256:index"})

    def test_deployed_controller_version_prefers_explicit_env(self):
        old_value = os.environ.get("GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION")
        try:
            os.environ["GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION"] = "v9.9.9"
            self.assertEqual(cleanup_ghcr_images.deployed_controller_version_from_status(), "v9.9.9")
        finally:
            if old_value is None:
                os.environ.pop("GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION", None)
            else:
                os.environ["GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION"] = old_value

    def test_deployed_controller_version_reads_release_status(self):
        old_explicit = os.environ.pop("GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION", None)
        old_path = os.environ.get("GHCR_CLEANUP_RELEASE_STATUS_PATH")
        try:
            with tempfile.TemporaryDirectory() as tmp:
                status = Path(tmp) / ".release-status.json"
                status.write_text(
                    json.dumps({"deployedControllerVersion": "v1.2.3"}),
                    encoding="utf-8",
                )
                os.environ["GHCR_CLEANUP_RELEASE_STATUS_PATH"] = str(status)
                self.assertEqual(cleanup_ghcr_images.deployed_controller_version_from_status(), "v1.2.3")
        finally:
            if old_explicit is not None:
                os.environ["GHCR_CLEANUP_DEPLOYED_CONTROLLER_VERSION"] = old_explicit
            if old_path is None:
                os.environ.pop("GHCR_CLEANUP_RELEASE_STATUS_PATH", None)
            else:
                os.environ["GHCR_CLEANUP_RELEASE_STATUS_PATH"] = old_path


if __name__ == "__main__":
    unittest.main()
