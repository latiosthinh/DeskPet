import { readFileSync } from "node:fs";
import { registerHooks } from "node:module";
import { join } from "node:path";
import { expect, test } from "@playwright/test";

// site.ts imports pets.generated.json without an import attribute, which Node's
// ESM loader rejects. Add the attribute while loading that one module so the
// test can reuse the real authorSlug implementation.
registerHooks({
  load(url, context, nextLoad) {
    const result = nextLoad(url, context);
    if (url.endsWith("/src/lib/site.ts")) {
      return {
        ...result,
        source: String(result.source).replace(
          '"../data/pets.generated.json"',
          '"../data/pets.generated.json" with { type: "json" }',
        ),
      };
    }
    return result;
  },
});

const { authorSlug } = await import("../../src/lib/site.ts");

interface PetRecord {
  id: string;
  name: string;
  category: string;
  author: string;
}

const pets = JSON.parse(
  readFileSync(join(process.cwd(), "src", "data", "pets.generated.json"), "utf8"),
).pets as PetRecord[];

const ORIGIN = "https://deskpet-vn.pages.dev";
const PAGE_SIZE = 12;

const absolute = (path: string) => `${ORIGIN}${path}`;

function chunk<T>(items: T[], size: number): T[][] {
  const chunks: T[][] = [];
  for (let index = 0; index < items.length; index += size) {
    chunks.push(items.slice(index, index + size));
  }
  return chunks.length ? chunks : [[]];
}

const sortedPets = [...pets].sort(
  (a, b) => a.name.localeCompare(b.name) || a.id.localeCompare(b.id),
);

const cataloguePages = chunk(sortedPets, PAGE_SIZE).map((_, index) =>
  index === 0 ? "/pets/" : `/pets/page/${index + 1}/`,
);

function groupBy<T>(items: T[], key: (item: T) => string): Map<string, T[]> {
  const groups = new Map<string, T[]>();
  for (const item of items) {
    const value = key(item);
    const group = groups.get(value);
    if (group) group.push(item);
    else groups.set(value, [item]);
  }
  return groups;
}

function pagedPaths(basePath: string, items: PetRecord[]): string[] {
  return chunk(items, PAGE_SIZE).map((_, index) =>
    index === 0 ? basePath : `${basePath}page/${index + 1}/`,
  );
}

const listingUrls = new Set<string>(["/authors/", ...cataloguePages]);

for (const [categoryId, categoryPets] of groupBy(pets, (pet) => pet.category)) {
  for (const path of pagedPaths(`/pets/category/${categoryId}/`, categoryPets)) {
    listingUrls.add(path);
  }
}

for (const [author, authorPets] of groupBy(pets, (pet) => pet.author)) {
  for (const path of pagedPaths(`/authors/${authorSlug(author)}/`, authorPets)) {
    listingUrls.add(path);
  }
}

const expectedSitemap = [
  "/",
  "/contribute/",
  "/support/",
  ...pets.map((pet) => `/pets/${pet.id}/`),
  ...listingUrls,
];

test("sitemap.xml is valid XML with every page location", async ({ request }) => {
  const response = await request.get("/sitemap.xml");
  expect(response.status()).toBe(200);
  expect(response.headers()["content-type"]).toContain("xml");

  const body = await response.text();
  expect(body).toContain('<?xml version="1.0" encoding="UTF-8"?>');
  expect(body).toContain('<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">');

  const locations = [...body.matchAll(/<loc>([^<]+)<\/loc>/g)].map((match) => match[1]);
  expect(locations.sort()).toEqual(expectedSitemap.map(absolute).sort());
});

test("listing pages have canonical URLs, descriptions and unique titles", async ({ request }) => {
  const titles = new Set<string>();
  const urls = [...listingUrls, "/", `/pets/${pets[0].id}/`];

  for (const url of urls) {
    const response = await request.get(url);
    expect(response.status(), url).toBe(200);
    const html = await response.text();

    const title = html.match(/<title>([\s\S]*?)<\/title>/)?.[1]?.trim();
    const description = html.match(/<meta name="description" content="([^"]*)"/)?.[1]?.trim();
    const canonical = html.match(/<link[^>]*rel="canonical"[^>]*href="([^"]+)"/)?.[1];

    expect(title, `${url} title`).toBeTruthy();
    expect(description, `${url} description`).toBeTruthy();
    expect(canonical, `${url} canonical`).toBe(absolute(url));
    expect(html.toLowerCase(), `${url} must not be noindex`).not.toContain("noindex");

    titles.add(title!);
  }

  expect(titles.size).toBe(urls.length);
});

test("pet routes and static assets keep working", async ({ request }) => {
  const pet = await request.get("/pets/mochi/");
  expect(pet.status()).toBe(200);
  expect(await pet.text()).toContain("<title>Mochi · DeskPet</title>");

  const image = await request.get("/pets/mochi/gallery.png");
  expect(image.status()).toBe(200);
  expect(image.headers()["content-type"]).toContain("image");

  const missing = await request.get("/pets/this-pet-does-not-exist/");
  expect(missing.status()).toBe(404);
});

test("robots.txt points search engines at sitemap.xml", async ({ request }) => {
  const response = await request.get("/robots.txt");
  expect(response.status()).toBe(200);
  expect(await response.text()).toContain("Sitemap: https://deskpet-vn.pages.dev/sitemap.xml");
});
