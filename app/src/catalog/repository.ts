import { randomUUID } from "node:crypto";

import type { DatabaseExecutor, SqlBinds } from "../db/oracle";
import type {
  AutographImage,
  AutographImageInput,
  AutographItem,
  AutographItemInput,
  AutographItemUpdate,
  AutographListOptions,
  CatalogRepository,
  PublicationStatus,
} from "./types";

type ItemRow = {
  ID: string;
  TITLE: string;
  SIGNER: string;
  DESCRIPTION: string | null;
  CATEGORY: string;
  OBJECT_REFERENCE: string | null;
  EVENT_NAME: string | null;
  EVENT_LOCATION: string | null;
  SOURCE: string | null;
  INSCRIPTION: string | null;
  CERTIFICATION_COMPANY: string | null;
  CERTIFICATION_ID: string | null;
  ESTIMATED_YEAR: number | null;
  PUBLICATION_STATUS: PublicationStatus;
  CREATED_AT: Date;
  UPDATED_AT: Date;
};

type TagRow = {
  ITEM_ID: string;
  TAG: string;
};

type ImageRow = {
  ID: string;
  ITEM_ID: string;
  STORAGE_NAMESPACE: string;
  BUCKET_NAME: string;
  OBJECT_KEY: string;
  CONTENT_TYPE: string;
  BYTE_SIZE: number | null;
  CHECKSUM: string | null;
  ETAG: string | null;
  IS_PRIMARY: "Y" | "N";
  SORT_ORDER: number;
  ALT_TEXT: string | null;
  CREATED_AT: Date;
  UPDATED_AT: Date;
};

const itemSelect = `
  select id, title, signer, description, category, object_reference,
         event_name, event_location, source, inscription,
         certification_company, certification_id, estimated_year,
         publication_status, created_at, updated_at
  from autograph_items
`;

const toItem = (
  row: ItemRow,
  tags: string[],
  images: AutographImage[],
): AutographItem => ({
  id: row.ID,
  title: row.TITLE,
  signer: row.SIGNER,
  description: row.DESCRIPTION,
  category: row.CATEGORY,
  tags,
  objectReference: row.OBJECT_REFERENCE,
  eventName: row.EVENT_NAME,
  eventLocation: row.EVENT_LOCATION,
  source: row.SOURCE,
  inscription: row.INSCRIPTION,
  certificationCompany: row.CERTIFICATION_COMPANY,
  certificationId: row.CERTIFICATION_ID,
  estimatedYear: row.ESTIMATED_YEAR,
  publicationStatus: row.PUBLICATION_STATUS,
  createdAt: row.CREATED_AT,
  updatedAt: row.UPDATED_AT,
  images,
});

const toImage = (row: ImageRow): AutographImage => ({
  id: row.ID,
  itemId: row.ITEM_ID,
  storageNamespace: row.STORAGE_NAMESPACE,
  bucketName: row.BUCKET_NAME,
  objectKey: row.OBJECT_KEY,
  contentType: row.CONTENT_TYPE,
  byteSize: row.BYTE_SIZE,
  checksum: row.CHECKSUM,
  etag: row.ETAG,
  isPrimary: row.IS_PRIMARY === "Y",
  sortOrder: row.SORT_ORDER,
  altText: row.ALT_TEXT,
  createdAt: row.CREATED_AT,
  updatedAt: row.UPDATED_AT,
});

const requireOnePrimary = (images: AutographImageInput[] = []): void => {
  const primaryCount = images.filter((image) => image.isPrimary).length;
  if (images.length > 0 && primaryCount !== 1) {
    throw new Error("Exactly one image must be marked primary when images are provided.");
  }
};

export class OracleCatalogRepository implements CatalogRepository {
  constructor(private readonly executor: DatabaseExecutor) {}

