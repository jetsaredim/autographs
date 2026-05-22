import { Breadcrumbs } from "../components/Breadcrumbs";
import { GalleryFilters } from "../components/GalleryFilters";
import { GalleryGrid } from "../components/GalleryGrid";
import { EmptyState } from "../components/EmptyState";
import { PublicFooter } from "../components/PublicFooter";
import { listPublishedCatalogItems } from "../catalog-data";
import { derivePublicFacets, toPublicGalleryItem } from "../../src/catalog/public-view-models";

export const dynamic = "force-dynamic";

type CollectionPageProps = {
  searchParams: Promise<{
    signer?: string | string[];
    category?: string | string[];
    tag?: string | string[];
  }>;
};

export default async function CollectionPage({ searchParams }: CollectionPageProps) {
  const params = await searchParams;
  const filters = {
    signer: normalizeFilter(params.signer),
    category: normalizeFilter(params.category),
    tag: normalizeFilter(params.tag),
  };
  const [filteredItems, allPublishedItems] = await Promise.all([
    listPublishedCatalogItems({
      signer: filters.signer,
      category: filters.category,
      tag: filters.tag,
    }),
    listPublishedCatalogItems(),
  ]);
  const galleryItems = filteredItems.map(toPublicGalleryItem);
  const facetGroups = derivePublicFacets(allPublishedItems);

  return (
    <main className="site-shell collection-shell">
      <section className="collection-heading" aria-labelledby="collection-title">
        <Breadcrumbs items={[{ label: "Home", href: "/" }, { label: "Collection" }]} />
        <h1 id="collection-title">Collection</h1>
        <p className="lede">
          {galleryItems.length === 1
            ? "1 published autograph"
            : `${galleryItems.length} published autographs`}
        </p>
      </section>

      <section className="collection-panel" aria-label="Collection items">
        <GalleryFilters facets={facetGroups} selected={filters} />

        {galleryItems.length > 0 ? (
          <GalleryGrid items={galleryItems} />
        ) : (
          <EmptyState variant="no-results" />
        )}
      </section>

      <PublicFooter />
    </main>
  );
}

const normalizeFilter = (value: string | string[] | undefined): string | undefined => {
  const candidate = Array.isArray(value) ? value[0] : value;
  return candidate && candidate !== "all" ? candidate : undefined;
};
