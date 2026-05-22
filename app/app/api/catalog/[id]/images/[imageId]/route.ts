import { Readable } from "node:stream";

import { createCatalogService } from "../../../../../../src/catalog";
import { getMockImageSvg } from "../../../../../../src/catalog/mock-data";

export const dynamic = "force-dynamic";

type RouteContext = {
  params: Promise<{ id: string; imageId: string }> | { id: string; imageId: string };
};

export async function GET(_request: Request, context: RouteContext) {
  const { id, imageId } = await context.params;
  const image = await readPublishedImage(id, imageId).catch((error: unknown) => {
    if (isMissingLocalDataConfig(error) && process.env.NODE_ENV !== "production") {
      return null;
    }
    throw error;
  });

  if (!image) {
    const mockSvg = process.env.NODE_ENV !== "production" ? getMockImageSvg(id, imageId) : null;
    if (mockSvg) {
      return new Response(mockSvg, {
        headers: {
          "Content-Type": "image/svg+xml",
          "Cache-Control": "no-store",
          "X-Content-Type-Options": "nosniff",
        },
      });
    }

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

const readPublishedImage = (id: string, imageId: string) =>
  createCatalogService().readPublishedImage(id, imageId);

const isMissingLocalDataConfig = (error: unknown): boolean =>
  error instanceof Error && error.message.startsWith("Oracle database configuration is incomplete.");
