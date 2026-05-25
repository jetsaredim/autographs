import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import { join, relative } from "node:path";
import test from "node:test";
import nextConfig from "../../next.config";

const appRoot = new URL("../../app", import.meta.url);
const publicSurfaceRoot = appRoot.pathname;
const repoRoot = new URL("../../..", import.meta.url).pathname;
const publicDenyList = [
  "storageNamespace",
  "bucketName",
  "objectKey",
  "OCI_",
  "MEDIA_BUCKET",
  "https://objectstorage",
];
const adminWorkflowDenyList = [
  "POST",
  "PUT",
  "PATCH",
  "DELETE",
  "Authorization",
  "Bearer",
  "upload",
  "publish",
  "password",
  "login",
];

test("public surface source does not expose private storage identifiers", async () => {
  const files = await listSourceFiles(publicSurfaceRoot);
  const publicFiles = files.filter((file) => !relative(publicSurfaceRoot, file).startsWith("api/operator/"));

  for (const file of publicFiles) {
    const source = await readFile(file, "utf8");
    for (const denied of publicDenyList) {
      assert.equal(source.includes(denied), false, `${relative(publicSurfaceRoot, file)} contains ${denied}`);
    }
  }
});

test("public ui does not expose standalone image route anchors", async () => {
  const files = await listSourceFiles(publicSurfaceRoot);
  const publicUiFiles = files.filter((file) => !relative(publicSurfaceRoot, file).startsWith("api/"));

  for (const file of publicUiFiles) {
    const source = await readFile(file, "utf8");
    assert.equal(
      /href=["'`{][^"'`}]*(\/api\/catalog\/[^"'`}]*)/u.test(source),
      false,
      `${relative(publicSurfaceRoot, file)} exposes an image route href`,
    );
  }
});

test("public surface keeps Surprise Me off the collection page", async () => {
  const collectionPage = await readFile(join(publicSurfaceRoot, "collection/page.tsx"), "utf8");

  assert.equal(collectionPage.includes("Surprise Me"), false);
});

test("admin placeholder files do not implement privileged workflows", async () => {
  const files = [
    join(publicSurfaceRoot, "admin/page.tsx"),
    join(publicSurfaceRoot, "components/AdminUnlock.tsx"),
  ];

  for (const file of files) {
    const source = await readFile(file, "utf8");
    for (const denied of adminWorkflowDenyList) {
      assert.equal(source.includes(denied), false, `${relative(publicSurfaceRoot, file)} contains ${denied}`);
    }
  }
});

test("next config applies baseline public security headers", async () => {
  assert.equal(typeof nextConfig.headers, "function");

  const headerRules = await nextConfig.headers();
  const allHeaders = new Map(headerRules.flatMap((rule) => rule.headers.map((header) => [header.key, header.value])));

  assert.equal(allHeaders.get("X-Content-Type-Options"), "nosniff");
  assert.equal(allHeaders.get("Referrer-Policy"), "strict-origin-when-cross-origin");
  assert.match(allHeaders.get("Permissions-Policy") ?? "", /camera=\(\), microphone=\(\), geolocation=\(\)/u);
  assert.match(allHeaders.get("Content-Security-Policy") ?? "", /frame-ancestors 'none'/u);
});

test("public caddy edge blocks temporary operator routes", async () => {
  const caddyfile = await readFile(
    join(repoRoot, "deploy/ansible/roles/autographs_deploy/files/Caddyfile"),
    "utf8",
  );

  assert.match(caddyfile, /@operator\s+path\s+\/api\/operator\s+\/api\/operator\/\*/u);
  assert.match(caddyfile, /respond\s+@operator\s+404/u);
});

const listSourceFiles = async (directory: string): Promise<string[]> => {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry) => {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        return listSourceFiles(path);
      }
      return /\.(ts|tsx)$/u.test(entry.name) ? [path] : [];
    }),
  );

  return files.flat();
};
