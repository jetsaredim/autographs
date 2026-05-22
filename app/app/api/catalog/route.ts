import { createCatalogService } from "../../../src/catalog";
import { toPublicGalleryItem } from "../../../src/catalog/public-view-models";

export const dynamic = "force-dynamic";

export async function GET(request: Request) {
  const url = new URL(request.url);
  const service = createCatalogService();
  const items = await service.list({
    category: url.searchParams.get("category") ?? undefined,
    signer: url.searchParams.get("signer") ?? undefined,
    tag: url.searchParams.get("tag") ?? undefined,
  });

  return Response.json({ items: items.map(toPublicGalleryItem) });
}
