import { absoluteUrl, authorGroups, authorSlug, categories, pets, type Pet } from "./site";

export const PAGE_SIZE = 12;

export type SortKey = "az" | "newest" | "tricks";
export type TypeKey = "included" | "addon";

export interface CatalogueState {
  q: string;
  category: string;
  author: string;
  type: "" | TypeKey;
  sort: SortKey;
  page: number;
}

export const DEFAULT_STATE: CatalogueState = {
  q: "",
  category: "",
  author: "",
  type: "",
  sort: "az",
  page: 1,
};

export type ListingScope =
  | { kind: "all" }
  | { kind: "category"; id: string }
  | { kind: "author"; slug: string };

export interface ListingRecord {
  scope: ListingScope;
  basePath: string;
  page: number;
  pageCount: number;
  total: number;
  path: string;
  pets: Pet[];
}

function compareAz(a: Pet, b: Pet): number {
  return a.name.localeCompare(b.name) || a.id.localeCompare(b.id);
}

export function sortPets(list: readonly Pet[], sort: SortKey): Pet[] {
  if (sort === "newest") return [...list].reverse();
  if (sort === "tricks") {
    return [...list].sort((a, b) => b.trickCount - a.trickCount || compareAz(a, b));
  }
  return [...list].sort(compareAz);
}

export function filterPets(
  list: readonly Pet[],
  state: Pick<CatalogueState, "q" | "category" | "author" | "type">,
): Pet[] {
  const query = state.q.trim().toLowerCase();
  return list.filter((pet) => {
    if (
      query &&
      ![pet.name, pet.description, pet.author].some((value) =>
        value.toLowerCase().includes(query),
      )
    ) {
      return false;
    }
    if (state.category && pet.category !== state.category) return false;
    if (state.author && authorSlug(pet.author) !== state.author) return false;
    if (state.type === "included" && !pet.builtin) return false;
    if (state.type === "addon" && pet.builtin) return false;
    return true;
  });
}

export function pagePath(basePath: string, page: number): string {
  return page === 1 ? basePath : `${basePath}page/${page}/`;
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
  const q = (params.get("q") ?? "").trim();

  const categoryParam = params.get("category") ?? "";
  const category =
    categoryParam && list.some((pet) => pet.category === categoryParam) ? categoryParam : "";

  const authorParam = params.get("author") ?? "";
  const author =
    authorParam && list.some((pet) => authorSlug(pet.author) === authorParam) ? authorParam : "";

  const typeParam = params.get("type") ?? "";
  const type = typeParam === "included" || typeParam === "addon" ? typeParam : "";

  const sortParam = params.get("sort") ?? "";
  const sort =
    sortParam === "az" || sortParam === "newest" || sortParam === "tricks" ? sortParam : "az";

  const pageParam = Number.parseInt(params.get("page") ?? "", 10);
  const page = Number.isInteger(pageParam) && pageParam >= 1 ? pageParam : 1;

  return { q, category, author, type, sort, page };
}

export function stateParams(state: CatalogueState): URLSearchParams {
  const params = new URLSearchParams();
  if (state.q) params.set("q", state.q);
  if (state.category) params.set("category", state.category);
  if (state.author) params.set("author", state.author);
  if (state.type) params.set("type", state.type);
  if (state.sort !== "az") params.set("sort", state.sort);
  if (state.page > 1) params.set("page", String(state.page));
  return params;
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