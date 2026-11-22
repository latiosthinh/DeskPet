import { expect, test, type Page } from "@playwright/test";
import { CATALOGUE_DESCRIPTION } from "../../src/lib/catalogue-core";
import { expectNoHorizontalOverflow, gotoThemed } from "../helpers";

const CARD_COUNT = 15;

async function jsonLd(page: Page) {
  const text = await page.locator('script[type="application/ld+json"]').textContent();
  return JSON.parse(text ?? "{}") as {
    itemListElement: { position: number; item: string }[];
  };
}

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

async function clickCategory(page: Page, id: string) {
  await page.locator(`[data-category-nav] a[data-cat="${id}"]`).click();
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
  await clickCategory(page, "hero");
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
  await clickCategory(page, "hero");
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
  await expect(page.locator('[data-category-nav] a[data-cat="critter"]')).toHaveAttribute(
    "aria-current",
    "page",
  );
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
  await clickCategory(page, "hero");
  await page.selectOption("[data-type]", "included");
  await expect(page.locator("[data-chip-list] li")).toHaveCount(2);
  await expect(cards(page)).toHaveCount(1);
  await expect(cards(page).first()).toHaveAttribute("href", "/pets/ninja/");

  const categoryChip = page
    .locator("[data-chip-list] li")
    .filter({ hasText: "Heroes" })
    .locator("button");
  await categoryChip.click();
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

  const toggleBox = await toggle.boundingBox();
  const sortBox = await page.locator("select[data-sort]").boundingBox();
  if (!toggleBox || !sortBox) throw new Error("missing toolbar boxes");
  expect(Math.abs(toggleBox.y - sortBox.y)).toBeLessThanOrEqual(1);
  expect(Math.abs(toggleBox.height - sortBox.height)).toBeLessThanOrEqual(1);

  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator("[data-author]")).toBeVisible();
  await expect(page.locator("[data-type]")).toBeVisible();
  await page.selectOption("[data-type]", "included");
  await expect(cards(page)).toHaveCount(3);
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

test("history restores static pages with breadcrumb, title and canonical", async ({ page }) => {
  await gotoThemed(page, "/pets/page/2/", "light");
  await ready(page);
  await expect(cards(page)).toHaveCount(3);
  await expect(page.locator("[data-pagination] a[aria-current='page']")).toHaveText("2");
  const crumbs = page.locator('nav[aria-label="Breadcrumb"]');
  await expect(crumbs).toBeVisible();
  await expect(crumbs).toContainText("Page 2");

  await page.selectOption("[data-author]", "deskpet");
  await expect(cards(page)).toHaveCount(12);
  await expect(page).toHaveURL(/\/pets\/\?author=deskpet$/);
  await expect(crumbs).toBeHidden();
  await expect(page).toHaveTitle("Pets · DeskPet");
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute(
    "href",
    "https://deskpet-vn.pages.dev/pets/?author=deskpet",
  );
  await expect(page.locator('meta[name="description"]')).toHaveAttribute(
    "content",
    CATALOGUE_DESCRIPTION,
  );
  const filteredLd = await jsonLd(page);
  expect(filteredLd.itemListElement).toHaveLength(1);
  expect(filteredLd.itemListElement[0].item).toBe("https://deskpet-vn.pages.dev/pets/");

  await page.goBack();
  await expect(page).toHaveURL(/\/pets\/page\/2\/$/);
  await expect(cards(page)).toHaveCount(3);
  await expect(page.locator("[data-pagination] a[aria-current='page']")).toHaveText("2");
  await expect(crumbs).toBeVisible();
  await expect(crumbs).toContainText("Page 2");
  await expect(page).toHaveTitle("Pets · Page 2 · DeskPet");
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute(
    "href",
    "https://deskpet-vn.pages.dev/pets/page/2/",
  );
  await expect(page.locator('meta[name="description"]')).toHaveAttribute(
    "content",
    `${CATALOGUE_DESCRIPTION} Page 2 of 2.`,
  );
  const pageLd = await jsonLd(page);
  expect(pageLd.itemListElement).toHaveLength(2);
  expect(pageLd.itemListElement.map((entry) => entry.position)).toEqual([1, 2]);
  expect(pageLd.itemListElement.map((entry) => entry.item)).toEqual([
    "https://deskpet-vn.pages.dev/pets/",
    "https://deskpet-vn.pages.dev/pets/page/2/",
  ]);

  await page.goForward();
  await expect(cards(page)).toHaveCount(12);
  await expect(crumbs).toBeHidden();
  await expect(page).toHaveURL(/author=deskpet$/);
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute(
    "href",
    "https://deskpet-vn.pages.dev/pets/?author=deskpet",
  );
  await expect(page.locator('meta[name="description"]')).toHaveAttribute(
    "content",
    CATALOGUE_DESCRIPTION,
  );
  const forwardLd = await jsonLd(page);
  expect(forwardLd.itemListElement).toHaveLength(1);
  expect(forwardLd.itemListElement[0].item).toBe("https://deskpet-vn.pages.dev/pets/");
});