  async create(input: AutographItemInput): Promise<AutographItem> {
    requireOnePrimary(input.images);
    const id = randomUUID();

    await this.executor.transaction(async (tx) => {
      await tx.execute(
        `insert into autograph_items (
          id, title, signer, description, category, object_reference,
          event_name, event_location, source, inscription,
          certification_company, certification_id, estimated_year,
          publication_status
        ) values (
          :id, :title, :signer, :description, :category, :objectReference,
          :eventName, :eventLocation, :source, :inscription,
          :certificationCompany, :certificationId, :estimatedYear,
          :publicationStatus
        )`,
        toItemBinds(id, input),
      );
      await replaceTags(tx, id, input.tags);
      await replaceImages(tx, id, input.images ?? []);
    });

    const created = await this.getById(id, { includeUnpublished: true });
    if (!created) {
      throw new Error(`Created autograph item ${id} could not be read back.`);
    }
    return created;
  }

  async update(id: string, input: AutographItemUpdate): Promise<AutographItem> {
    if (input.images) {
      requireOnePrimary(input.images);
    }

    await this.executor.transaction(async (tx) => {
      await tx.execute(
        `update autograph_items
         set title = coalesce(:title, title),
             signer = coalesce(:signer, signer),
             description = :description,
             category = coalesce(:category, category),
             object_reference = :objectReference,
             event_name = :eventName,
             event_location = :eventLocation,
             source = :source,
             inscription = :inscription,
             certification_company = :certificationCompany,
             certification_id = :certificationId,
             estimated_year = :estimatedYear,
             publication_status = coalesce(:publicationStatus, publication_status),
             updated_at = current_timestamp
         where id = :id`,
        toItemBinds(id, input),
      );

      if (input.tags) {
        await replaceTags(tx, id, input.tags);
      }
      if (input.images) {
        await replaceImages(tx, id, input.images);
      }
    });

    const updated = await this.getById(id, { includeUnpublished: true });
    if (!updated) {
      throw new Error(`Autograph item ${id} was not found after update.`);
    }
    return updated;
  }

  async delete(id: string): Promise<void> {
    await this.executor.transaction(async (tx) => {
      await tx.execute("delete from autograph_item_tags where item_id = :id", { id });
      await tx.execute("delete from autograph_images where item_id = :id", { id });
      await tx.execute("delete from autograph_items where id = :id", { id });
    });
  }

  async getById(
    id: string,
    options: { includeUnpublished?: boolean } = {},
  ): Promise<AutographItem | null> {
    const publicationClause = options.includeUnpublished
      ? ""
      : "and publication_status = 'published'";
    const result = await this.executor.execute<ItemRow>(
      `${itemSelect} where id = :id ${publicationClause}`,
      { id },
    );
    const [row] = result.rows;
    if (!row) {
      return null;
    }

    const [tags, images] = await Promise.all([
      this.getTags([id]),
      this.getImages([id]),
    ]);
    return toItem(row, tags.get(id) ?? [], images.get(id) ?? []);
  }

  async list(options: AutographListOptions = {}): Promise<AutographItem[]> {
    const clauses: string[] = [];
    const binds: SqlBinds = {};

    if (!options.includeUnpublished) {
      clauses.push("publication_status = 'published'");
    }
    if (options.signer) {
      clauses.push("lower(signer) like :signer");
      binds.signer = `%${options.signer.toLowerCase()}%`;
    }
    if (options.category) {
      clauses.push("category = :category");
      binds.category = options.category;
    }
    if (options.tag) {
      clauses.push(
        "exists (select 1 from autograph_item_tags t where t.item_id = autograph_items.id and t.tag = :tag)",
      );
      binds.tag = options.tag;
    }

    const where = clauses.length > 0 ? `where ${clauses.join(" and ")}` : "";
    const result = await this.executor.execute<ItemRow>(
      `${itemSelect} ${where} order by signer, title`,
      binds,
    );
    const ids = result.rows.map((row) => row.ID);
    const [tags, images] = await Promise.all([this.getTags(ids), this.getImages(ids)]);

    return result.rows.map((row) => toItem(row, tags.get(row.ID) ?? [], images.get(row.ID) ?? []));
  }

