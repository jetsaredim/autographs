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

    candidates_by_id = {}
    keep_ids = set()
    candidates_by_repository = {repository: {} for repository in repositories}

    for image in images:
        image_id = image_id_for(image)
        refs = image_refs(image)
        matched_repositories = {
            repository
            for repository in repositories
            if any(ref.startswith(f"{repository}:") or ref.startswith(f"{repository}@") for ref in refs)
        }

        if not matched_repositories or not image_id:
            continue

        digest_refs = set(repo_digests(image))
        candidates_by_id[image_id] = image
        for repository in matched_repositories:
            candidates_by_repository[repository][image_id] = image

        if (
            current_image in refs
            or current_image in digest_refs
            or any(tag in protected_tags or tag == "latest" for tag in tags(refs))
        ):
            keep_ids.add(image_id)

    newest_first = sorted(candidates_by_id.values(), key=created_at, reverse=True)

    for repository_candidates in candidates_by_repository.values():
        repository_newest_first = sorted(repository_candidates.values(), key=created_at, reverse=True)
        for image in repository_newest_first[:retain_count]:
            keep_ids.add(image_id_for(image))

    for image in newest_first:
        image_id = image_id_for(image)
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


def image_id_for(image: dict) -> str:
    return image.get("Id") or image.get("ID") or image.get("id") or ""


def image_refs(image: dict) -> list[str]:
    refs = []
    repository = image.get("Repository", "")
    tag = image.get("Tag", "")

    if repository and tag:
        refs.append(f"{repository}:{tag}")

    for key in ("Names", "RepoTags", "names", "repoTags"):
        value = image.get(key) or []
        if isinstance(value, str):
            refs.append(value)
        elif isinstance(value, list):
            refs.extend(item for item in value if isinstance(item, str))

    refs.extend(repo_digests(image))
    digest = image.get("Digest") or image.get("digest")
    if digest:
        for ref in list(refs):
            repository = repository_for_tag_ref(ref)
            if repository:
                refs.append(f"{repository}@{digest}")

    return refs


def repository_for_tag_ref(ref: str) -> str:
    if "@" in ref:
        return ""
    repository, separator, _tag = ref.rpartition(":")
    if separator:
        return repository
    return ""


def tags(refs: list[str]) -> list[str]:
    values = []
    for ref in refs:
        if "@" in ref:
            continue
        _, separator, tag = ref.rpartition(":")
        if separator:
            values.append(tag)
    return values


if __name__ == "__main__":
    raise SystemExit(main())
