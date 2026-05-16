const token = process.env.GITHUB_TOKEN;
const imageRepository = process.env.GHCR_IMAGE_REPOSITORY;
const currentSha = process.env.GITHUB_SHA;
const retainCount = parsePositiveInt(process.env.GHCR_CLEANUP_RETAIN_TAGGED, 10);
const minAgeDays = parsePositiveInt(process.env.GHCR_CLEANUP_MIN_AGE_DAYS, 7);

if (!token) {
  throw new Error("GITHUB_TOKEN is required");
}

if (!imageRepository) {
  throw new Error("GHCR_IMAGE_REPOSITORY is required");
}

const packageInfo = parseGhcrImageRepository(imageRepository);
const versions = await listPackageVersions(packageInfo);
const cutoffTime = Date.now() - minAgeDays * 24 * 60 * 60 * 1000;

const newestFirst = versions.toSorted(
  (left, right) => new Date(right.created_at).getTime() - new Date(left.created_at).getTime(),
);

let deletedCount = 0;

for (const [index, version] of newestFirst.entries()) {
  const tags = version.metadata?.container?.tags ?? [];
  const createdAt = new Date(version.created_at).getTime();
  const keepReasons = [];

  if (index < retainCount) {
    keepReasons.push(`among ${retainCount} newest versions`);
  }

  if (currentSha && tags.includes(currentSha)) {
    keepReasons.push("current deployment sha");
  }

  if (tags.includes("latest")) {
    keepReasons.push("latest tag");
  }

  if (createdAt > cutoffTime) {
    keepReasons.push(`newer than ${minAgeDays} days`);
  }

  if (keepReasons.length > 0) {
    console.log(`Keeping GHCR version ${version.id}: ${formatTags(tags)} (${keepReasons.join(", ")})`);
    continue;
  }

  await deletePackageVersion(packageInfo, version.id);
  deletedCount += 1;
  console.log(`Deleted GHCR version ${version.id}: ${formatTags(tags)}`);
}

console.log(
  `GHCR cleanup complete for ${imageRepository}: ${deletedCount} deleted, ${versions.length - deletedCount} kept`,
);

function parsePositiveInt(value, fallback) {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function parseGhcrImageRepository(repository) {
  const normalized = repository.replace(/^https?:\/\//, "");
  const parts = normalized.split("/").filter(Boolean);

  if (parts[0] !== "ghcr.io" || parts.length < 3) {
    throw new Error(`GHCR_IMAGE_REPOSITORY must look like ghcr.io/owner/package, got ${repository}`);
  }

  return {
    owner: parts[1],
    packageName: parts.slice(2).join("/"),
    ownerKind: undefined,
  };
}

async function listPackageVersions(packageInfo) {
  for (const ownerKind of ["orgs", "users"]) {
    const scopedPackage = { ...packageInfo, ownerKind };
    const response = await githubRequest(scopedPackage);

    if (response.status === 404) {
      continue;
    }

    if (!response.ok) {
      throw new Error(`Failed to list GHCR versions: ${response.status} ${await response.text()}`);
    }

    packageInfo.ownerKind = ownerKind;
    return paginate(scopedPackage, await response.json(), response.headers.get("link"));
  }

  throw new Error(
    `Could not find GHCR package ${packageInfo.owner}/${packageInfo.packageName} as an org or user package`,
  );
}

async function paginate(packageInfo, firstPage, firstLinkHeader) {
  const pages = [...firstPage];
  let nextUrl = getNextUrl(firstLinkHeader);

  while (nextUrl) {
    const response = await githubRequest(packageInfo, nextUrl);

    if (!response.ok) {
      throw new Error(`Failed to paginate GHCR versions: ${response.status} ${await response.text()}`);
    }

    pages.push(...(await response.json()));
    nextUrl = getNextUrl(response.headers.get("link"));
  }

  return pages;
}

async function deletePackageVersion(packageInfo, versionId) {
  const response = await githubRequest(packageInfo, String(versionId), "DELETE");

  if (!response.ok && response.status !== 404) {
    throw new Error(`Failed to delete GHCR version ${versionId}: ${response.status} ${await response.text()}`);
  }
}

async function githubRequest(packageInfo, suffixOrUrl, method = "GET", includePagination = true) {
  const encodedPackageName = encodeURIComponent(packageInfo.packageName);
  const basePath = `${packageInfo.ownerKind}/${packageInfo.owner}/packages/container/${encodedPackageName}/versions`;
  const url = suffixOrUrl?.startsWith?.("https://")
    ? suffixOrUrl
    : `https://api.github.com/${basePath}${suffixOrUrl ? `/${suffixOrUrl}` : includePagination ? "?per_page=100" : ""}`;

  return fetch(url, {
    method,
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token}`,
      "X-GitHub-Api-Version": "2022-11-28",
    },
  });
}

function getNextUrl(linkHeader) {
  if (!linkHeader) {
    return undefined;
  }

  return linkHeader
    .split(",")
    .map((link) => link.trim())
    .find((link) => link.endsWith('rel="next"'))
    ?.match(/<([^>]+)>/)?.[1];
}

function formatTags(tags) {
  return tags.length > 0 ? tags.join(", ") : "untagged";
}
