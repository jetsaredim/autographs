import { Buffer } from "node:buffer";

import { createCatalogService, type AutographItemUpdate } from "../../../../../src/catalog";

export const dynamic = "force-dynamic";

type RouteContext = {
  params: Promise<{ id: string }> | { id: string };
};

type OperatorImageInput = {
  filename: string;
  contentType: string;
  bodyBase64: string;
  byteSize?: number | null;
  metadata?: Record<string, string>;
  isPrimary?: boolean;
  sortOrder?: number;
  altText?: string | null;
};

type OperatorPatchRequest = {
  item?: AutographItemUpdate;
  imageUploads?: OperatorImageInput[];
};

export async function PATCH(request: Request, context: RouteContext) {
  const authResponse = authorizeOperator(request);
  if (authResponse) {
    return authResponse;
  }

  const { id } = await context.params;
  const body = await parsePatchRequest(request);
  const service = createCatalogService();
  const updated = body.item 
    ? await service.update(id, body.item) 
    : await service.getById(id, { includeUnpublished: true });

  if (!updated) {
    return Response.json({ error: "Not found" }, { status: 404 });
  }

  const item =
    body.imageUploads && body.imageUploads.length > 0
      ? await service.attachImages(id, body.imageUploads.map(toUploadInput))
      : updated;

  return Response.json({ item });
}

const authorizeOperator = (request: Request): Response | null => {
  const token = process.env.AUTOGRAPHS_OPERATOR_API_TOKEN;
  if (!token) {
    return Response.json({ error: "Operator API is disabled" }, { status: 404 });
  }

  const providedToken = request.headers.get("authorization")?.replace(/^Bearer\s+/i, "");
  if (providedToken !== token) {
    return Response.json({ error: "Unauthorized" }, { status: 401 });
  }

  return null;
};

const parsePatchRequest = async (request: Request): Promise<OperatorPatchRequest> => {
  const contentType = request.headers.get("content-type") ?? "";

  if (contentType.includes("multipart/form-data")) {
    return parseMultipartPatchRequest(request);
  }

  return (await request.json()) as OperatorPatchRequest;
};

const parseMultipartPatchRequest = async (
  request: Request,
): Promise<OperatorPatchRequest> => {
  const formData = await request.formData();

  const itemValue = formData.get("item");
  const item =
    typeof itemValue === "string" && itemValue.trim().length > 0
      ? (JSON.parse(itemValue) as AutographItemUpdate)
      : undefined;

  const altTextValue = formData.get("altText");
  const altText = typeof altTextValue === "string" && altTextValue.trim().length > 0
    ? altTextValue
    : undefined;

  const files = [...formData.getAll("image"), ...formData.getAll("images")].filter(
    (value): value is File => value instanceof File && value.size > 0
  );

  const imageUploads = await Promise.all(
    files.map(async (file, index): Promise<OperatorImageInput> => ({
      filename: file.name || `image-${index +1}`,
      contentType: file.type || "application/octet-stream",
      bodyBase64: Buffer.from(await file.arrayBuffer()).toString("base64"),
      byteSize: file.size,
      isPrimary: index === 0,
      sortOrder: index,
      altText: altText ?? (file.name || null),
    })),
  );

  return {
    item,
    imageUploads,
  };
};

const toUploadInput = (image: OperatorImageInput) => ({
  filename: image.filename,
  contentType: image.contentType,
  body: Buffer.from(image.bodyBase64, "base64"),
  byteSize: image.byteSize ?? Buffer.byteLength(image.bodyBase64, "base64"),
  metadata: image.metadata,
  isPrimary: image.isPrimary,
  sortOrder: image.sortOrder,
  altText: image.altText,
});
