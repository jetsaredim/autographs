import assert from "node:assert/strict";
import test from "node:test";

import { DefaultCatalogService } from "./service";
import type {
  AutographImage,
  AutographItem,
  AutographItemInput,
  AutographItemUpdate,
  AutographListOptions,
  CatalogRepository,
} from "./types";
import type {
  MediaObjectLocation,
  MediaReadResult,
  MediaUpload,
  MediaUploadResult,
  PrivateMediaStore,
} from "../media";

const now = new Date("2026-01-01T00:00:00.000Z");

const item = (overrides: Partial<AutographItem> = {}): AutographItem => ({
  id: overrides.id ?? "item-1",
  title: overrides.title ?? "Signed Example Card",
  signer: overrides.signer ?? "Example Signer",
  description: overrides.description ?? "Existing description",
  category: overrides.category ?? "Star Wars CCG",
  tags: overrides.tags ?? ["star wars", "trading cards"],
  objectReference: overrides.objectReference ?? "Premiere",
  eventName: overrides.eventName ?? "Example Signing",
  eventLocation: overrides.eventLocation ?? "Orlando, FL",
  source: overrides.source ?? "Convention signing",
  inscription: overrides.inscription ?? "To Jared",
  certificationCompany: overrides.certificationCompany ?? "PSA",
  certificationId: overrides.certificationId ?? "ABC123",
  estimatedYear: overrides.estimatedYear ?? 2026,
  publicationStatus: overrides.publicationStatus ?? "published",
  images: overrides.images ?? [],
  createdAt: overrides.createdAt ?? now,
  updatedAt: overrides.updatedAt ?? now,
});

const image = (overrides: Partial<AutographImage> = {}): AutographImage => ({
  id: overrides.id ?? "image-1",
  itemId: overrides.itemId ?? "item-1",
  storageNamespace: overrides.storageNamespace ?? "namespace",
  bucketName: overrides.bucketName ?? "bucket",
  objectKey: overrides.objectKey ?? "autographs/item-1/image-1.svg",
  contentType: overrides.contentType ?? "image/svg+xml",
  byteSize: overrides.byteSize ?? 10,
  checksum: overrides.checksum ?? null,
  etag: overrides.etag ?? null,
  isPrimary: overrides.isPrimary ?? true,
  sortOrder: overrides.sortOrder ?? 0,
  altText: overrides.altText ?? "Smoke image",
  createdAt: overrides.createdAt ?? now,
  updatedAt: overrides.updatedAt ?? now,
});

test("catalog service update preserves omitted metadata and allows explicit clears", async () => {
  const existing = item();
  const repository = new FakeRepository(existing);
  const service = new DefaultCatalogService(repository, new FakeMediaStore());

  await service.update(existing.id, {
    description: "Updated description",
    certificationId: null,
  });

  assert.deepEqual(repository.lastUpdate, {
    title: existing.title,
    signer: existing.signer,
    description: "Updated description",
    category: existing.category,
    tags: existing.tags,
    objectReference: existing.objectReference,
    eventName: existing.eventName,
    eventLocation: existing.eventLocation,
    source: existing.source,
    inscription: existing.inscription,
    certificationCompany: existing.certificationCompany,
    certificationId: null,
    estimatedYear: existing.estimatedYear,
    publicationStatus: existing.publicationStatus,
    images: undefined,
  });
});

test("catalog service removes uploaded media when image attach persistence fails", async () => {
  const existing = item();
  const repository = new FakeRepository(existing);
  repository.updateError = new Error("repository update failed");
  const mediaStore = new FakeMediaStore();
  const service = new DefaultCatalogService(repository, mediaStore);

  await assert.rejects(
    service.attachImages(existing.id, [
      {
        filename: "front.svg",
        contentType: "image/svg+xml",
        body: Buffer.from("<svg />"),
        byteSize: 7,
        isPrimary: true,
      },
    ]),
    /repository update failed/,
  );

  assert.equal(mediaStore.uploaded.length, 1);
  assert.deepEqual(mediaStore.deleted, [
    {
      storageNamespace: "namespace",
      bucketName: "bucket",
      objectKey: mediaStore.uploaded[0]?.objectKey,
    },
  ]);
});

test("catalog service removes created metadata when image attachment fails during create", async () => {
  const existing = item();
  const repository = new FakeRepository(existing);
  repository.updateError = new Error("repository update failed");
  const mediaStore = new FakeMediaStore();
  const service = new DefaultCatalogService(repository, mediaStore);

  await assert.rejects(
    service.create({
      title: existing.title,
      signer: existing.signer,
      description: existing.description,
      category: existing.category,
      tags: existing.tags,
      publicationStatus: existing.publicationStatus,
      imageUploads: [
        {
          filename: "front.svg",
          contentType: "image/svg+xml",
          body: Buffer.from("<svg />"),
          byteSize: 7,
          isPrimary: true,
        },
      ],
    }),
    /repository update failed/,
  );

  assert.deepEqual(repository.deletedIds, [existing.id]);
  assert.equal(mediaStore.uploaded.length, 1);
  assert.deepEqual(mediaStore.deleted, [
    {
      storageNamespace: "namespace",
      bucketName: "bucket",
      objectKey: mediaStore.uploaded[0]?.objectKey,
    },
  ]);
});

