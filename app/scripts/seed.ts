import { sampleAutographs } from "../db/seed/sample-autographs";
import { createCatalogService } from "../src/catalog";
import type { AutographImageInput, AutographItemInput } from "../src/catalog/types";

const dryRun = process.argv.includes("--dry-run");

if (dryRun) {
  console.log(JSON.stringify({ dryRun: true, records: sampleAutographs }, null, 2));
  process.exit(0);
}

const service = createCatalogService();

function toFixtureSvg(
  item: Omit<AutographItemInput, "images">,
  image: AutographImageInput,
): string {
  const label = image.isPrimary ? "Primary image" : "Supporting image";
  return `<svg xmlns="http://www.w3.org/2000/svg" width="640" height="420" role="img" aria-label="${escapeXml(image.altText ?? label)}"><rect width="640" height="420" fill="#f4f4f5"/><rect x="32" y="32" width="576" height="356" rx="24" fill="#ffffff" stroke="#a1a1aa" stroke-width="4"/><text x="64" y="126" font-family="Arial, sans-serif" font-size="34" font-weight="700" fill="#111111">${escapeXml(item.signer)}</text><text x="64" y="180" font-family="Arial, sans-serif" font-size="24" fill="#3f3f46">${escapeXml(item.title)}</text><text x="64" y="250" font-family="Arial, sans-serif" font-size="22" fill="#52525b">${escapeXml(label)}</text><text x="64" y="302" font-family="Arial, sans-serif" font-size="18" fill="#71717a">Generated fixture - not a collection asset</text></svg>`;
}

function escapeXml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

for (const item of sampleAutographs) {
  const { images = [], ...itemInput } = item;
  const created = await service.create({
    ...itemInput,
    imageUploads: images.map((image) => ({
      filename: image.objectKey.split("/").at(-1) ?? "sample.svg",
      contentType: image.contentType,
      body: Buffer.from(toFixtureSvg(itemInput, image)),
      byteSize: Buffer.byteLength(toFixtureSvg(itemInput, image)),
      metadata: {
        sample: "true",
        signer: item.signer,
      },
      isPrimary: image.isPrimary,
      sortOrder: image.sortOrder,
      altText: image.altText,
    })),
  });
  console.log(`seeded ${created.id} ${created.title}`);
}
