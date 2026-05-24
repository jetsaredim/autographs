import assert from "node:assert/strict";
import test from "node:test";

import { DefaultCatalogService } from "./service";
import type {
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

class FakeRepository implements CatalogRepository {
  lastUpdate: AutographItemUpdate | null = null;

  constructor(private readonly existing: AutographItem) {}

  async create(_input: AutographItemInput): Promise<AutographItem> {
    return this.existing;
  }

  async update(_id: string, input: AutographItemUpdate): Promise<AutographItem> {
    this.lastUpdate = input;
    return { ...this.existing, ...input, images: this.existing.images };
  }

  async getById(): Promise<AutographItem | null> {
    return this.existing;
  }

  async list(_options?: AutographListOptions): Promise<AutographItem[]> {
    return [this.existing];
  }
}

class FakeMediaStore implements PrivateMediaStore {
  async upload(_input: MediaUpload): Promise<MediaUploadResult> {
    throw new Error("upload is not used in this test.");
  }

  async read(_input: MediaObjectLocation): Promise<MediaReadResult> {
    throw new Error("read is not used in this test.");
  }

  async delete(_input: MediaObjectLocation): Promise<void> {
    throw new Error("delete is not used in this test.");
  }

  async assertReady(): Promise<void> {
    return undefined;
  }
}
