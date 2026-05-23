import { createCatalogService } from "../../../../../../../src/catalog";

export const dynamic = "force-dynamic";

type RouteContext = {
  params:
    | Promise<{ id: string; imageId: string }>
    | { id: string; imageId: string };
};

export async function DELETE(request: Request, context: RouteContext) {
  const authResponse = authorizeOperator(request);
  if (authResponse) {
    return authResponse;
  }

  const { id, imageId } = await context.params;
  const service = createCatalogService();

  try {
    const item = await service.deleteImage(id, imageId);
    return Response.json({ item });
  } catch (error) {
    if (isNotFoundError(error)) {
      return Response.json({ error: "Not found" }, { status: 404 });
    }

    throw error;
  }
}

const authorizeOperator = (request: Request): Response | null => {
  const token = process.env.AUTOGRAPHS_OPERATOR_API_TOKEN;
  if (!token) {
    return Response.json({ error: "Operator API is disabled" }, { status: 404 });
  }

  const providedToken = request.headers
    .get("authorization")
    ?.replace(/^Bearer\s+/i, "");
  if (providedToken !== token) {
    return Response.json({ error: "Unauthorized" }, { status: 401 });
  }

  return null;
};

const isNotFoundError = (error: unknown): boolean =>
  error instanceof Error &&
  (error.message.startsWith("Autograph item ") ||
    error.message.startsWith("Autograph image ")) &&
  error.message.endsWith(" was not found.");

