import { Readable } from "node:stream";

import { common, objectstorage, SimpleAuthenticationDetailsProvider } from "oci-sdk";

import type { MediaStorageConfig } from "./config";
import type {
  MediaObjectLocation,
  MediaReadResult,
  MediaUpload,
  MediaUploadResult,
  PrivateMediaStore,
} from "./types";

export class OciObjectStorageMediaStore implements PrivateMediaStore {
  private readonly client: objectstorage.ObjectStorageClient;

  constructor(private readonly config: MediaStorageConfig) {
    if (!config.oci) {
      throw new Error("OCI auth configuration is required for the OCI media store.");
    }

    const provider = new SimpleAuthenticationDetailsProvider(
      config.oci.tenancyOcid,
      config.oci.userOcid,
      config.oci.fingerprint,
      config.oci.privateKey,
      null,
      common.Region.fromRegionId(config.region),
    );

    this.client = new objectstorage.ObjectStorageClient({
      authenticationDetailsProvider: provider,
    });
    this.client.regionId = config.region;
  }

  async upload(input: MediaUpload): Promise<MediaUploadResult> {
    const result = await this.client.putObject({
      namespaceName: this.config.namespace,
      bucketName: this.config.bucketName,
      objectName: input.objectKey,
      putObjectBody: input.body,
      contentLength: input.byteSize ?? undefined,
      contentType: input.contentType,
      opcMeta: input.metadata,
    });

    return {
      storageNamespace: this.config.namespace,
      bucketName: this.config.bucketName,
      objectKey: input.objectKey,
      contentType: input.contentType,
      byteSize: input.byteSize ?? null,
      checksum: result.opcContentMd5 ?? null,
      etag: result.eTag ?? null,
    };
  }

  async read(location: MediaObjectLocation): Promise<MediaReadResult> {
    this.assertSameBucket(location);
    const result = await this.client.getObject({
      namespaceName: location.storageNamespace,
      bucketName: location.bucketName,
      objectName: location.objectKey,
    });

    return {
      ...location,
      contentType: result.contentType ?? "application/octet-stream",
      byteSize: result.contentLength ?? null,
      checksum: result.contentMd5 ?? null,
      etag: result.eTag ?? null,
      body: result.value,
    };
  }

  async delete(location: MediaObjectLocation): Promise<void> {
    this.assertSameBucket(location);
    await this.client.deleteObject({
      namespaceName: location.storageNamespace,
      bucketName: location.bucketName,
      objectName: location.objectKey,
    });
  }

  async assertReady(): Promise<void> {
    await this.client.headBucket({
      namespaceName: this.config.namespace,
      bucketName: this.config.bucketName,
    });
  }

  private assertSameBucket(location: MediaObjectLocation): void {
    if (
      location.storageNamespace !== this.config.namespace ||
      location.bucketName !== this.config.bucketName
    ) {
      throw new Error("Media object location does not belong to the configured OCI media store.");
    }
  }
}
