import { mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "@playwright/test";
import { expectNoHorizontalOverflow, expectTheme, gotoThemed, loadAllImages } from "../helpers";

const pets = JSON.parse(
  readFileSync(join(process.cwd(), "src", "data", "pets.generated.json"), "utf8"),
).pets as { id: string }[];

const OUT = join(process.cwd(), "qa", "pet-pages");
mkdirSync(OUT, { recursive: true });

for (const theme of ["light", "dark"] as const) {
  for (const width of [375, 1440]) {
    test(`every pet page fits and screenshots at ${width}px (${theme})`, async ({ page }) => {
      test.setTimeout(180_000);
      await page.setViewportSize({ width, height: 900 });
      for (const pet of pets) {
        await gotoThemed(page, `/pets/${pet.id}/`, theme, { reducedMotion: true });
        await expectTheme(page, theme);
        await loadAllImages(page);
        await expectNoHorizontalOverflow(page);
        await page.screenshot({
          path: join(OUT, `${pet.id}-${width}-${theme}.png`),
          fullPage: true,
        });
      }
    });
  }
}
