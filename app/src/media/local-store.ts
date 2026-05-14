import { createReadStream } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, join, normalize } from "node:path";
import { Readable } from "node:stream";

import type {
  MediaObjectLocation,
  MediaReadResult,
  MediaUpload,
  MediaUploadResult,
  PrivateMediaStore,
} from "./types";

const toBuffer = async (body: MediaUpload["body"]): Promise<Buffer> => {
  if (Buffer.isBuffer(body)) {
    return body;
  }
  if (body instanceof Uint8Array) {
    return Buffer.from(body);
  }

  const chunks: Buffer[] = [];
  for await (const chunk of body) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  return Buffer.concat(chunks);
};

export class LocalMediaStore implements PrivateMediaStore {
  constructor(
    private readonly root: string,
    private readonly namespace: string,
    private readonly bucketName: string,
  ) {}

  async upload(input: MediaUpload): Promise<MediaUploadResult> {
    const objectPath = this.getObjectPath(input.objectKey);
    const body = await toBuffer(input.body);

    await mkdir(dirname(objectPath), { recursive: true });
    await writeFile(objectPath, body);

    return {
      storageNamespace: this.namespace,
      bucketName: this.bucketName,
      objectKey: input.objectKey,
      contentType: input.contentType,
      byteSize: input.byteSize ?? body.byteLength,
      checksum: null,
      etag: null,
    };
  }

  async read(location: MediaObjectLocation): Promise<MediaReadResult> {
    this.assertSameBucket(location);

    return {
      ...location,
      contentType: "application/octet-stream",
      byteSize: null,
      checksum: null,
      etag: null,
      body: createReadStream(this.getObjectPath(location.objectKey)),
    };
  }

  async assertReady(): Promise<void> {
    await mkdir(this.root, { recursive: true });
  }

  private assertSameBucket(location: MediaObjectLocation): void {
    if (location.storageNamespace !== this.namespace || location.bucketName !== this.bucketName) {
      throw new Error("Media object location does not belong to the configured local media store.");
    }
  }

  private getObjectPath(objectKey: string): string {
    const normalized = normalize(objectKey);
    if (normalized.startsWith("..") || normalized.startsWith("/") || normalized.includes("/../")) {
      throw new Error("Object key must stay within the local media root.");
    }
    return join(this.root, normalized);
  }
}
