import { expect, test, type Page } from "@playwright/test";
import { gotoThemed } from "../helpers";

const CARD_COUNT = 15;

async function ready(page: Page) {
  await expect(page.locator("[data-controls]")).toHaveAttribute("data-ready", "true");
}

function cards(page: Page) {
  return page.locator("#catalogue-grid a.card");
}

async function openCatalogue(page: Page) {
  await gotoThemed(page, "/pets/", "light");
  await ready(page);
}

test("the static index exposes only the compact catalogue fields", async ({ request }) => {
  const response = await request.get("/pets/catalogue.json");
  expect(response.status()).toBe(200);
  expect(response.headers()["content-type"]).toContain("application/json");
  const body = (await response.json()) as {
    pets: Record<string, unknown>[];
  };
  expect(body.pets).toHaveLength(CARD_COUNT);
  expect(body.pets[0].id).toBe("mochi");
  for (const pet of body.pets) {
    expect(pet.authorSlug).toBe("deskpet");
    const gallery = pet.gallery as { src: string };
    expect(gallery.src.startsWith("/pets/")).toBe(true);
    expect(pet).not.toHaveProperty("hero");
    expect(pet).not.toHaveProperty("tricks");
  }
});

test("search finds a page-two pet and survives reload and history", async ({ page }) => {
  await openCatalogue(page);
  await page.locator("input[data-q]").fill("seedling");
  await expect(cards(page)).toHaveCount(1);
  await expect(cards(page).first()).toHaveAttribute("href", "/pets/sprout/");
  await expect(page.locator("[data-count]")).toHaveText("1 pet");
  await expect.poll(() => page.url()).toContain("q=seedling");

  await page.reload();
  await ready(page);
  await expect(cards(page)).toHaveCount(1);
  await expect(cards(page).first()).toHaveAttribute("href", "/pets/sprout/");

  await page.goBack();
  await expect(cards(page)).toHaveCount(12);
});

test("keystrokes during one burst coalesce into a single history entry", async ({ page }) => {
  await openCatalogue(page);
  const input = page.locator("input[data-q]");
  await input.pressSequentially("moss", { delay: 60 });
  await expect.poll(() => page.url()).toContain("q=moss");

  await page.goBack();
  await expect(page).toHaveURL(/\/pets\/$/);
  await expect(cards(page)).toHaveCount(12);
});

test("filters, type and sorts reflect in the URL without defaults", async ({ page }) => {
  await openCatalogue(page);
  await page.selectOption("[data-category]", "hero");
  await expect(cards(page)).toHaveCount(8);
  await expect.poll(() => page.url()).toContain("category=hero");

  await openCatalogue(page);
  await page.selectOption("[data-author]", "deskpet");
  await expect(page.locator("[data-count]")).toHaveText("15 pets");
  await expect.poll(() => page.url()).toContain("author=deskpet");

  await openCatalogue(page);
  await page.selectOption("[data-type]", "included");
  await expect(cards(page)).toHaveCount(3);
  await expect.poll(() => page.url()).toContain("type=included");

  await openCatalogue(page);
  await page.selectOption("[data-type]", "addon");
  await expect(cards(page)).toHaveCount(12);

  await openCatalogue(page);
  await page.selectOption("[data-category]", "hero");
  await page.selectOption("[data-type]", "included");
  await expect(cards(page)).toHaveCount(1);
  await expect.poll(() => page.url()).toContain("category=hero");
  await expect.poll(() => page.url()).toContain("type=included");

  await openCatalogue(page);
  await page.selectOption("[data-sort]", "newest");
  await expect(cards(page).first()).toHaveAttribute("href", "/pets/witcher/");
  await expect.poll(() => page.url()).toContain("sort=newest");

  await openCatalogue(page);
  await page.selectOption("[data-sort]", "tricks");
  await expect(cards(page).first()).toHaveAttribute("href", "/pets/witcher/");

  await openCatalogue(page);
  await page.selectOption("[data-sort]", "tricks");
  await page.selectOption("[data-sort]", "az");
  await expect(cards(page).first()).toHaveAttribute("href", "/pets/ember/");
  await expect.poll(() => page.url()).not.toContain("sort=");
});

test("a direct query URL hydrates controls, chips and results", async ({ page }) => {
  await gotoThemed(page, "/pets/?q=fox&category=critter&sort=tricks", "light");
  await ready(page);
  await expect(cards(page)).toHaveCount(1);
  await expect(cards(page).first()).toHaveAttribute("href", "/pets/ember/");
  await expect(page.locator("input[data-q]")).toHaveValue("fox");
  await expect(page.locator("[data-category]")).toHaveValue("critter");
  await expect(page.locator("[data-sort]")).toHaveValue("tricks");
  await expect(page.locator("[data-chip-list] li")).toHaveCount(2);

  await page.reload();
  await ready(page);
  await expect(cards(page)).toHaveCount(1);
  await expect(cards(page).first()).toHaveAttribute("href", "/pets/ember/");
  await expect(page.locator("[data-chip-list] li")).toHaveCount(2);
});

