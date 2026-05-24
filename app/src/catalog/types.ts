export type PublicationStatus = "draft" | "published" | "archived";

export type AutographImageInput = {
  storageNamespace: string;
  bucketName: string;
  objectKey: string;
  contentType: string;
  byteSize?: number | null;
  checksum?: string | null;
  etag?: string | null;
  isPrimary: boolean;
  sortOrder: number;
  altText?: string | null;
};

export type AutographImage = AutographImageInput & {
  id: string;
  itemId: string;
  createdAt: Date;
  updatedAt: Date;
};

export type AutographItemInput = {
  title: string;
  signer: string;
  description?: string | null;
  category: string;
  tags: string[];
  objectReference?: string | null;
  eventName?: string | null;
  eventLocation?: string | null;
  source?: string | null;
  inscription?: string | null;
  certificationCompany?: string | null;
  certificationId?: string | null;
  estimatedYear?: number | null;
  publicationStatus: PublicationStatus;
  images?: AutographImageInput[];
};

export type AutographItem = Omit<AutographItemInput, "images"> & {
  id: string;
  createdAt: Date;
  updatedAt: Date;
  images: AutographImage[];
};

export type AutographItemUpdate = Partial<AutographItemInput>;

export type AutographListOptions = {
  includeUnpublished?: boolean;
  signer?: string;
  category?: string;
  tag?: string;
};

export type CatalogRepository = {
  create(input: AutographItemInput): Promise<AutographItem>;
  update(id: string, input: AutographItemUpdate): Promise<AutographItem>;
  delete(id: string): Promise<void>;
  getById(id: string, options?: { includeUnpublished?: boolean }): Promise<AutographItem | null>;
  list(options?: AutographListOptions): Promise<AutographItem[]>;
};
