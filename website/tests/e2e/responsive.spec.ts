import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "@playwright/test";
import {
  PAGES,
  expectNoHorizontalOverflow,
  expectTheme,
  gotoThemed,
  loadAllImages,
} from "../helpers";

const themes = ["light", "dark"] as const;
const widths = [320, 375, 768, 1024, 1440, 1920].map((width) => ({ width, height: 900 }));

mkdirSync(join(process.cwd(), "qa", "screenshots"), { recursive: true });

test("author cards contain their sprite strip at 768px", async ({ page }) => {
  await page.setViewportSize({ width: 768, height: 900 });
  for (const theme of themes) {
    await gotoThemed(page, "/authors/", theme);
    await loadAllImages(page);
    const card = page.locator("a.author-card").first();
    const cardBox = await card.boundingBox();
    if (!cardBox) throw new Error("missing author card");
    const images = card.locator(".author-pets img");
    await expect(images).toHaveCount(4);
    for (let index = 0; index < 4; index++) {
      const box = await images.nth(index).boundingBox();
      if (!box) throw new Error("missing author sprite");
      expect(box.x).toBeGreaterThanOrEqual(cardBox.x - 1);
      expect(box.y).toBeGreaterThanOrEqual(cardBox.y - 1);
      expect(box.x + box.width).toBeLessThanOrEqual(cardBox.x + cardBox.width + 1);
      expect(box.y + box.height).toBeLessThanOrEqual(cardBox.y + cardBox.height + 1);
    }
  }
});

for (const theme of themes) {
  test.describe(`responsive + screenshots (${theme})`, () => {
    for (const target of PAGES) {
      test(`${target.name} fits and screenshots at every width`, async ({ page }) => {
        test.setTimeout(240_000);
        for (const viewport of widths) {
          await page.setViewportSize(viewport);
          await gotoThemed(page, target.path, theme, { reducedMotion: true });
          await expectTheme(page, theme);
          await loadAllImages(page);
          await expectNoHorizontalOverflow(page);
          await page.screenshot({
            path: join(
              process.cwd(),
              "qa",
              "screenshots",
              `${target.name}-${viewport.width}-${theme}.png`,
            ),
            fullPage: true,
          });
        }
      });
    }

    test("catalogue mobile filters open fits and screenshots at 375px", async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 900 });
      await gotoThemed(page, "/pets/", theme, { reducedMotion: true });
      await expectTheme(page, theme);
      await expect(page.locator("[data-controls]")).toHaveAttribute("data-ready", "true");

      await page.locator("[data-filters-toggle]").click();
      await expect(page.locator("[data-author]")).toBeVisible();
      await loadAllImages(page);
      await expectNoHorizontalOverflow(page);
      await page.screenshot({
        path: join(process.cwd(), "qa", "screenshots", `catalogue-filters-open-375-${theme}.png`),
        fullPage: true,
      });
    });

    test("catalogue long-query chip fits and screenshots at 320", async ({ page }) => {
      await page.setViewportSize({ width: 320, height: 900 });
      await gotoThemed(page, `/pets/?q=${"x".repeat(80)}`, theme, { reducedMotion: true });
      await expect(page.locator("[data-controls]")).toHaveAttribute("data-ready", "true");
      await loadAllImages(page);
      await expectNoHorizontalOverflow(page);
      await page.screenshot({
        path: join(process.cwd(), "qa", "screenshots", `catalogue-long-query-320-${theme}.png`),
        fullPage: true,
      });
    });

    test("catalogue filtered state fits and screenshots at 375 and 1440", async ({ page }) => {
      for (const width of [375, 1440]) {
        await page.setViewportSize({ width, height: 900 });
        await gotoThemed(page, "/pets/?author=deskpet", theme, { reducedMotion: true });
        await expectTheme(page, theme);
        await expect(page.locator("[data-controls]")).toHaveAttribute("data-ready", "true");
        await loadAllImages(page);
        await expectNoHorizontalOverflow(page);
        await page.screenshot({
          path: join(
            process.cwd(),
            "qa",
            "screenshots",
            `catalogue-filtered-${width}-${theme}.png`,
          ),
          fullPage: true,
        });
      }
    });
  });
}
