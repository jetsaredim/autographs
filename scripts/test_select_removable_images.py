import json
import os
from pathlib import Path
import subprocess
import unittest


SELECTOR = Path(__file__).resolve().parents[1] / "deploy/ansible/roles/autographs_system_cleanup/files/select-removable-images.py"
REPOSITORY = "ghcr.io/example/controller"


class LocalRetentionTests(unittest.TestCase):
    def select(self, images, current="", previous="", containers="[]"):
        result = subprocess.run(
            ["python3", str(SELECTOR)], capture_output=True, text=True,
            env={**os.environ, "RETAIN_COUNT": "0", "CONTROLLER_IMAGE_REPOSITORY": REPOSITORY,
                 "CURRENT_IMAGES": current, "PREVIOUS_IMAGES": previous,
                 "PROTECTED_TAGS": "", "IMAGES_JSON": json.dumps(images),
                 "CONTAINERS_JSON": containers},
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout.splitlines()

    def test_current_and_previous_tags_protect_multitag_images(self):
        images = [
            {"Id": "active", "Names": [REPOSITORY + ":v1.2.0", REPOSITORY + ":other"]},
            {"Id": "previous", "Names": [REPOSITORY + ":v1.1.0"]},
            {"Id": "old", "Names": [REPOSITORY + ":v1.0.0"]},
        ]
        self.assertEqual(self.select(images, REPOSITORY + ":v1.2.0", REPOSITORY + ":v1.1.0"), ["old"])

    def test_previous_digest_protects_image_without_matching_tag(self):
        digest = "sha256:" + "a" * 64
        images = [{"Id": "previous", "Names": [REPOSITORY + ":old"],
                   "RepoDigests": [REPOSITORY + "@" + digest]}]
        self.assertEqual(self.select(images, previous=REPOSITORY + "@" + digest), [])

    def test_container_referenced_image_is_never_removed(self):
        images = [{"Id": "used", "Names": [REPOSITORY + ":v1.0.0"]}]
        self.assertEqual(self.select(images, containers=json.dumps([{"ImageID": "used"}])), [])


if __name__ == "__main__":
    unittest.main()
