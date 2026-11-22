export const PAGE_SIZE = 12;

export const CATALOGUE_DESCRIPTION =
  "Browse every DeskPet pixel pet. Search, filter by category, author and type, and find your next desktop companion.";

export type SortKey = "az" | "newest" | "tricks";
export type TypeKey = "included" | "addon";

export interface CataloguePet {
  id: string;
  name: string;
  description: string;
  author: string;
  category: string;
  builtin: boolean;
  trickCount: number;
  authorSlug?: string;
}

export interface IndexPet extends CataloguePet {
  shortName: string;
  authorSlug: string;
  gallery: { src: string; width: number; height: number; zoom: number };
}

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

function compareAz(a: CataloguePet, b: CataloguePet): number {
  return a.name.localeCompare(b.name) || a.id.localeCompare(b.id);
}

export function sortPets<T extends CataloguePet>(list: readonly T[], sort: SortKey): T[] {
  if (sort === "newest") return [...list].reverse();
  if (sort === "tricks") {
    return [...list].sort((a, b) => b.trickCount - a.trickCount || compareAz(a, b));
  }
  return [...list].sort(compareAz);
}

export function filterPets<T extends CataloguePet>(
  list: readonly T[],
  state: Pick<CatalogueState, "q" | "category" | "author" | "type">,
  slugOf: (pet: T) => string,
): T[] {
  const query = state.q.trim().toLowerCase();
  return list.filter((pet) => {
    if (
      query &&
      ![pet.name, pet.description, pet.author].some((value) => value.toLowerCase().includes(query))
    ) {
      return false;
    }
    if (state.category && pet.category !== state.category) return false;
    if (state.author && slugOf(pet) !== state.author) return false;
    if (state.type === "included" && !pet.builtin) return false;
    if (state.type === "addon" && pet.builtin) return false;
    return true;
  });
}

export function parseState(
  params: URLSearchParams,
  valid: { categories: readonly string[]; authors: readonly string[] },
): CatalogueState {
  const q = (params.get("q") ?? "").trim();

  const categoryParam = params.get("category") ?? "";
  const category = categoryParam && valid.categories.includes(categoryParam) ? categoryParam : "";

  const authorParam = params.get("author") ?? "";
  const author = authorParam && valid.authors.includes(authorParam) ? authorParam : "";

  const typeParam = params.get("type") ?? "";
  const type = typeParam === "included" || typeParam === "addon" ? typeParam : "";

  const sortParam = params.get("sort") ?? "";
  const sort =
    sortParam === "az" || sortParam === "newest" || sortParam === "tricks" ? sortParam : "az";

  const pageParam = Number.parseInt(params.get("page") ?? "", 10);
  const page = Number.isInteger(pageParam) && pageParam >= 1 ? pageParam : 1;

  return { q, category, author, type, sort, page };
}

export function pagePath(basePath: string, page: number): string {
  return page === 1 ? basePath : `${basePath}page/${page}/`;
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
