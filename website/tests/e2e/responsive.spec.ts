import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "@playwright/test";
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
  });
}
