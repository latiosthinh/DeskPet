import { describe, expect, it } from "vitest";
import { CATEGORY_ID, deriveCategories } from "../../scripts/lib/categories.mjs";
import {
  DEFAULT_STATE,
  PAGE_SIZE,
  breadcrumbSchema,
  filterPets,
  listingRecords,
  pagePath,
  parseState,
  sortPets,
  stateParams,
  type CatalogueState,
} from "../../src/lib/catalogue";
import { authorGroups, authorSlug, type Pet } from "../../src/lib/site";

const forest = Array.from({ length: 13 }, (_, index) => {
  const number = String(index + 1).padStart(2, "0");
  return {
    id: `f${number}`,
    name: `Forest ${number}`,
    category: "forest-friend",
    author: "DeskPet",
    builtin: index === 0,
    trickCount: index % 4,
    description: index % 3 === 0 ? "A mossy little friend." : "A forest friend.",
  };
});

const fixture = [
  ...forest,
  {
    id: "ab1",
    name: "Alpha One",
    category: "critter",
    author: "A B",
    builtin: false,
    trickCount: 2,
    description: "Alpha of the grove.",
  },
  {
    id: "ab2",
    name: "Alpha Two",
    category: "critter",
    author: "A-B",
    builtin: false,
    trickCount: 2,
    description: "Beta of the grove.",
  },
  {
    id: "uni1",
    name: "Unicode One",
    category: "critter",
    author: "一",
    builtin: false,
    trickCount: 1,
    description: "First unicode friend.",
  },
  {
    id: "uni2",
    name: "Unicode Two",
    category: "critter",
    author: "业",
    builtin: false,
    trickCount: 1,
    description: "Second unicode friend.",
  },
] as unknown as Pet[];

describe("listingRecords", () => {
  it("lists every scope and page in order", () => {
    const records = listingRecords(fixture);
    const authorPaths = authorGroups(fixture)
      .filter((group) => group.slug !== "deskpet")
      .map((group) => `/authors/${group.slug}/`);

    expect(records.map((record) => record.path)).toEqual([
      "/pets/",
      "/pets/page/2/",
      "/pets/category/forest-friend/",
      "/pets/category/forest-friend/page/2/",
      "/pets/category/critter/",
      "/authors/deskpet/",
      "/authors/deskpet/page/2/",
      ...authorPaths,
    ]);

    const all = records.filter((record) => record.scope.kind === "all");
    expect(all.map((record) => [record.page, record.pageCount, record.total])).toEqual([
      [1, 2, 17],
      [2, 2, 17],
    ]);
    expect(all[0].pets.map((pet) => pet.name)).toEqual([
      "Alpha One",
      "Alpha Two",
      "Forest 01",
      "Forest 02",
      "Forest 03",
      "Forest 04",
      "Forest 05",
      "Forest 06",
      "Forest 07",
      "Forest 08",
      "Forest 09",
      "Forest 10",
    ]);
    expect(all[1].pets.map((pet) => pet.id)).toEqual(["f11", "f12", "f13", "uni1", "uni2"]);

    const forestRecords = records.filter(
      (record) => record.scope.kind === "category" && record.scope.id === "forest-friend",
    );
    expect(forestRecords.map((record) => [record.page, record.pageCount, record.total])).toEqual([
      [1, 2, 13],
      [2, 2, 13],
    ]);
    expect(forestRecords[0].pets).toHaveLength(PAGE_SIZE);
    expect(forestRecords[1].pets.map((pet) => pet.id)).toEqual(["f13"]);

    const critter = records.find(
      (record) => record.scope.kind === "category" && record.scope.id === "critter",
    );
    expect(critter?.pageCount).toBe(1);
    expect(critter?.total).toBe(4);

    const similar = authorGroups(fixture).filter((group) => group.slug !== "deskpet");
    expect(similar).toHaveLength(4);
    for (let i = 0; i < similar.length; i++) {
      for (let j = i + 1; j < similar.length; j++) {
        expect(similar[i].slug).not.toBe(similar[j].slug);
      }
    }
    for (const group of similar) {
      expect(
        records.filter(
          (record) => record.scope.kind === "author" && record.scope.slug === group.slug,
        ),
      ).toHaveLength(1);
    }
  });

  it("keeps a single all-scope record at the 12-pet page boundary", () => {
    const records = listingRecords(fixture.slice(0, 12));
    const all = records.filter((record) => record.scope.kind === "all");
    expect(all).toHaveLength(1);
    expect(all[0]).toMatchObject({ page: 1, pageCount: 1, total: 12 });
  });

  it("keeps one empty record for an empty list", () => {
    const records = listingRecords([]);
    expect(records).toHaveLength(1);
    expect(records[0]).toMatchObject({
      basePath: "/pets/",
      page: 1,
      pageCount: 1,
      total: 0,
      path: "/pets/",
      pets: [],
    });
  });
});