test("catalog service removes earlier uploads when a later image upload fails", async () => {
  const existing = item();
  const mediaStore = new FakeMediaStore({ failUploadAt: 1 });
  const service = new DefaultCatalogService(new FakeRepository(existing), mediaStore);

  await assert.rejects(
    service.attachImages(existing.id, [
      {
        filename: "front.svg",
        contentType: "image/svg+xml",
        body: Buffer.from("<svg />"),
        byteSize: 7,
        isPrimary: true,
      },
      {
        filename: "back.svg",
        contentType: "image/svg+xml",
        body: Buffer.from("<svg />"),
        byteSize: 7,
      },
    ]),
    /upload failed/,
  );

  assert.equal(mediaStore.uploaded.length, 1);
  assert.deepEqual(mediaStore.deleted, [
    {
      storageNamespace: "namespace",
      bucketName: "bucket",
      objectKey: mediaStore.uploaded[0]?.objectKey,
    },
  ]);
});

test("catalog service deletes media before deleting item metadata", async () => {
  const events: string[] = [];
  const existing = item({ images: [image()] });
  const repository = new FakeRepository(existing, events);
  const mediaStore = new FakeMediaStore({ events });
  const service = new DefaultCatalogService(repository, mediaStore);

  await service.delete(existing.id);

  assert.deepEqual(events, ["media.delete:autographs/item-1/image-1.svg", "repository.delete"]);
});

test("catalog service deletes media before removing one image reference", async () => {
  const events: string[] = [];
  const existing = item({
    images: [
      image({ id: "image-1", objectKey: "autographs/item-1/image-1.svg", isPrimary: true }),
      image({
        id: "image-2",
        objectKey: "autographs/item-1/image-2.svg",
        isPrimary: false,
        sortOrder: 1,
      }),
    ],
  });
  const repository = new FakeRepository(existing, events);
  const mediaStore = new FakeMediaStore({ events });
  const service = new DefaultCatalogService(repository, mediaStore);

  await service.deleteImage(existing.id, "image-1");

  assert.deepEqual(events, ["media.delete:autographs/item-1/image-1.svg", "repository.update"]);
});

class FakeRepository implements CatalogRepository {
  lastUpdate: AutographItemUpdate | null = null;
  updateError: Error | null = null;
  readonly deletedIds: string[] = [];

  constructor(
    private readonly existing: AutographItem,
    private readonly events: string[] = [],
  ) {}

  async create(_input: AutographItemInput): Promise<AutographItem> {
    return this.existing;
  }

  async update(_id: string, input: AutographItemUpdate): Promise<AutographItem> {
    if (this.updateError) {
      throw this.updateError;
    }
    this.lastUpdate = input;
    this.events.push("repository.update");
    return { ...this.existing, ...input, images: this.existing.images };
  }

  async delete(id: string): Promise<void> {
    this.deletedIds.push(id);
    this.events.push("repository.delete");
    return undefined;
  }

  async getById(): Promise<AutographItem | null> {
    return this.existing;
  }

  async list(_options?: AutographListOptions): Promise<AutographItem[]> {
    return [this.existing];
  }
}

class FakeMediaStore implements PrivateMediaStore {
  readonly uploaded: MediaUploadResult[] = [];
  readonly deleted: MediaObjectLocation[] = [];
  private uploadCount = 0;

  constructor(
    private readonly options: {
      events?: string[];
      failUploadAt?: number;
    } = {},
  ) {}

  async upload(input: MediaUpload): Promise<MediaUploadResult> {
    if (this.uploadCount === this.options.failUploadAt) {
      throw new Error("upload failed");
    }
    this.uploadCount += 1;

    const result = {
      storageNamespace: "namespace",
      bucketName: "bucket",
      objectKey: input.objectKey,
      contentType: input.contentType,
      byteSize: input.byteSize,
      checksum: null,
      etag: null,
    };
    this.uploaded.push(result);
    return result;
  }

  async read(_input: MediaObjectLocation): Promise<MediaReadResult> {
    throw new Error("read is not used in this test.");
  }

  async delete(input: MediaObjectLocation): Promise<void> {
    this.deleted.push(input);
    this.options.events?.push(`media.delete:${input.objectKey}`);
  }

  async assertReady(): Promise<void> {
    return undefined;
  }
}
