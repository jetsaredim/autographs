"use client";

import type { ReactNode } from "react";
import { useMemo, useState } from "react";
import Image from "next/image";

import type { PublicImage } from "../../src/catalog/public-view-models";

type ImageViewerProps = {
  images: PublicImage[];
  title: string;
  signer: string;
  metadataPanel: ReactNode;
};

export function ImageViewer({ images, title, signer, metadataPanel }: ImageViewerProps) {
  const [selectedImageId, setSelectedImageId] = useState(images[0]?.id ?? "");
  const [revealed, setRevealed] = useState(false);
  const selectedImage = useMemo(
    () => images.find((image) => image.id === selectedImageId) ?? images[0] ?? null,
    [images, selectedImageId],
  );

  if (!selectedImage) {
    return (
      <section className="image-viewer image-viewer-empty">
        <div className="image-viewer-fallback">No public image is available for {title}.</div>
        <div className="detail-metadata-panel is-revealed">{metadataPanel}</div>
      </section>
    );
  }

  return (
    <section className={revealed ? "image-viewer is-revealed" : "image-viewer"}>
      <div className="image-viewer-gallery">
        <button
          className="focused-image-button"
          type="button"
          onClick={() => setRevealed((current) => !current)}
          aria-expanded={revealed}
        >
          <Image
            src={selectedImage.src}
            alt={selectedImage.altText}
            width={900}
            height={1125}
            priority
            draggable={false}
            unoptimized
            onContextMenu={(event) => event.preventDefault()}
          />
          <span className="sr-only">
            Toggle details for {title} signed by {signer}
          </span>
        </button>

        {images.length > 1 ? (
          <div className="image-thumbnails" aria-label={`${title} images`}>
            {images.map((image) => (
              <button
                className="thumbnail-button"
                type="button"
                key={image.id}
                aria-pressed={image.id === selectedImage.id}
                onClick={() => setSelectedImageId(image.id)}
                onContextMenu={(event) => event.preventDefault()}
              >
                <Image
                  src={image.src}
                  alt={image.altText}
                  width={176}
                  height={220}
                  draggable={false}
                  unoptimized
                />
              </button>
            ))}
          </div>
        ) : null}
      </div>

      <div className="detail-metadata-panel" aria-hidden={!revealed}>
        {metadataPanel}
      </div>
    </section>
  );
}