test("chips remove filters and Clear all restores the catalogue", async ({ page }) => {
  await openCatalogue(page);
  await page.selectOption("[data-category]", "hero");
  await page.selectOption("[data-type]", "included");
  await expect(page.locator("[data-chip-list] li")).toHaveCount(2);
  await expect(cards(page)).toHaveCount(1);
  await expect(cards(page).first()).toHaveAttribute("href", "/pets/ninja/");

  const authorChip = page
    .locator("[data-chip-list] li")
    .filter({ hasText: "Heroes" })
    .locator("button");
  await authorChip.click();
  await expect(cards(page)).toHaveCount(3);
  await expect.poll(() => page.url()).not.toContain("category=");
  expect(await page.evaluate(() => document.activeElement === document.body)).toBe(false);

  await page.locator("[data-clear]").click();
  await expect(cards(page)).toHaveCount(12);
  await expect(page).toHaveURL(/\/pets\/$/);
  await expect(page.locator("[data-chips-row]")).toBeHidden();
});

test("an empty result offers a reset back to the full shelf", async ({ page }) => {
  await gotoThemed(page, "/pets/?q=zzz-no-match", "light");
  await ready(page);
  await expect(cards(page)).toHaveCount(0);
  await expect(page.locator("[data-empty]")).toBeVisible();

  await page.locator("[data-reset]").click();
  await expect(cards(page)).toHaveCount(12);
  await expect(page).toHaveURL(/\/pets\/$/);
  await expect(page.locator("[data-empty]")).toBeHidden();
});

test("back and forward restore search and sort state", async ({ page }) => {
  await openCatalogue(page);
  await page.locator("input[data-q]").fill("fox");
  await expect.poll(() => page.url()).toContain("q=fox");
  await page.selectOption("[data-sort]", "tricks");
  await expect.poll(() => page.url()).toContain("sort=tricks");

  await page.goBack();
  await expect.poll(() => page.url()).toContain("q=fox");
  await expect.poll(() => page.url()).not.toContain("sort=");
  await expect(cards(page)).toHaveCount(1);
  await expect(cards(page).first()).toHaveAttribute("href", "/pets/ember/");

  await page.goForward();
  await expect.poll(() => page.url()).toContain("sort=tricks");
});

test("enhanced pagination keeps filters in the query string", async ({ page }) => {
  await openCatalogue(page);
  await page.selectOption("[data-author]", "deskpet");
  await expect(page.locator("[data-pagination] a[data-page='2']:not([rel])")).toBeVisible();
  await page.locator("[data-pagination] a[data-page='2']:not([rel])").click();

  await expect.poll(() => page.url()).toContain("author=deskpet");
  await expect.poll(() => page.url()).toContain("page=2");
  await expect(cards(page)).toHaveCount(3);
  await expect(page.locator("[data-pagination] a[aria-current='page']")).toHaveText("2");

  await page.goBack();
  await expect(cards(page)).toHaveCount(12);
});

test("unfiltered pagination stays on static paths", async ({ page }) => {
  await openCatalogue(page);
  await page.locator("[data-pagination] a[rel='next']").click();
  await expect(page).toHaveURL(/\/pets\/page\/2\/$/);
  await expect(cards(page)).toHaveCount(3);
  await expect(page.locator("[data-pagination] a[aria-current='page']")).toHaveText("2");
  await expect(page.locator("[data-pagination] a[rel='prev']")).toHaveAttribute("href", "/pets/");
});

test("the mobile Filters disclosure opens the extra selects", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 900 });
  await openCatalogue(page);
  const toggle = page.locator("[data-filters-toggle]");
  await expect(toggle).toBeVisible();
  await expect(toggle).toHaveAttribute("aria-expanded", "false");
  await expect(page.locator("[data-filters-panel]")).toBeHidden();

  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator("[data-category]")).toBeVisible();
  await page.selectOption("[data-category]", "legend");
  await expect(cards(page)).toHaveCount(2);
});

test("a failed index leaves the static catalogue untouched", async ({ page }) => {
  await page.route("**/pets/catalogue.json", (route) => route.abort());
  await gotoThemed(page, "/pets/", "light");
  await expect(page.locator("[data-controls]")).toBeHidden();
  await expect(cards(page)).toHaveCount(12);

  await page.locator("[data-pagination] a[rel='next']").click();
  await expect(page).toHaveURL(/\/pets\/page\/2\/$/);
  await expect(cards(page)).toHaveCount(3);
});

