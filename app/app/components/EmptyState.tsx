import Link from "next/link";

import { selectApprovedQuote } from "../../src/gallery/approved-quotes";

type EmptyStateVariant = "no-results" | "not-found" | "media-missing" | "catalog-error";

type EmptyStateProps = {
  variant: EmptyStateVariant;
};

type CopyVariant = Exclude<EmptyStateVariant, "no-results">;

const variantCopy: Record<CopyVariant, { title: string; body: string; action: string; href: string }> = {
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
  const quoteBlock = (
    <blockquote>
      <p>
        <span aria-hidden="true">&ldquo;</span>
        {quote.quote}
        <span aria-hidden="true">&rdquo;</span>
      </p>
      <footer>{quote.attribution}</footer>
    </blockquote>
  );

  if (variant === "no-results") {
    return (
      <section className="empty-state" data-empty-state={variant}>
        <Link className="empty-state-quote-link" href="/collection" aria-label="Show the full collection">
          {quoteBlock}
        </Link>
      </section>
    );
  }

  const copy = variantCopy[variant];

  return (
    <section className="empty-state" data-empty-state={variant}>
      {quoteBlock}
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
