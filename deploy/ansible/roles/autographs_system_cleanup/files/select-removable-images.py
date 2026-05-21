#!/usr/bin/env python3

import json
import os
from datetime import datetime, timezone


def main() -> int:
    retain_count = max(int(os.environ["RETAIN_COUNT"]), 0)
    repositories = {
        os.environ["IMAGE_REPOSITORY"],
        os.environ["TOOLS_IMAGE_REPOSITORY"],
    }
    current_image = os.environ["CURRENT_IMAGE"]
    protected_tags = {
        tag.strip()
        for tag in os.environ.get("PROTECTED_TAGS", "").split(",")
        if tag.strip()
    }
    images = json.loads(os.environ["IMAGES_JSON"])

    candidates = []
    keep_ids = set()
    candidates_by_repository = {repository: [] for repository in repositories}

    for image in images:
        repository = image.get("Repository", "")
        tag = image.get("Tag", "")
        image_id = image.get("Id") or image.get("ID")

        if repository not in repositories or not image_id:
            continue

        ref = f"{repository}:{tag}"
        digest_refs = set(repo_digests(image))
        candidates.append(image)
        candidates_by_repository[repository].append(image)

        if (
            ref == current_image
            or current_image in digest_refs
            or tag == "latest"
            or tag in protected_tags
        ):
            keep_ids.add(image_id)

    newest_first = sorted(candidates, key=created_at, reverse=True)

    for repository_candidates in candidates_by_repository.values():
        repository_newest_first = sorted(repository_candidates, key=created_at, reverse=True)
        for image in repository_newest_first[:retain_count]:
            keep_ids.add(image.get("Id") or image.get("ID"))

    for image in newest_first:
        image_id = image.get("Id") or image.get("ID")
        if image_id not in keep_ids:
            print(image_id)

    return 0


def created_at(image: dict) -> float:
    value = image.get("CreatedAt") or image.get("Created")
    if isinstance(value, int):
        return value
    if not value:
        return 0
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc).timestamp()
    except ValueError:
        return 0


def repo_digests(image: dict) -> list[str]:
    value = image.get("RepoDigests") or image.get("Digests") or []
    if isinstance(value, str):
        return [value]
    if isinstance(value, list):
        return [item for item in value if isinstance(item, str)]
    return []


if __name__ == "__main__":
    raise SystemExit(main())
