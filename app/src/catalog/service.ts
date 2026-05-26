import { randomUUID } from "node:crypto";

import type {
  AutographImage,
  AutographImageInput,
  AutographItem,
  AutographItemInput,
  AutographListOptions,
  AutographItemUpdate,
  CatalogRepository,
} from "./types";
import type { MediaUpload, PrivateMediaStore } from "../media";
import type { MediaReadResult } from "../media/types";

export type CatalogImageUploadInput = Omit<MediaUpload, "objectKey"> & {
  filename: string;
  isPrimary?: boolean;
  sortOrder?: number;
  altText?: string | null;
};

export type CatalogCreateInput = Omit<AutographItemInput, "images"> & {
  imageUploads?: CatalogImageUploadInput[];
};

export type CatalogService = {
  create(input: CatalogCreateInput): Promise<AutographItem>;
  update(id: string, input: AutographItemUpdate): Promise<AutographItem>;
  delete(id: string): Promise<AutographItem>;
  deleteImage(id: string, imageId: string): Promise<AutographItem>;
  attachImages(id: string, images: CatalogImageUploadInput[]): Promise<AutographItem>;
  readPublishedImage(id: string, imageId: string): Promise<MediaReadResult | null>;
  getById(id: string, options?: { includeUnpublished?: boolean }): Promise<AutographItem | null>;
  list(options?: AutographListOptions): Promise<AutographItem[]>;
};

export class DefaultCatalogService implements CatalogService {
  constructor(
    private readonly repository: CatalogRepository,
    private readonly mediaStore: PrivateMediaStore,
  ) {}

  async create(input: CatalogCreateInput): Promise<AutographItem> {
    const { imageUploads = [], ...itemInput } = input;
    const item = await this.repository.create({ ...itemInput, images: [] });

    if (imageUploads.length === 0) {
      return item;
    }

    return this.attachImages(item.id, imageUploads);
  }

  async update(id: string, input: AutographItemUpdate): Promise<AutographItem> {
    const existing = await this.repository.getById(id, { includeUnpublished: true });
    if (!existing) {
      throw new Error(`Autograph item ${id} was not found.`);
    }

    return this.repository.update(id, mergeItemUpdate(existing, input));
  }

  async attachImages(id: string, images: CatalogImageUploadInput[]): Promise<AutographItem> {
    if (images.length === 0) {
      const existing = await this.repository.getById(id, { includeUnpublished: true });
      if (!existing) {
        throw new Error(`Autograph item ${id} was not found.`);
      }
      return existing;
    }

    const existing = await this.repository.getById(id, { includeUnpublished: true });
    if (!existing) {
      throw new Error(`Autograph item ${id} was not found.`);
    }

    const uploadedImages: AutographImageInput[] = [];

    try {
      for (const [index, image] of images.entries()) {
        const upload = await this.mediaStore.upload({
          objectKey: buildObjectKey(id, image.filename),
          contentType: image.contentType,
          body: image.body,
          byteSize: image.byteSize,
          metadata: image.metadata,
        });

        uploadedImages.push({
          ...upload,
          isPrimary: image.isPrimary ?? (existing.images.length === 0 && index === 0),
          sortOrder: image.sortOrder ?? existing.images.length + index,
          altText: image.altText ?? null,
        });
      }

      const normalizedImages = normalizePrimary([
        ...existing.images.map(toImageInput),
        ...uploadedImages,
      ]);

      return await this.repository.update(id, { images: normalizedImages });
    } catch (error) {
      await deleteMediaObjects(this.mediaStore, uploadedImages);
      throw error;
    }
  }

  async delete(id: string): Promise<AutographItem> {
    const existing = await this.repository.getById(id, { includeUnpublished: true });
    if (!existing) {
      throw new Error(`Autograph item ${id} was not found.`);
    }

    await Promise.all(
      existing.images.map((image) =>
        this.mediaStore.delete(toMediaObjectLocation(image)),
      ),
    );

    await this.repository.delete(id);

    return existing;
  }

