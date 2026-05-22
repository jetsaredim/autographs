"use client";

import Image from "next/image";
import Link from "next/link";

import type { PublicGalleryItem } from "../../src/catalog/public-view-models";

type GalleryGridProps = {
  items: PublicGalleryItem[];
};

export function GalleryGrid({ items }: GalleryGridProps) {
  return (
    <section className="gallery-grid" aria-label="Published autograph items">
      {items.map((item) => (
        <Link
          className="gallery-card-link"
          href={`/collection/${item.id}`}
          key={item.id}
          aria-label={`${item.title} signed by ${item.signer}`}
        >
          <article className="gallery-card">
            <div className="gallery-card-media" onContextMenu={(event) => event.preventDefault()}>
              {item.primaryImage ? (
                <Image
                  src={item.primaryImage.src}
                  alt={item.primaryImage.altText}
                  width={480}
                  height={600}
                  draggable={false}
                  unoptimized
                />
              ) : (
                <span>No image published yet</span>
              )}
              <div className="gallery-card-overlay">
                <span>{item.signer}</span>
              </div>
            </div>
          </article>
        </Link>
      ))}
    </section>
  );
}
