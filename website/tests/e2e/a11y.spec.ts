import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";
import { PAGES, expectTheme, gotoThemed } from "../helpers";

const themes = ["light", "dark"] as const;
const TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22a", "wcag22aa"];

async function expectNoViolations(page: Page) {
  const results = await new AxeBuilder({ page }).withTags(TAGS).analyze();
  const summary = results.violations.map((violation) => ({
    id: violation.id,
    impact: violation.impact,
    help: violation.help,
    nodes: violation.nodes.map((node) => node.target.join(" ")),
  }));
  expect(summary).toEqual([]);
}

async function waitReady(page: Page) {
  await expect(page.locator("[data-controls]")).toHaveAttribute("data-ready", "true");
}

/** Press Tab until the focused element matches `selector`. Returns whether it was reached. */
async function tabTo(page: Page, selector: string, max = 25) {
  for (let index = 0; index < max; index++) {
    await page.keyboard.press("Tab");
    const hit = await page.evaluate(
      (value) => document.activeElement?.matches(value) ?? false,
      selector,
    );
    if (hit) return true;
  }
  return false;
}

for (const theme of themes) {
  test.describe(`axe (${theme})`, () => {
    test.use({ colorScheme: theme });

    for (const target of PAGES) {
      test(`${target.name} has no WCAG 2.2 AA violations`, async ({ page }) => {
        await gotoThemed(page, target.path, theme);
        await expectTheme(page, theme);
        await expectNoViolations(page);
      });
    }

    test("catalogue mobile filter drawer has no violations", async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 900 });
      await gotoThemed(page, "/pets/", theme);
      await waitReady(page);

      const toggle = page.locator("[data-filters-toggle]");
      await toggle.click();
      await expect(toggle).toHaveAttribute("aria-expanded", "true");
      await expect(page.locator("[data-category]")).toBeVisible();

      await expectNoViolations(page);
    });
  });
}

test("catalogue controls are keyboard operable with visible focus", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await gotoThemed(page, "/pets/", "light");
  await waitReady(page);

  expect(await tabTo(page, "input[data-q]")).toBe(true);
  const outlineWidth = await page.evaluate(() => {
    const active = document.activeElement as HTMLElement | null;
    return active ? getComputedStyle(active).outlineWidth : "";
  });
  expect(outlineWidth).not.toBe("0px");

  await page.keyboard.type("fox");
  await expect(page.locator("#catalogue-grid a.card")).toHaveCount(1);

  await page.keyboard.press("Tab");
  expect(
    await page.evaluate(() => document.activeElement?.matches("select[data-sort]") ?? false),
  ).toBe(true);
  const sortBefore = await page.locator("select[data-sort]").inputValue();
  await page.keyboard.press("ArrowDown");
  await expect(page.locator("select[data-sort]")).not.toHaveValue(sortBefore);
  await expect.poll(() => page.url()).toContain("sort=newest");

  await page.locator("select[data-category]").focus();
  await page.keyboard.press("ArrowDown");
  await expect(page.locator("[data-chip-list] li")).toHaveCount(2);

  expect(await tabTo(page, "[data-chip-list] button")).toBe(true);
  await page.keyboard.press("Enter");
  await expect(page.locator("[data-chip-list] li")).toHaveCount(1);
  expect(await page.evaluate(() => document.activeElement === document.body)).toBe(false);
});

test("mobile Filters disclosure is keyboard operable", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 900 });
  await gotoThemed(page, "/pets/", "light");
  await waitReady(page);

  expect(await tabTo(page, "[data-filters-toggle]")).toBe(true);
  await page.keyboard.press("Enter");
  await expect(page.locator("[data-filters-toggle]")).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator("[data-category]")).toBeVisible();

  await page.keyboard.press("Tab");
  expect(
    await page.evaluate(() => document.activeElement?.matches("select[data-category]") ?? false),
  ).toBe(true);
  await page.keyboard.press("ArrowDown");
  await expect.poll(() => page.url()).toContain("category=critter");
  await expect(page.locator("[data-chip-list] li")).toHaveCount(1);
});

const TARGET_PAGES = [
  "/pets/",
  "/pets/page/2/",
  "/pets/category/hero/",
  "/authors/",
  "/authors/deskpet/",
];
const TARGET_VIEWPORTS = [
  { width: 375, height: 900 },
  { width: 1280, height: 900 },
];

test("interactive targets on the new pages meet 24px", async ({ page }) => {
  test.setTimeout(180_000);
  for (const viewport of TARGET_VIEWPORTS) {
    for (const path of TARGET_PAGES) {
      await page.setViewportSize(viewport);
      await gotoThemed(page, path, "light");
      const controls = page.locator("[data-controls]");
      if ((await controls.count()) > 0) {
        await expect(controls).toHaveAttribute("data-ready", "true");
      }

      const boxes = await page.evaluate(() => {
        const selectors = [
          "button",
          "input",
          "select",
          "a[rel]",
          "nav.pagination a",
          ".cat-chip",
          ".nav-link",
          "footer nav a",
        ];
        const seen = new Set<Element>();
        const result: { label: string; width: number; height: number }[] = [];
        for (const selector of selectors) {
          for (const element of document.querySelectorAll(selector)) {
            if (seen.has(element)) continue;
            seen.add(element);
            const rect = element.getBoundingClientRect();
            const style = getComputedStyle(element);
            if (rect.width === 0 || rect.height === 0) continue;
            if (style.visibility === "hidden" || style.display === "none") continue;
            result.push({
              label: `${element.tagName.toLowerCase()}.${String(element.className)}`,
              width: rect.width,
              height: rect.height,
            });
          }
        }
        return result;
      });

      for (const box of boxes) {
        expect(box.width, `${path} ${box.label} width ${box.width} < 24`).toBeGreaterThanOrEqual(
          24,
        );
        expect(box.height, `${path} ${box.label} height ${box.height} < 24`).toBeGreaterThanOrEqual(
          24,
        );
      }
    }
  }
});
