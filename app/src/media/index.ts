import { getMediaStorageConfig } from "./config";
import { LocalMediaStore } from "./local-store";
import { OciObjectStorageMediaStore } from "./oci-store";
import type { PrivateMediaStore } from "./types";

export const createPrivateMediaStore = (): PrivateMediaStore => {
  const config = getMediaStorageConfig();

  if (config.provider === "local") {
    return new LocalMediaStore(config.localRoot, config.namespace, config.bucketName);
  }

  return new OciObjectStorageMediaStore(config);
};

export type {
  MediaObjectLocation,
  MediaReadResult,
  MediaUpload,
  MediaUploadResult,
  PrivateMediaStore,
} from "./types";
