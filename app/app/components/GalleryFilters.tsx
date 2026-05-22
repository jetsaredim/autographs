"use client";

import { useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";

import type { PublicFacetGroup } from "../../src/catalog/public-view-models";

type SelectedFilters = {
  signer?: string;
  category?: string;
  tag?: string;
};

type GalleryFiltersProps = {
  facets: PublicFacetGroup[];
  selected: SelectedFilters;
};

export function GalleryFilters({ facets, selected }: GalleryFiltersProps) {
  const [isOpen, setIsOpen] = useState(false);
  const router = useRouter();
  const searchParams = useSearchParams();
  const activeFilters = facets.flatMap((facet) => {
    const value = selected[facet.id];
    if (!value) {
      return [];
    }

    const label = facet.options.find((option) => option.value === value)?.label ?? value;
    return [{ id: facet.id, label: `${facet.label}: ${label}` }];
  });

  const updateFilter = (key: PublicFacetGroup["id"], value: string) => {
    const next = new URLSearchParams(searchParams.toString());
    if (value && value !== "all") {
      next.set(key, value);
    } else {
      next.delete(key);
    }
    router.push(`/collection${next.toString() ? `?${next.toString()}` : ""}`);
  };

  return (
    <>
      <button
        className="filter-toggle"
        type="button"
        aria-expanded={isOpen}
        aria-controls="collection-filters"
        aria-label={isOpen ? "Close filters" : "Open filters"}
        onClick={() => setIsOpen((current) => !current)}
      >
        {isOpen ? (
          <svg aria-hidden="true" viewBox="0 0 24 24">
            <path d="M6 6l12 12M18 6L6 18" />
          </svg>
        ) : (
          <svg aria-hidden="true" viewBox="0 0 24 24">
            <path d="M4 6h16l-6.5 7.5V19l-3 1.5v-7z" />
          </svg>
        )}
      </button>

      {isOpen ? (
        <section className="gallery-filters" id="collection-filters" aria-label="Collection filters">
          <div className="filter-menu">
            {facets.map((facet) => (
              <select
                aria-label={facet.label}
                key={facet.id}
                value={selected[facet.id] ?? "all"}
                onChange={(event) => updateFilter(facet.id, event.target.value)}
              >
                <option value="all">{facet.label}</option>
                {facet.options.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            ))}
          </div>

          {activeFilters.length > 0 ? (
            <div className="selected-filters" aria-label="Selected filters">
              {activeFilters.map((filter) => (
                <button
                  className="filter-chip"
                  key={filter.id}
                  type="button"
                  onClick={() => updateFilter(filter.id, "all")}
                >
                  {filter.label}
                </button>
              ))}
            </div>
          ) : null}
        </section>
      ) : null}
    </>
  );
}