  async deleteImage(id: string, imageId: string): Promise<AutographItem> {
    const existing = await this.repository.getById(id, { includeUnpublished: true });
    if (!existing) {
      throw new Error(`Autograph item ${id} was not found.`);
    }

    const imageToDelete = existing.images.find((image) => image.id === imageId);
    if (!imageToDelete) {
      throw new Error(`Autograph image ${imageId} was not found.`);
    }

    await this.mediaStore.delete(toMediaObjectLocation(imageToDelete));

    return this.repository.update(id, {
      images: normalizePrimary(
        existing.images
          .filter((image) => image.id !== imageId)
          .map(toImageInput),
      ),
    });
  }

  async getById(
    id: string,
    options?: { includeUnpublished?: boolean },
  ): Promise<AutographItem | null> {
    return this.repository.getById(id, options);
  }

  async readPublishedImage(id: string, imageId: string): Promise<MediaReadResult | null> {
    const item = await this.repository.getById(id);
    if (!item) {
      return null;
    }

    const image = item.images.find((candidate) => candidate.id === imageId);
    if (!image) {
      return null;
    }

    return this.mediaStore.read({
      storageNamespace: image.storageNamespace,
      bucketName: image.bucketName,
      objectKey: image.objectKey,
    });
  }

  async list(options?: AutographListOptions): Promise<AutographItem[]> {
    return this.repository.list(normalizeListOptions(options));
  }
}

const normalizeListOptions = (options: AutographListOptions = {}): AutographListOptions => ({
  ...options,
  signer: normalizeFilterOption(options.signer),
  category: normalizeFilterOption(options.category),
  tag: normalizeFilterOption(options.tag),
});

const normalizeFilterOption = (value: string | undefined): string | undefined =>
  value && value !== "all" ? value : undefined;

const mergeItemUpdate = (
  existing: AutographItem,
  input: AutographItemUpdate,
): AutographItemUpdate => ({
  title: input.title ?? existing.title,
  signer: input.signer ?? existing.signer,
  description: input.description === undefined ? existing.description : input.description,
  category: input.category ?? existing.category,
  tags: input.tags ?? existing.tags,
  objectReference:
    input.objectReference === undefined ? existing.objectReference : input.objectReference,
  eventName: input.eventName === undefined ? existing.eventName : input.eventName,
  eventLocation: input.eventLocation === undefined ? existing.eventLocation : input.eventLocation,
  source: input.source === undefined ? existing.source : input.source,
  inscription: input.inscription === undefined ? existing.inscription : input.inscription,
  certificationCompany:
    input.certificationCompany === undefined
      ? existing.certificationCompany
      : input.certificationCompany,
  certificationId:
    input.certificationId === undefined ? existing.certificationId : input.certificationId,
  estimatedYear: input.estimatedYear === undefined ? existing.estimatedYear : input.estimatedYear,
  publicationStatus: input.publicationStatus ?? existing.publicationStatus,
  images: input.images,
});

const toImageInput = (image: AutographImage): AutographImageInput => ({
  storageNamespace: image.storageNamespace,
  bucketName: image.bucketName,
  objectKey: image.objectKey,
  contentType: image.contentType,
  byteSize: image.byteSize,
  checksum: image.checksum,
  etag: image.etag,
  isPrimary: image.isPrimary,
  sortOrder: image.sortOrder,
  altText: image.altText,
});

const toMediaObjectLocation = (image: AutographImageInput) => ({
  storageNamespace: image.storageNamespace,
  bucketName: image.bucketName,
  objectKey: image.objectKey,
});

const deleteMediaObjects = async (
  mediaStore: PrivateMediaStore,
  images: AutographImageInput[],
): Promise<void> => {
  await Promise.all(images.map((image) => mediaStore.delete(toMediaObjectLocation(image))));
};

const normalizePrimary = (images: AutographImageInput[]): AutographImageInput[] => {
  const primaryIndex = images.findLastIndex((image) => image.isPrimary);
  if (images.length === 0 || primaryIndex === -1) {
    return images;
  }

  return images.map((image, index) => ({
    ...image,
    isPrimary: index === primaryIndex,
  }));
};

const buildObjectKey = (itemId: string, filename: string): string => {
  const safeFilename = filename
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return `autographs/${itemId}/${randomUUID()}-${safeFilename || "image"}`;
};
