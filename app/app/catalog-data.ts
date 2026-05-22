import { createCatalogService } from "../src/catalog";
import type { AutographItem, AutographListOptions } from "../src/catalog";

const missingOracleConfigMessage = "Oracle database configuration is incomplete.";

export const listPublishedCatalogItems = async (
  options?: AutographListOptions,
): Promise<AutographItem[]> => {
  try {
    return await createCatalogService().list(options);
  } catch (error) {
    if (isMissingLocalDataConfig(error)) {
      return [];
    }
    throw error;
  }
};

export const getPublishedCatalogItem = async (id: string): Promise<AutographItem | null> => {
  try {
    return await createCatalogService().getById(id);
  } catch (error) {
    if (isMissingLocalDataConfig(error)) {
      return null;
    }
    throw error;
  }
};

const isMissingLocalDataConfig = (error: unknown): boolean =>
  error instanceof Error && error.message.startsWith(missingOracleConfigMessage);
