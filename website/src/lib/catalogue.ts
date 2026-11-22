import { absoluteUrl, authorGroups, authorSlug, categories, pets, type Pet } from "./site";
import {
  PAGE_SIZE,
  filterPets as filterCataloguePets,
  pagePath,
  parseState as parseCatalogueState,
  sortPets as sortCataloguePets,
  type CatalogueState,
  type ListingScope,
  type SortKey,
} from "./catalogue-core";

export {
  CATALOGUE_DESCRIPTION,
  DEFAULT_STATE,
  PAGE_SIZE,
  pagePath,
  stateParams,
  type CataloguePet,
  type CatalogueState,
  type IndexPet,
  type ListingScope,
  type SortKey,
  type TypeKey,
} from "./catalogue-core";

export interface ListingRecord {
  scope: ListingScope;
  basePath: string;
  page: number;
  pageCount: number;
  total: number;
  path: string;
  pets: Pet[];
}

export function sortPets(list: readonly Pet[], sort: SortKey): Pet[] {
  return sortCataloguePets(list, sort);
}

export function filterPets(
  list: readonly Pet[],
  state: Pick<CatalogueState, "q" | "category" | "author" | "type">,
): Pet[] {
  return filterCataloguePets(list, state, (pet) => authorSlug(pet.author));
}

/** Category ids in list, derived shelves first, then curated shelves in `categories` order. */
function presentCategoryIds(list: readonly Pet[]): string[] {
  const present = new Set(list.map((pet) => pet.category));
  const curated = categories.filter((category) => present.has(category.id)).map((c) => c.id);
  const derived = [...present].filter((id) => !categories.some((c) => c.id === id)).sort();
  return [...derived, ...curated];
}

export function listingRecords(list: readonly Pet[] = pets): ListingRecord[] {
  const scopes: { scope: ListingScope; basePath: string; scopePets: readonly Pet[] }[] = [
    { scope: { kind: "all" }, basePath: "/pets/", scopePets: list },
  ];
  for (const id of presentCategoryIds(list)) {
    scopes.push({
      scope: { kind: "category", id },
      basePath: `/pets/category/${id}/`,
      scopePets: list.filter((pet) => pet.category === id),
    });
  }
  for (const group of authorGroups(list)) {
    scopes.push({
      scope: { kind: "author", slug: group.slug },
      basePath: `/authors/${group.slug}/`,
      scopePets: group.pets,
    });
  }

  const records: ListingRecord[] = [];
  for (const { scope, basePath, scopePets } of scopes) {
    const total = scopePets.length;
    const pageCount = Math.max(1, Math.ceil(total / PAGE_SIZE));
    const sorted = sortPets(scopePets, "az");
    for (let page = 1; page <= pageCount; page++) {
      records.push({
        scope,
        basePath,
        page,
        pageCount,
        total,
        path: pagePath(basePath, page),
        pets: sorted.slice((page - 1) * PAGE_SIZE, page * PAGE_SIZE),
      });
    }
  }
  return records;
}

export function parseState(params: URLSearchParams, list: readonly Pet[] = pets): CatalogueState {
  return parseCatalogueState(params, {
    categories: [...new Set(list.map((pet) => pet.category))],
    authors: authorGroups(list).map((group) => group.slug),
  });
}

export function breadcrumbSchema(
  items: readonly { label: string; path: string }[],
): Record<string, unknown> {
  return {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: items.map((item, index) => ({
      "@type": "ListItem",
      position: index + 1,
      name: item.label,
      item: absoluteUrl(item.path),
    })),
  };
}