test("single-page catalogues still get working controls", async ({ page, request }) => {
  const response = await request.get("/pets/catalogue.json");
  const body = (await response.json()) as { pets: unknown[] };
  await page.route("**/pets/catalogue.json", (route) =>
    route.fulfill({ json: { pets: body.pets.slice(0, 12) } }),
  );

  await gotoThemed(page, "/pets/?page=1", "light");
  await ready(page);
  await expect(cards(page)).toHaveCount(12);
  await expect(page.locator("[data-pagination]")).toBeHidden();

  await page.locator("input[data-q]").fill("mochi");
  await expect(cards(page)).toHaveCount(1);
  await expect(page.locator("[data-pagination]")).toBeHidden();
});

test("client-created chips and pagination links keep their styles", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 900 });
  await gotoThemed(page, "/pets/?author=deskpet", "light");
  await ready(page);

  const chipButton = page.locator("[data-chip-list] li button").first();
  const removeBox = await chipButton.boundingBox();
  expect(removeBox?.width ?? 0).toBeGreaterThanOrEqual(24);
  expect(removeBox?.height ?? 0).toBeGreaterThanOrEqual(24);

  const chipBorder = await page
    .locator("[data-chip-list] li")
    .first()
    .evaluate((element) => getComputedStyle(element).borderTopWidth);
  expect(chipBorder).not.toBe("0px");

  const anchors = page.locator("[data-pagination] a");
  const linkCount = await anchors.count();
  expect(linkCount).toBeGreaterThan(0);
  for (let index = 0; index < linkCount; index++) {
    const box = await anchors.nth(index).boundingBox();
    expect(box?.width ?? 0).toBeGreaterThanOrEqual(24);
    expect(box?.height ?? 0).toBeGreaterThanOrEqual(24);
  }

  const linkBorder = await page
    .locator("[data-pagination] a[data-page='1']")
    .evaluate((element) => getComputedStyle(element).borderTopStyle);
  expect(linkBorder).toBe("solid");
});

test("category chips replace the removed category select", async ({ page }) => {
  await openCatalogue(page);
  await expect(page.locator("select[data-category]")).toHaveCount(0);

  const allChip = page.locator('[data-category-nav] a[data-cat=""]');
  const heroChip = page.locator('[data-category-nav] a[data-cat="hero"]');
  await expect(allChip).toHaveAttribute("aria-current", "page");

  await heroChip.click();
  await expect(cards(page)).toHaveCount(8);
  await expect.poll(() => page.url()).toContain("category=hero");
  await expect(heroChip).toHaveAttribute("aria-current", "page");
  await expect(allChip).not.toHaveAttribute("aria-current", "page");

  await allChip.click();
  await expect(cards(page)).toHaveCount(12);
  await expect(page).toHaveURL(/\/pets\/$/);
  await expect(allChip).toHaveAttribute("aria-current", "page");
});

test("the mobile Filters toggle counts the filters inside its panel", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 900 });
  await openCatalogue(page);
  const toggle = page.locator("[data-filters-toggle]");
  await toggle.click();

  await page.selectOption("[data-author]", "deskpet");
  await page.selectOption("[data-type]", "included");
  await expect(page.locator("[data-filter-count]")).toHaveText("2");
  await expect(toggle).toContainText("2");

  await page.selectOption("[data-author]", "");
  await expect(page.locator("[data-filter-count]")).toHaveText("1");
});

test("pagination shows a page status on static and enhanced views", async ({ page }) => {
  await openCatalogue(page);
  await expect(page.locator("[data-pagination]")).toContainText("Page 1 of 2");

  await page.selectOption("[data-author]", "deskpet");
  await expect(page.locator("[data-pagination]")).toContainText("Page 1 of 2");
});

test("toolbar tab order follows the visual order", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await openCatalogue(page);
  await page.locator("input[data-q]").focus();
  await page.keyboard.press("Tab");
  await expect(page.locator("select[data-author]")).toBeFocused();

  await page.setViewportSize({ width: 375, height: 900 });
  await openCatalogue(page);
  await page.locator("input[data-q]").focus();
  await page.keyboard.press("Tab");
  await expect(page.locator("[data-filters-toggle]")).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(page.locator("select[data-sort]")).toBeFocused();
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

test("a long unbroken search chip fits on a 320px viewport", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 900 });
  await gotoThemed(page, `/pets/?q=${"x".repeat(80)}`, "light");
  await ready(page);
  const chips = page.locator("[data-chip-list] li");
  await expect(chips).toHaveCount(1);
  await expectNoHorizontalOverflow(page);

  const chipBox = await chips.first().boundingBox();
  if (!chipBox) throw new Error("missing chip");
  expect(chipBox.x + chipBox.width).toBeLessThanOrEqual(320);

  const removeBox = await chips.first().locator("button").boundingBox();
  if (!removeBox) throw new Error("missing chip remove button");
  expect(removeBox.x + removeBox.width).toBeLessThanOrEqual(320);
  expect(removeBox.width).toBeGreaterThanOrEqual(24);
  expect(removeBox.height).toBeGreaterThanOrEqual(24);
});
