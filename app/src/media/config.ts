import { existsSync, readFileSync } from "node:fs";

export type MediaStorageProvider = "oci" | "local";

export type MediaStorageConfig = {
  provider: MediaStorageProvider;
  region: string;
  namespace: string;
  bucketName: string;
  localRoot: string;
  oci?: {
    tenancyOcid: string;
    userOcid: string;
    fingerprint: string;
    privateKey: string;
  };
};

const readEnv = (name: string): string | undefined => {
  const value = process.env[name];
  return value && value.trim().length > 0 ? value : undefined;
};

const readPrivateKey = (): string | undefined => {
  const privateKeyPem = readEnv("OCI_PRIVATE_KEY_PEM");
  if (privateKeyPem) {
    return privateKeyPem;
  }

  const privateKeyPath = readEnv("OCI_PRIVATE_KEY_PATH");
  if (privateKeyPath && existsSync(privateKeyPath)) {
    return readFileSync(privateKeyPath, "utf8");
  }

  return undefined;
};

export const getMediaStorageConfig = (): MediaStorageConfig => {
  const provider = (readEnv("AUTOGRAPHS_MEDIA_STORAGE_PROVIDER") ?? "oci") as MediaStorageProvider;
  const region = readEnv("OCI_REGION") ?? "us-ashburn-1";
  const namespace = readEnv("OCI_MEDIA_NAMESPACE");
  const bucketName = readEnv("OCI_MEDIA_BUCKET_NAME") ?? "autographs-media-prod";
  const localRoot = readEnv("AUTOGRAPHS_LOCAL_MEDIA_ROOT") ?? "/tmp/autographs-media";

  if (provider !== "oci" && provider !== "local") {
    throw new Error("AUTOGRAPHS_MEDIA_STORAGE_PROVIDER must be oci or local.");
  }

  if (provider === "local") {
    return {
      provider,
      region,
      namespace: namespace ?? "local",
      bucketName,
      localRoot,
    };
  }

  if (!namespace) {
    throw new Error("OCI_MEDIA_NAMESPACE is required for OCI media storage.");
  }

  const tenancyOcid = readEnv("OCI_TENANCY_OCID");
  const userOcid = readEnv("OCI_CLI_USER_OCID");
  const fingerprint = readEnv("OCI_FINGERPRINT");
  const privateKey = readPrivateKey();

  if (!tenancyOcid || !userOcid || !fingerprint || !privateKey) {
    throw new Error(
      "OCI media storage is incomplete. Set OCI_TENANCY_OCID, OCI_CLI_USER_OCID, OCI_FINGERPRINT, and OCI_PRIVATE_KEY_PEM or OCI_PRIVATE_KEY_PATH.",
    );
  }

  return {
    provider,
    region,
    namespace,
    bucketName,
    localRoot,
    oci: {
      tenancyOcid,
      userOcid,
      fingerprint,
      privateKey,
    },
  };
};
