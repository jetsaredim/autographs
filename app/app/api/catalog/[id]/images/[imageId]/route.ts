import { Readable } from "node:stream";

import { createCatalogService } from "../../../../../../src/catalog";

export const dynamic = "force-dynamic";

type RouteContext = {
  params: Promise<{ id: string; imageId: string }> | { id: string; imageId: string };
};

export async function GET(_request: Request, context: RouteContext) {
  const { id, imageId } = await context.params;
  const service = createCatalogService();
  const image = await service.readPublishedImage(id, imageId);

  if (!image) {
    return Response.json({ error: "Not found" }, { status: 404 });
  }

  const body = (
    image.body instanceof Readable ? Readable.toWeb(image.body) : image.body
  ) as BodyInit;

  return new Response(body, {
    headers: {
      "Content-Type": image.contentType,
      "Cache-Control": "public, max-age=300, stale-while-revalidate=3600",
      "X-Content-Type-Options": "nosniff",
    },
  });
}
