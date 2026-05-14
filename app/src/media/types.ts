import type { Readable } from "node:stream";

export type MediaObjectLocation = {
  storageNamespace: string;
  bucketName: string;
  objectKey: string;
};

export type MediaUpload = {
  objectKey: string;
  contentType: string;
  body: Uint8Array | Buffer | Readable;
  byteSize?: number | null;
  metadata?: Record<string, string>;
};

export type MediaUploadResult = MediaObjectLocation & {
  contentType: string;
  byteSize?: number | null;
  checksum?: string | null;
  etag?: string | null;
};

export type MediaReadResult = MediaObjectLocation & {
  contentType: string;
  byteSize?: number | null;
  checksum?: string | null;
  etag?: string | null;
  body: ReadableStream | Readable;
};

export type PrivateMediaStore = {
  upload(input: MediaUpload): Promise<MediaUploadResult>;
  read(location: MediaObjectLocation): Promise<MediaReadResult>;
  assertReady(): Promise<void>;
};
