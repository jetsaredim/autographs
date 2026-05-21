import { GalleryFilters } from "../components/GalleryFilters";
import { GalleryGrid } from "../components/GalleryGrid";
import { EmptyState } from "../components/EmptyState";
import { createCatalogService } from "../../src/catalog";
import { derivePublicFacets, toPublicGalleryItem } from "../../src/catalog/public-view-models";

export const dynamic = "force-dynamic";

type CollectionPageProps = {
  searchParams: Promise<{
    signer?: string;
    category?: string;
    tag?: string;
  }>;
};

export default async function CollectionPage({ searchParams }: CollectionPageProps) {
  const params = await searchParams;
  const filters = {
    signer: params.signer,
    category: params.category,
    tag: params.tag,
  };
  const service = createCatalogService();
  const [filteredItems, allPublishedItems] = await Promise.all([
    service.list({
      signer: filters.signer,
      category: filters.category,
      tag: filters.tag,
    }),
    service.list(),
  ]);
  const galleryItems = filteredItems.map(toPublicGalleryItem);
  const facetGroups = derivePublicFacets(allPublishedItems);

  return (
    <main className="site-shell collection-shell">
      <section className="collection-heading" aria-labelledby="collection-title">
        <p className="eyebrow">Published collection</p>
        <h1 id="collection-title">Collection</h1>
        <p className="lede">
          {galleryItems.length === 1
            ? "1 published autograph"
            : `${galleryItems.length} published autographs`}
        </p>
      </section>

      <GalleryFilters facets={facetGroups} selected={filters} />

      {galleryItems.length > 0 ? (
        <GalleryGrid items={galleryItems} />
      ) : (
        <EmptyState variant="no-results" />
      )}
    </main>
  );
}
