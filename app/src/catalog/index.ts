import { createOracleExecutor } from "../db/oracle";
import { createPrivateMediaStore } from "../media";
import { OracleCatalogRepository } from "./repository";
import { DefaultCatalogService } from "./service";

export const createCatalogRepository = (): OracleCatalogRepository =>
  new OracleCatalogRepository(createOracleExecutor());

export const createCatalogService = (): DefaultCatalogService =>
  new DefaultCatalogService(createCatalogRepository(), createPrivateMediaStore());

export type { CatalogService, CatalogCreateInput, CatalogImageUploadInput } from "./service";
export type {
  AutographImage,
  AutographImageInput,
  AutographItem,
  AutographItemInput,
  AutographItemUpdate,
  AutographListOptions,
  CatalogRepository,
  PublicationStatus,
} from "./types";
