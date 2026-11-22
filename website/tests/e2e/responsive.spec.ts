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
