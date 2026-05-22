import assert from "node:assert/strict";
import test from "node:test";

import {
  buildPublicImageSrc,
  derivePublicFacets,
  toPublicGalleryItem,
  toPublicItemDetail,
} from "./public-view-models";
import type { AutographImage, AutographItem } from "./types";

const image = (overrides: Partial<AutographImage>): AutographImage => ({
  id: overrides.id ?? "image-1",
  itemId: overrides.itemId ?? "item-1",
  storageNamespace: overrides.storageNamespace ?? "private-namespace",
  bucketName: overrides.bucketName ?? "private-bucket",
  objectKey: overrides.objectKey ?? "private/object/key.jpg",
  contentType: overrides.contentType ?? "image/jpeg",
  byteSize: overrides.byteSize ?? 1234,
  checksum: overrides.checksum ?? "secret-checksum",
  etag: overrides.etag ?? "secret-etag",
  isPrimary: overrides.isPrimary ?? false,
  sortOrder: overrides.sortOrder ?? 0,
  altText: overrides.altText ?? null,
  createdAt: overrides.createdAt ?? new Date("2026-01-01T00:00:00Z"),
  updatedAt: overrides.updatedAt ?? new Date("2026-01-01T00:00:00Z"),
});

const nullable = <T>(value: T | null | undefined, fallback: T): T | null =>
  value === undefined ? fallback : value;

const item = (overrides: Partial<AutographItem> = {}): AutographItem => ({
  id: overrides.id ?? "item-1",
  title: overrides.title ?? "Signed Jedi Card",
  signer: overrides.signer ?? "Mark Hamill",
  description: nullable(overrides.description, "A signed collectible from the archive."),
  category: overrides.category ?? "Star Wars CCG",
  tags: overrides.tags ?? ["jedi", "featured", "force", "private-note"],
  objectReference: nullable(overrides.objectReference, "Premiere"),
  eventName: nullable(overrides.eventName, "Celebration"),
  eventLocation: nullable(overrides.eventLocation, "Chicago"),
  source: nullable(overrides.source, "Convention signing"),
  inscription: nullable(overrides.inscription, "To Jared"),
  certificationCompany: nullable(overrides.certificationCompany, "PSA"),
  certificationId: nullable(overrides.certificationId, "ABC123"),
  estimatedYear: nullable(overrides.estimatedYear, 1999),
  publicationStatus: overrides.publicationStatus ?? "published",
  createdAt: overrides.createdAt ?? new Date("2026-01-01T00:00:00Z"),
  updatedAt: overrides.updatedAt ?? new Date("2026-01-02T00:00:00Z"),
  images:
    overrides.images ??
    [
      image({ id: "image-1", isPrimary: false, sortOrder: 2 }),
      image({ id: "image-2", isPrimary: true, sortOrder: 3, altText: "Signed card front" }),
      image({ id: "image-3", isPrimary: false, sortOrder: 1 }),
    ],
});

const privateFieldNames = [
  "storageNamespace",
  "bucketName",
  "objectKey",
  "checksum",
  "etag",
];

test("public view models build app-mediated image routes", () => {
  assert.equal(buildPublicImageSrc("item-1", "image-2"), "/api/catalog/item-1/images/image-2");
});

test("public view models expose gallery-safe item data", () => {
  const publicItem = toPublicGalleryItem(item());

  assert.deepEqual(Object.keys(publicItem).sort(), [
    "category",
    "description",
    "id",
    "primaryImage",
    "signer",
    "tags",
    "title",
  ]);
  assert.equal(publicItem.primaryImage?.src, "/api/catalog/item-1/images/image-2");
  assert.equal(publicItem.primaryImage?.altText, "Signed card front");
  assert.deepEqual(publicItem.tags, ["jedi", "featured", "force"]);
});

test("public view models never include private image fields", () => {
  const rendered = JSON.stringify({
    gallery: toPublicGalleryItem(item()),
    detail: toPublicItemDetail(item()),
  });

  for (const field of privateFieldNames) {
    assert.equal(rendered.includes(field), false, `${field} leaked into public output`);
  }
});

test("public view models select primary images deterministically", () => {
  assert.equal(toPublicGalleryItem(item()).primaryImage?.id, "image-2");

  const bySortOrder = toPublicGalleryItem(
    item({
      images: [
        image({ id: "later", isPrimary: false, sortOrder: 5 }),
        image({ id: "earlier", isPrimary: false, sortOrder: 1 }),
      ],
    }),
  );
  assert.equal(bySortOrder.primaryImage?.id, "earlier");

  const byFirst = toPublicGalleryItem(
    item({
      images: [
        image({ id: "first", isPrimary: false, sortOrder: 0 }),
        image({ id: "second", isPrimary: false, sortOrder: 0 }),
      ],
    }),
  );
  assert.equal(byFirst.primaryImage?.id, "first");
});

test("public view models derive curated public facets", () => {
  const facets = derivePublicFacets([
    item({ signer: "Carrie Fisher", category: "Star Wars CCG", tags: ["princess", "featured"] }),
    item({ signer: "Mark Hamill", category: "Star Wars CCG", tags: ["jedi", "featured"] }),
    item({ signer: "Carrie Fisher", category: "Posters", tags: ["princess", "rare"] }),
  ]);

  assert.deepEqual(facets, [
    {
      id: "signer",
      label: "Signer",
      options: [
        { label: "Carrie Fisher", value: "Carrie Fisher" },
        { label: "Mark Hamill", value: "Mark Hamill" },
      ],
    },
    {
      id: "category",
      label: "Game",
      options: [
        { label: "Posters", value: "Posters" },
        { label: "Star Wars CCG", value: "Star Wars CCG" },
      ],
    },
    {
      id: "tag",
      label: "IP / Genre",
      options: [
        { label: "featured", value: "featured" },
        { label: "jedi", value: "jedi" },
        { label: "princess", value: "princess" },
        { label: "rare", value: "rare" },
      ],
    },
  ]);
});

test("public view models group detail metadata and hide empty values", () => {
  const detail = toPublicItemDetail(
    item({
      eventName: null,
      eventLocation: null,
      certificationCompany: null,
      certificationId: null,
    }),
  );

  assert.equal(detail.images.length, 3);
  assert.equal(detail.detailGroups.some((group) => group.label === "Certification"), false);
  assert.ok(detail.detailGroups.find((group) => group.label === "Essentials"));
  assert.ok(detail.detailGroups.find((group) => group.label === "Provenance"));
});
