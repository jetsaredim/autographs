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
  const body = (await request.json()) as OperatorPatchRequest;
  const service = createCatalogService();
  const updated = body.item ? await service.update(id, body.item) : await service.getById(id, { includeUnpublished: true });

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
