import Link from "next/link";
import Image from "next/image";

import { PublicFooter } from "./components/PublicFooter";
import { listPublishedCatalogItems } from "./catalog-data";
import { toPublicGalleryItem } from "../src/catalog/public-view-models";

export const dynamic = "force-dynamic";

export default async function HomePage() {
  const items = await listPublishedCatalogItems();
  const galleryItems = items.map(toPublicGalleryItem);
  const featuredItem = galleryItems.find((item) => item.primaryImage) ?? galleryItems[0];
  const surpriseItem = galleryItems.length > 0 ? pickSurprise(galleryItems) : null;

  return (
    <main className="site-shell">
      <section className="landing-hero" aria-labelledby="landing-title">
        <div className="landing-copy">
          <p className="eyebrow">Personal autograph archive</p>
          <h1 id="landing-title">Jared Greenwald&apos;s Autograph Gallery</h1>
          <p className="lede">
            A public window into published signed memorabilia, organized for
            quiet browsing with the private image storage kept behind the app.
          </p>
          <div className="cta-row" aria-label="Gallery actions">
            <Link className="button-primary" href="/collection">
              View Collection
            </Link>
            {surpriseItem ? (
              <Link className="button-secondary" href={`/collection/${surpriseItem.id}`}>
                Surprise Me
              </Link>
            ) : null}
          </div>
        </div>

        <div className="landing-preview" aria-label="Featured autograph preview">
          {featuredItem?.primaryImage ? (
            <figure className="public-image-surface">
              <Image
                src={featuredItem.primaryImage.src}
                alt={featuredItem.primaryImage.altText}
                width={640}
                height={800}
                priority
                unoptimized
              />
            </figure>
          ) : (
            <div className="image-fallback">
              <span>The next published autograph will take this spot.</span>
            </div>
          )}
        </div>
      </section>

      <section className="landing-preview-copy" aria-label="Collection summary">
        <p className="eyebrow">Published collection</p>
        <p className="lede">
          Browse by signer, category, and meaningful tags, then open each item
          for image-forward detail views as the gallery comes online.
        </p>
      </section>

      <PublicFooter />
    </main>
  );
}

type SurpriseCandidate = {
  id: string;
};

const pickSurprise = <T extends SurpriseCandidate>(items: T[]): T => {
  const index = Math.floor(Math.random() * items.length);
  return items[index];
};
