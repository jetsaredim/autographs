import { Buffer } from "node:buffer";

import { createCatalogService, type CatalogCreateInput } from "../../../../src/catalog";

export const dynamic = "force-dynamic";

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

type OperatorCreateRequest = Omit<CatalogCreateInput, "imageUploads"> & {
  imageUploads?: OperatorImageInput[];
};

export async function POST(request: Request) {
  const authResponse = authorizeOperator(request);
  if (authResponse) {
    return authResponse;
  }

  const body = (await request.json()) as OperatorCreateRequest;
  const service = createCatalogService();
  const item = await service.create({
    ...body,
    imageUploads: body.imageUploads?.map(toUploadInput),
  });

  return Response.json({ item }, { status: 201 });
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
