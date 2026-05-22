import Link from "next/link";
import { PublicFooter } from "./components/PublicFooter";
import { listPublishedCatalogItems } from "./catalog-data";
import { toPublicGalleryItem } from "../src/catalog/public-view-models";

export const dynamic = "force-dynamic";

export default async function HomePage() {
  const items = await listPublishedCatalogItems();
  const galleryItems = items.map(toPublicGalleryItem);
  const surpriseItem = galleryItems.length > 0 ? pickSurprise(galleryItems) : null;

  return (
    <main className="site-shell">
      <section className="landing-hero" aria-labelledby="landing-title">
        <div className="landing-copy">
          <h1 id="landing-title">Jared Greenwald&apos;s Autograph Gallery</h1>
          <p className="lede">
            A curated inventory of my personal autograph collection.
            <br />
            Organized for quiet browsing and the occasional happy discovery.
          </p>
          <div className="cta-row" aria-label="Gallery actions">
            <Link className="button-primary" href="/collection">
              Browse the Collection
            </Link>
            {surpriseItem ? (
              <Link className="button-secondary" href={`/collection/${surpriseItem.id}`}>
                Surprise Me
              </Link>
            ) : (
              <button className="button-secondary" type="button" disabled>
                Surprise Me
              </button>
            )}
          </div>
        </div>
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
