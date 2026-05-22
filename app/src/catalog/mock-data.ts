import type { AutographImage, AutographItem, AutographListOptions } from "./types";

const mockDate = new Date("2026-01-01T00:00:00.000Z");

const mockImage = (
  itemId: string,
  id: string,
  altText: string,
  sortOrder: number,
  isPrimary = false,
): AutographImage => ({
  id,
  itemId,
  storageNamespace: "local-preview",
  bucketName: "local-preview",
  objectKey: `local-preview/${itemId}/${id}.svg`,
  contentType: "image/svg+xml",
  byteSize: null,
  checksum: null,
  etag: null,
  isPrimary,
  sortOrder,
  altText,
  createdAt: mockDate,
  updatedAt: mockDate,
});

export const mockCatalogItems: AutographItem[] = [
  {
    id: "mock-autograph",
    title: "Mock Autograph Layout Review",
    signer: "Sample Signer",
    description:
      "A local-only placeholder record for reviewing spacing, metadata, breadcrumbs, and detail formatting without a configured database.",
    category: "Star Wars CCG",
    tags: ["star wars", "trading cards", "layout review"],
    objectReference: "Premiere sample card",
    eventName: "Local Preview Signing",
    eventLocation: "Design Review",
    source: "Mock local catalog data",
    inscription: "To Jared, thanks for the preview.",
    certificationCompany: "Preview Authenticator",
    certificationId: "LOCAL-001",
    estimatedYear: 1997,
    publicationStatus: "published",
    images: [
      mockImage("mock-autograph", "mock-front", "Mock autograph front preview", 0, true),
      mockImage("mock-autograph", "mock-back", "Mock autograph reverse preview", 1),
    ],
    createdAt: mockDate,
    updatedAt: mockDate,
  },
];

export const listMockCatalogItems = (options: AutographListOptions = {}): AutographItem[] =>
  mockCatalogItems.filter((item) => {
    if (options.signer && !item.signer.toLowerCase().includes(options.signer.toLowerCase())) {
      return false;
    }
    if (options.category && item.category !== options.category) {
      return false;
    }
    if (options.tag && !item.tags.includes(options.tag)) {
      return false;
    }
    return true;
  });

export const getMockCatalogItem = (id: string): AutographItem | null =>
  mockCatalogItems.find((item) => item.id === id) ?? null;

export const getMockImageSvg = (id: string, imageId: string): string | null => {
  const item = getMockCatalogItem(id);
  const image = item?.images.find((candidate) => candidate.id === imageId);
  if (!item || !image) {
    return null;
  }

  const isBack = imageId.includes("back");
  const background = isBack ? "#e9e2d6" : "#f4f1ea";
  const accent = isBack ? "#8d887f" : "#6f675c";

  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 480 600" role="img" aria-label="${image.altText}">
    <rect width="480" height="600" fill="${background}"/>
    <rect x="56" y="56" width="368" height="488" rx="14" fill="#fbfaf7" stroke="#dfd8cc" stroke-width="3"/>
    <path d="M112 360c58-62 102-64 132-8 24 45 62 39 124-18" fill="none" stroke="${accent}" stroke-width="12" stroke-linecap="round"/>
    <text x="240" y="176" text-anchor="middle" fill="#2f2f2d" font-family="IBM Plex Sans, Segoe UI, sans-serif" font-size="30" font-weight="600">${item.signer}</text>
    <text x="240" y="470" text-anchor="middle" fill="#8d887f" font-family="IBM Plex Sans, Segoe UI, sans-serif" font-size="16">${item.category}</text>
  </svg>`;
};
