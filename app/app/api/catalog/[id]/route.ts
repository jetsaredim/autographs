import { createCatalogService } from "../../../../src/catalog";
import { toPublicItemDetail } from "../../../../src/catalog/public-view-models";

export const dynamic = "force-dynamic";

type RouteContext = {
  params: Promise<{ id: string }> | { id: string };
};

export async function GET(_request: Request, context: RouteContext) {
  const { id } = await context.params;
  const service = createCatalogService();
  const item = await service.getById(id);

  if (!item) {
    return Response.json({ error: "Not found" }, { status: 404 });
  }

  return Response.json({ item: toPublicItemDetail(item) });
}