describe("deriveCategories", () => {
  it("keeps known copy first and titles derived shelves", () => {
    const derived = deriveCategories(fixture);
    expect(derived.map((category) => category.id)).toEqual(["critter", "forest-friend"]);
    expect(derived[0]).toEqual({
      id: "critter",
      title: "Critters",
      blurb: "Small, soft and always up to something.",
    });
    expect(derived[1]).toEqual({
      id: "forest-friend",
      title: "Forest Friend",
      blurb: "Community pets on the Forest Friend shelf.",
    });
  });
});

describe("stable ordering", () => {
  it("bounds long non-latin author slugs with a fixed-length digest", () => {
    const slug = authorSlug("作".repeat(42));
    expect(slug).toMatch(/^author-[0-9a-f]{32}$/);
    expect(slug.length).toBeLessThanOrEqual(65);
  });

  it("keeps author slugs unique and listings stable under input order", () => {
    expect(authorSlug("A B")).not.toBe(authorSlug("A-B"));
    expect(authorSlug("一")).not.toBe(authorSlug("业"));
    expect(authorSlug("一").toLowerCase()).not.toBe(authorSlug("业").toLowerCase());
    const names = [...new Set(fixture.map((pet) => pet.author))];
    for (let i = 0; i < names.length; i++) {
      for (let j = i + 1; j < names.length; j++) {
        expect(authorSlug(names[i]).toLowerCase()).not.toBe(authorSlug(names[j]).toLowerCase());
      }
    }
    const slugs = authorGroups(fixture).map((group) => group.slug);
    for (const slug of slugs) expect(slug).toMatch(/^[A-Za-z0-9_-]+$/);

    const shuffled = [...fixture].reverse();
    expect(listingRecords(shuffled).map((record) => record.path)).toEqual(
      listingRecords(fixture).map((record) => record.path),
    );
    expect(authorGroups(shuffled).map((group) => group.slug)).toEqual(slugs);
  });
});

describe("sortPets", () => {
  it("reverses source order for newest and returns a new array", () => {
    const sorted = sortPets(fixture, "newest");
    expect(sorted).not.toBe(fixture);
    expect(sorted.map((pet) => pet.id)).toEqual([...fixture].reverse().map((pet) => pet.id));
  });

  it("orders tricks by count then name", () => {
    const pets = [
      { id: "t2", name: "Beta", trickCount: 3 },
      { id: "t1", name: "Alpha", trickCount: 3 },
      { id: "t3", name: "Alpha", trickCount: 5 },
      { id: "t4", name: "Alpha", trickCount: 3 },
    ] as unknown as Pet[];
    expect(sortPets(pets, "tricks").map((pet) => pet.id)).toEqual(["t3", "t1", "t4", "t2"]);
  });

  it("breaks az ties by id", () => {
    const pets = [
      { id: "b", name: "Same" },
      { id: "a", name: "Same" },
      { id: "c", name: "Alpha" },
    ] as unknown as Pet[];
    expect(sortPets(pets, "az").map((pet) => pet.id)).toEqual(["c", "a", "b"]);
  });
});

