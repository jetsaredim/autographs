"use client";

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
    if (value) {
      next.set(key, value);
    } else {
      next.delete(key);
    }
    router.push(`/collection${next.toString() ? `?${next.toString()}` : ""}`);
  };

  return (
    <section className="gallery-filters" aria-label="Collection filters">
      <div className="filter-menu">
        {facets.map((facet) => (
          <label key={facet.id}>
            <span>{facet.label}</span>
            <select
              value={selected[facet.id] ?? ""}
              onChange={(event) => updateFilter(facet.id, event.target.value)}
            >
              <option value="">All</option>
              {facet.options.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
        ))}
      </div>

      {activeFilters.length > 0 ? (
        <div className="selected-filters" aria-label="Selected filters">
          {activeFilters.map((filter) => (
            <button
              className="filter-chip"
              key={filter.id}
              type="button"
              onClick={() => updateFilter(filter.id, "")}
            >
              {filter.label}
            </button>
          ))}
          <button className="filter-chip" type="button" onClick={() => router.push("/collection")}>
            Clear filters
          </button>
        </div>
      ) : null}
    </section>
  );
}
