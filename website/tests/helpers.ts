import { expect, type Page } from "@playwright/test";

export type Theme = "light" | "dark";

export const PAGES = [
  { name: "home", path: "/" },
  { name: "pet-builtin", path: "/pets/mochi/" },
  { name: "pet-addon", path: "/pets/ember/" },
  { name: "contribute", path: "/contribute/" },
  { name: "support", path: "/support/" },
  { name: "not-found", path: "/this-page-does-not-exist/" },
  { name: "catalogue", path: "/pets/" },
  { name: "catalogue-page-2", path: "/pets/page/2/" },
  { name: "category-hero", path: "/pets/category/hero/" },
  { name: "authors", path: "/authors/" },
  { name: "author-deskpet", path: "/authors/deskpet/" },
  { name: "author-deskpet-page-2", path: "/authors/deskpet/page/2/" },
] as const;

export async function gotoThemed(
  page: Page,
  path: string,
  theme: Theme,
  { reducedMotion = false }: { reducedMotion?: boolean } = {},
) {
  await page.emulateMedia({
    colorScheme: theme,
    reducedMotion: reducedMotion ? "reduce" : "no-preference",
  });
  await page.addInitScript((value) => {
    try {
      localStorage.setItem("deskpet-theme", value);
    } catch {
      /* ignore */
    }
  }, theme);
  return page.goto(path, { waitUntil: "load" });
}

export async function expectTheme(page: Page, theme: Theme) {
  const html = page.locator("html");
  if (theme === "dark") {
    await expect(html).toHaveClass(/dark/);
  } else {
    await expect(html).not.toHaveClass(/dark/);
  }
}

/** Load every image (including lazy ones) so screenshots are complete. */
export async function loadAllImages(page: Page) {
  await page.evaluate(async () => {
    for (const image of document.querySelectorAll('img[loading="lazy"]')) {
      (image as HTMLImageElement).loading = "eager";
    }
    await Promise.all(
      Array.from(document.images)
        .filter((image) => !image.complete)
        .map(
          (image) =>
            new Promise<void>((resolve) => {
              image.addEventListener("load", () => resolve(), { once: true });
              image.addEventListener("error", () => resolve(), { once: true });
            }),
        ),
    );
  });
  await page.waitForTimeout(150);
}

export async function expectNoHorizontalOverflow(page: Page) {
  const report = await page.evaluate(() => {
    const viewport = document.documentElement.clientWidth;
    const offenders: string[] = [];
    for (const element of document.body.querySelectorAll<HTMLElement>("*")) {
      if (element.classList.contains("sr-only")) continue;
      const rect = element.getBoundingClientRect();
      if (rect.width === 0 && rect.height === 0) continue;
      if (rect.right > viewport + 1 || rect.left < -2) {
        offenders.push(
          `${element.tagName.toLowerCase()}.${String(element.className).slice(0, 70)} left=${Math.round(rect.left)} right=${Math.round(rect.right)}`,
        );
      }
    }
    return {
      viewport,
      documentWidth: document.documentElement.scrollWidth,
      bodyWidth: document.body.scrollWidth,
      offenders: offenders.slice(0, 8),
    };
  });

  expect(
    report.documentWidth,
    `document scrollWidth ${report.documentWidth} > ${report.viewport}`,
  ).toBeLessThanOrEqual(report.viewport + 1);
  expect(
    report.bodyWidth,
    `body scrollWidth ${report.bodyWidth} > ${report.viewport}`,
  ).toBeLessThanOrEqual(report.viewport + 1);
  expect(report.offenders, `elements outside the viewport: ${report.offenders.join("; ")}`).toEqual(
    [],
  );
}