describe("filterPets", () => {
  it("matches name, description or author case-insensitively after trimming", () => {
    expect(
      filterPets(fixture, { q: "  MOSS  ", category: "", author: "", type: "" }).map(
        (pet) => pet.id,
      ),
    ).toEqual(["f01", "f04", "f07", "f10", "f13"]);
    expect(
      filterPets(fixture, { q: "forest 01", category: "", author: "", type: "" }).map(
        (pet) => pet.id,
      ),
    ).toEqual(["f01"]);
    expect(filterPets(fixture, { q: "deskpet", category: "", author: "", type: "" })).toHaveLength(
      13,
    );
    expect(
      filterPets(fixture, { q: "a-b", category: "", author: "", type: "" }).map((pet) => pet.id),
    ).toEqual(["ab2"]);
  });

  it("intersects category, author, type and query filters", () => {
    const state = {
      q: "moss",
      category: "forest-friend",
      author: authorSlug("DeskPet"),
      type: "included" as const,
    };
    expect(filterPets(fixture, state).map((pet) => pet.id)).toEqual(["f01"]);
    expect(
      filterPets(fixture, { q: "", category: "", author: "", type: "included" }).map(
        (pet) => pet.id,
      ),
    ).toEqual(["f01"]);
    expect(filterPets(fixture, { q: "", category: "", author: "", type: "addon" })).toHaveLength(
      16,
    );
    expect(
      filterPets(fixture, { q: "", category: "", author: authorSlug("A B"), type: "" }).map(
        (pet) => pet.id,
      ),
    ).toEqual(["ab1"]);
  });
});

describe("parseState", () => {
  it("keeps valid values and drops unknown ones", () => {
    const params = new URLSearchParams(
      "q=%20Forest%20&category=forest-friend&author=deskpet&type=included&sort=tricks&page=2",
    );
    expect(parseState(params, fixture)).toEqual({
      q: "Forest",
      category: "forest-friend",
      author: "deskpet",
      type: "included",
      sort: "tricks",
      page: 2,
    });
    expect(
      parseState(new URLSearchParams("q=&category=nope&author=nope&type=nope&sort=nope"), fixture),
    ).toEqual(DEFAULT_STATE);
  });

  it("clamps bad page values to one", () => {
    for (const page of ["0", "-3", "abc"]) {
      expect(parseState(new URLSearchParams(`page=${page}`), fixture).page).toBe(1);
    }
    expect(parseState(new URLSearchParams("page=2"), fixture).page).toBe(2);
  });
});

describe("stateParams", () => {
  it("omits defaults and keeps insertion order", () => {
    expect(stateParams(DEFAULT_STATE).toString()).toBe("");
    const full: CatalogueState = {
      q: "Forest",
      category: "forest-friend",
      author: "deskpet",
      type: "included",
      sort: "tricks",
      page: 2,
    };
    expect(stateParams(full).toString()).toBe(
      "q=Forest&category=forest-friend&author=deskpet&type=included&sort=tricks&page=2",
    );
    expect(stateParams({ ...full, sort: "az", page: 1 }).toString()).toBe(
      "q=Forest&category=forest-friend&author=deskpet&type=included",
    );
  });
});

describe("pagePath", () => {
  it("leaves page one on the base path and paginates the rest", () => {
    expect(pagePath("/pets/", 1)).toBe("/pets/");
    expect(pagePath("/pets/", 3)).toBe("/pets/page/3/");
  });
});

describe("real data", () => {
  it("paginates the shipped catalogue and builds breadcrumb schema", () => {
    const records = listingRecords();
    const all = records.filter((record) => record.scope.kind === "all");
    expect(all[0].pageCount).toBe(2);

    const hero = records.find(
      (record) => record.scope.kind === "category" && record.scope.id === "hero",
    );
    expect(hero?.pageCount).toBe(1);

    const deskpet = records.find(
      (record) => record.scope.kind === "author" && record.scope.slug === "deskpet",
    );
    expect(deskpet?.pageCount).toBe(2);

    const schema = breadcrumbSchema([
      { label: "All pets", path: "/pets/" },
      { label: "Page 2", path: "/pets/page/2/" },
    ]);
    const items = schema.itemListElement as { position: number; item: string }[];
    expect(items.map((item) => item.position)).toEqual([1, 2]);
    expect(items.map((item) => item.item)).toEqual([
      "https://deskpet.pages.dev/pets/",
      "https://deskpet.pages.dev/pets/page/2/",
    ]);
  });
});

describe("CATEGORY_ID", () => {
  it("accepts slugs and rejects path or non-slug input", () => {
    expect(CATEGORY_ID.test("forest-friend")).toBe(true);
    expect(CATEGORY_ID.test("../etc")).toBe(false);
    expect(CATEGORY_ID.test("Bad_Category")).toBe(false);
    expect(CATEGORY_ID.test("")).toBe(false);
  });
});
