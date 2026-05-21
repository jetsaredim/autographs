import Link from "next/link";

import { selectApprovedQuote } from "../../src/gallery/approved-quotes";

type EmptyStateVariant = "no-results" | "not-found" | "media-missing" | "catalog-error";

type EmptyStateProps = {
  variant: EmptyStateVariant;
};

const variantCopy: Record<EmptyStateVariant, { title: string; body: string; action: string; href: string }> = {
  "no-results": {
    title: "No published autographs match this view yet.",
    body: "Clear the filters or return to the full collection.",
    action: "Clear filters",
    href: "/collection",
  },
  "not-found": {
    title: "That autograph is not on the public shelf.",
    body: "It may be unpublished, moved, or waiting for a better lead.",
    action: "Back to collection",
    href: "/collection",
  },
  "media-missing": {
    title: "The image did not turn up.",
    body: "The item details are still available while the media route is checked.",
    action: "Back to collection",
    href: "/collection",
  },
  "catalog-error": {
    title: "The collection could not be loaded.",
    body: "Refresh the page or return to the collection.",
    action: "View collection",
    href: "/collection",
  },
};

export function EmptyState({ variant }: EmptyStateProps) {
  const quote = selectApprovedQuote();
  const copy = variantCopy[variant];

  return (
    <section className="empty-state" data-empty-state={variant}>
      <blockquote>
        <p>&ldquo;{quote.quote}&rdquo;</p>
        <footer>{quote.attribution}</footer>
      </blockquote>
      <div className="empty-state-copy">
        <h2>{copy.title}</h2>
        <p>{copy.body}</p>
        <Link className="button-secondary" href={copy.href}>
          {copy.action}
        </Link>
      </div>
    </section>
  );
}