  private async getTags(ids: string[]): Promise<Map<string, string[]>> {
    if (ids.length === 0) {
      return new Map();
    }
    const result = await this.executor.execute<TagRow>(
      `select item_id, tag from autograph_item_tags
       where item_id in (${bindNames(ids, "tagItem")})
       order by tag`,
      bindArray(ids, "tagItem"),
    );
    const tags = new Map<string, string[]>();
    for (const row of result.rows) {
      tags.set(row.ITEM_ID, [...(tags.get(row.ITEM_ID) ?? []), row.TAG]);
    }
    return tags;
  }

  private async getImages(ids: string[]): Promise<Map<string, AutographImage[]>> {
    if (ids.length === 0) {
      return new Map();
    }
    const result = await this.executor.execute<ImageRow>(
      `select id, item_id, storage_namespace, bucket_name, object_key,
              content_type, byte_size, checksum, etag, is_primary, sort_order,
              alt_text, created_at, updated_at
       from autograph_images
       where item_id in (${bindNames(ids, "imageItem")})
       order by item_id, sort_order`,
      bindArray(ids, "imageItem"),
    );
    const images = new Map<string, AutographImage[]>();
    for (const row of result.rows) {
      images.set(row.ITEM_ID, [...(images.get(row.ITEM_ID) ?? []), toImage(row)]);
    }
    return images;
  }
}

const toItemBinds = (id: string, input: AutographItemUpdate): SqlBinds => ({
  id,
  title: input.title,
  signer: input.signer,
  description: input.description ?? null,
  category: input.category,
  objectReference: input.objectReference ?? null,
  eventName: input.eventName ?? null,
  eventLocation: input.eventLocation ?? null,
  source: input.source ?? null,
  inscription: input.inscription ?? null,
  certificationCompany: input.certificationCompany ?? null,
  certificationId: input.certificationId ?? null,
  estimatedYear: input.estimatedYear ?? null,
  publicationStatus: input.publicationStatus,
});

const replaceTags = async (
  executor: DatabaseExecutor,
  itemId: string,
  tags: string[],
): Promise<void> => {
  await executor.execute("delete from autograph_item_tags where item_id = :itemId", {
    itemId,
  });
  await executor.executeMany(
    `insert into autograph_item_tags (item_id, tag) values (:itemId, :tag)`,
    [...new Set(tags)].map((tag) => ({ itemId, tag })),
  );
};

const replaceImages = async (
  executor: DatabaseExecutor,
  itemId: string,
  images: AutographImageInput[],
): Promise<void> => {
  await executor.execute("delete from autograph_images where item_id = :itemId", {
    itemId,
  });
  await executor.executeMany(
    `insert into autograph_images (
      id, item_id, storage_namespace, bucket_name, object_key, content_type,
      byte_size, checksum, etag, is_primary, sort_order, alt_text
    ) values (
      :id, :itemId, :storageNamespace, :bucketName, :objectKey, :contentType,
      :byteSize, :checksum, :etag, :isPrimary, :sortOrder, :altText
    )`,
    images.map((image) => ({
      id: randomUUID(),
      itemId,
      storageNamespace: image.storageNamespace,
      bucketName: image.bucketName,
      objectKey: image.objectKey,
      contentType: image.contentType,
      byteSize: image.byteSize ?? null,
      checksum: image.checksum ?? null,
      etag: image.etag ?? null,
      isPrimary: image.isPrimary ? "Y" : "N",
      sortOrder: image.sortOrder,
      altText: image.altText ?? null,
    })),
  );
};

const bindNames = (values: string[], prefix: string): string =>
  values.map((_, index) => `:${prefix}${index}`).join(", ");

const bindArray = (values: string[], prefix: string): SqlBinds =>
  Object.fromEntries(values.map((value, index) => [`${prefix}${index}`, value]));
