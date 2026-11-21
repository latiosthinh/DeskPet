import { readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@playwright/test";

const pets = JSON.parse(
  readFileSync(join(process.cwd(), "src", "data", "pets.generated.json"), "utf8"),
).pets as { id: string }[];

test("sitemap.xml is valid XML with every page location", async ({ request }) => {
  const response = await request.get("/sitemap.xml");
  expect(response.status()).toBe(200);
  expect(response.headers()["content-type"]).toContain("xml");

  const body = await response.text();
  expect(body).toContain('<?xml version="1.0" encoding="UTF-8"?>');
  expect(body).toContain('<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">');
  expect(body).toContain("<loc>https://deskpet.pages.dev/</loc>");
  expect(body).toContain("<loc>https://deskpet.pages.dev/contribute/</loc>");
  for (const pet of pets) {
    expect(body).toContain(`<loc>https://deskpet.pages.dev/pets/${pet.id}/</loc>`);
  }
  expect(body.match(/<loc>/g)).toHaveLength(pets.length + 2);
});

test("robots.txt points search engines at sitemap.xml", async ({ request }) => {
  const response = await request.get("/robots.txt");
  expect(response.status()).toBe(200);
  expect(await response.text()).toContain("Sitemap: https://deskpet.pages.dev/sitemap.xml");
});