test.describe("without JavaScript", () => {
  test.use({ javaScriptEnabled: false });

  test("static pagination and category routes still navigate", async ({ page }) => {
    await page.goto("/pets/", { waitUntil: "load" });
    await expect(cards(page)).toHaveCount(12);
    await expect(page.locator("[data-controls]")).toBeHidden();

    const numbered = page.locator("[data-pagination] a").filter({ hasText: /^[0-9]+$/ });
    await expect(numbered).toHaveCount(2);
    expect(await numbered.nth(0).getAttribute("href")).toBe("/pets/");
    expect(await numbered.nth(1).getAttribute("href")).toBe("/pets/page/2/");
    await expect(page.locator("[data-pagination] a[rel='prev']")).toHaveCount(0);

    await page.locator("[data-pagination] a[rel='next']").click();
    await expect(page).toHaveURL(/\/pets\/page\/2\/$/);
    await expect(cards(page)).toHaveCount(3);
    await expect(page.locator("[data-pagination] a[aria-current='page']")).toHaveText("2");
    await expect(page.locator("[data-pagination] a[rel='next']")).toHaveCount(0);

    await page.goto("/pets/", { waitUntil: "load" });
    await page.locator("a[href='/pets/category/hero/']").click();
    await expect(page).toHaveURL(/\/pets\/category\/hero\/$/);
    await expect(page.locator(".gallery-grid a.card")).toHaveCount(8);
    const crumbs = page.locator('nav[aria-label="Breadcrumb"]');
    await expect(crumbs).toContainText("All pets");
    await expect(crumbs).toContainText("Heroes");
    const jsonLd = await page.locator('script[type="application/ld+json"]').first().textContent();
    expect(jsonLd).toContain("BreadcrumbList");
  });

  test("authors and pet detail pages keep their static routes", async ({ page }) => {
    await page.goto("/authors/", { waitUntil: "load" });
    const author = page.locator("a.author-card[href='/authors/deskpet/']");
    await expect(author).toContainText("15 pets");
    await author.click();
    await expect(page).toHaveURL(/\/authors\/deskpet\/$/);
    await expect(page.locator(".gallery-grid a.card")).toHaveCount(12);

    await page.locator("[data-pagination] a[rel='next']").click();
    await expect(page).toHaveURL(/\/authors\/deskpet\/page\/2\/$/);
    await expect(page.locator(".gallery-grid a.card")).toHaveCount(3);
    const crumbs = page.locator('nav[aria-label="Breadcrumb"]');
    await expect(crumbs).toContainText("Authors");
    await expect(crumbs).toContainText("DeskPet");

    await page.goto("/pets/grumbo/", { waitUntil: "load" });
    const petCrumbs = page.locator('nav[aria-label="Breadcrumb"]');
    await expect(petCrumbs).toContainText("All pets");
    await expect(petCrumbs).toContainText("Heroes");
    await expect(petCrumbs).toContainText("Grumbo the warrior");
    await expect(page.locator('.pet-meta a[href="/authors/deskpet/"]')).toHaveCount(1);
  });
});

test("enhanced cards keep one card geometry and one status treatment", async ({ page }) => {
  await openCatalogue(page);
  await page.selectOption("[data-type]", "addon");
  await expect(cards(page)).toHaveCount(12);

  const report = await cards(page).evaluateAll((elements) =>
    elements.map((card) => {
      const image = card.querySelectorAll(".media-frame.frame-gallery img");
      const title = card.querySelector(".card-title")?.textContent?.trim() ?? "";
      const meta = card.querySelector(".card-foot .card-meta")?.textContent?.trim() ?? "";
      const badge = card.querySelector<HTMLElement>("[data-badge]");
      const addon = card.querySelector<HTMLElement>("[data-addon]");
      const visibleBadge = Boolean(badge && !badge.hidden);
      const visibleAddon = Boolean(addon && !addon.hidden);
      return {
        images: image.length,
        width: image[0]?.getAttribute("width") ?? null,
        height: image[0]?.getAttribute("height") ?? null,
        title,
        meta,
        statuses: Number(visibleBadge) + Number(visibleAddon),
        addon: visibleAddon,
      };
    }),
  );

  for (const card of report) {
    expect(card.images).toBe(1);
    expect(card.width).not.toBeNull();
    expect(card.height).not.toBeNull();
    expect(card.title.length).toBeGreaterThan(0);
    expect(card.meta).toMatch(/^\d+ trick(s)?$/);
    expect(card.statuses).toBe(1);
    expect(card.addon).toBe(true);
  }

  await page.selectOption("[data-type]", "included");
  await expect(cards(page)).toHaveCount(3);
  const included = await cards(page).evaluateAll((elements) =>
    elements.map((card) => {
      const badge = card.querySelector<HTMLElement>("[data-badge]");
      const addon = card.querySelector<HTMLElement>("[data-addon]");
      return {
        badge: Boolean(badge && !badge.hidden),
        addon: Boolean(addon && !addon.hidden),
      };
    }),
  );
  for (const card of included) {
    expect(card.badge).toBe(true);
    expect(card.addon).toBe(false);
  }
});
