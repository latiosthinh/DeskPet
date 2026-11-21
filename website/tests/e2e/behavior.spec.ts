import { expect, test, type Locator } from "@playwright/test";
import { expectNoHorizontalOverflow, gotoThemed, loadAllImages } from "../helpers";

const RELEASE = "https://github.com/latiosthinh/DeskPet/releases/latest";

test("gallery lists every pet, grouped, and cards navigate", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  await expect(page.locator("h1")).toHaveCount(1);
  await expect(page.locator(".gallery-grid a.card")).toHaveCount(15);
  await expect(page.locator("#g-critter")).toHaveText("Critters");
  await expect(page.locator("#g-hero")).toHaveText("Heroes");
  await expect(page.locator("#g-legend")).toHaveText("Legends");
  await expect(page.locator(".chip", { hasText: "Comes with the app" })).toHaveCount(3);

  await page.locator('a.card[href="/pets/ember/"]').click();
  await expect(page).toHaveURL(/\/pets\/ember\/$/);
  await expect(page.locator("h1")).toHaveText("Ember the fox");
});

test("header navigation and contribute page", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  await page.locator("header").getByRole("link", { name: "Make a pet" }).click();
  await expect(page).toHaveURL(/\/contribute\/$/);
  await expect(page.locator("h1")).toHaveText("Make a pet");
  await expect(
    page
      .locator('a[href="https://github.com/latiosthinh/DeskPet/blob/main/CONTRIBUTING-PETS.md"]')
      .first(),
  ).toBeVisible();
  await expect(page.locator("ol.step-list li")).toHaveCount(3);
});

test("footer link targets meet the 24px minimum", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  const links = page.locator("footer nav a");
  await expect(links).toHaveCount(3);
  for (const link of await links.all()) {
    const box = await link.boundingBox();
    expect(box, "footer link has no layout box").not.toBeNull();
    expect(box!.width).toBeGreaterThanOrEqual(24);
    expect(box!.height).toBeGreaterThanOrEqual(24);
  }
});

test("download link targets: add-on downloads, built-ins do not", async ({ page }) => {
  await gotoThemed(page, "/pets/ember/", "light");
  const download = page.locator('a[href="/download/ember"]');
  await expect(download).toBeVisible();
  await expect(download).toHaveAttribute("download", "ember.deskpet");
  await expect(page.getByText("version 1.0.0").first()).toBeVisible();

  await gotoThemed(page, "/pets/mochi/", "light");
  await expect(page.getByText("Comes with the app").first()).toBeVisible();
  await expect(page.locator('a[href="/download/mochi"]')).toHaveCount(0);
});

test("both OS downloads are present, labelled, and link to the latest release", async ({
  page,
}) => {
  await gotoThemed(page, "/", "light");
  const mac = page.locator('a[data-os-link="mac"]');
  const win = page.locator('a[data-os-link="windows"]');
  await expect(mac).toBeVisible();
  await expect(win).toBeVisible();
  await expect(mac).toHaveAttribute("href", RELEASE);
  await expect(win).toHaveAttribute("href", RELEASE);
  await expect(mac).toContainText(".dmg");
  await expect(win).toContainText(".zip");
  await expect(page.locator("html")).toHaveAttribute("data-os", /mac|windows|other/);
});

test("a pet page POSTs exactly one visit per load", async ({ page }) => {
  const posts: string[] = [];
  page.on("request", (request) => {
    if (request.method() === "POST" && request.url().endsWith("/api/visit/ember"))
      posts.push(request.url());
  });

  const first = page.waitForResponse(
    (response) =>
      response.request().method() === "POST" && response.url().endsWith("/api/visit/ember"),
  );
  await gotoThemed(page, "/pets/ember/", "light");
  expect((await first).status()).toBe(204);
  await page.waitForTimeout(500);
  expect(posts).toHaveLength(1);

  posts.length = 0;
  const second = page.waitForResponse(
    (response) =>
      response.request().method() === "POST" && response.url().endsWith("/api/visit/ember"),
  );
  await page.reload({ waitUntil: "load" });
  expect((await second).status()).toBe(204);
  await page.waitForTimeout(500);
  expect(posts).toHaveLength(1);
});

test("theme toggle switches and persists", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await page.goto("/");
  const html = page.locator("html");
  await expect(html).not.toHaveClass(/dark/);

  await page.locator("[data-theme-toggle]").click();
  await expect(html).toHaveClass(/dark/);
  expect(await page.evaluate(() => localStorage.getItem("deskpet-theme"))).toBe("dark");

  await page.reload({ waitUntil: "load" });
  await expect(html).toHaveClass(/dark/);

  await page.locator("[data-theme-toggle]").click();
  await expect(html).not.toHaveClass(/dark/);
  expect(await page.evaluate(() => localStorage.getItem("deskpet-theme"))).toBe("light");
});

test("system dark is used when localStorage is blocked", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.addInitScript(() => {
    Storage.prototype.getItem = function () {
      throw new DOMException("Storage is blocked", "SecurityError");
    };
  });
  await page.goto("/");

  await expect(page.locator("html")).toHaveClass(/dark/);
  expect(await page.evaluate(() => document.documentElement.dataset.theme)).toBe("dark");
});

test("an explicit light choice still wins over system dark", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.addInitScript(() => localStorage.setItem("deskpet-theme", "light"));
  await page.goto("/");

  await expect(page.locator("html")).not.toHaveClass(/dark/);
  expect(await page.evaluate(() => document.documentElement.dataset.theme)).toBe("light");
});

test("hero animates by default, and every animation can pause and resume", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  const hero = page.locator(".hero-stage [data-anim]");
  const heroImage = hero.locator("img");
  await expect(hero).toHaveAttribute("data-playing", "true");
  await expect(heroImage).toHaveAttribute("src", /hero\.gif$/);

  await page.locator(".hero-stage [data-anim-toggle]").click();
  await expect(hero).toHaveAttribute("data-playing", "false");
  await expect(heroImage).toHaveAttribute("src", /hero\.png$/);
  await page.locator(".hero-stage [data-anim-toggle]").click();
  await expect(hero).toHaveAttribute("data-playing", "true");

  await page.locator("[data-anim-global]").click();
  await expect(page.locator("[data-anim-global]")).toHaveAttribute("aria-pressed", "true");
  for (const element of await page.locator("[data-anim]").all()) {
    await expect(element).toHaveAttribute("data-playing", "false");
  }

  await page.locator("[data-anim-global]").click();
  await expect(page.locator("[data-anim-global]")).toHaveAttribute("aria-pressed", "false");
  await expect(hero).toHaveAttribute("data-playing", "true");
});

test("trick GIFs animate when visible and pause individually", async ({ page }) => {
  await gotoThemed(page, "/pets/ember/", "light");
  const figure = page.locator('section[aria-labelledby="tricks"] figure').first();
  const media = figure.locator("[data-anim]");
  await figure.scrollIntoViewIfNeeded();
  await expect(media).toHaveAttribute("data-playing", "true");
  await expect(media.locator("img")).toHaveAttribute("src", /firework\.gif$/);

  await figure.locator("[data-anim-toggle]").click();
  await expect(media).toHaveAttribute("data-playing", "false");
  await expect(media.locator("img")).toHaveAttribute("src", /firework\.png$/);
  await expect(figure.locator("figcaption")).toContainText(/\d+(\.\d)?s/);
});

test("gallery frames show every pet's full artwork on a matching background", async ({ page }) => {
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  await loadAllImages(page);
  const cards = page.locator(".gallery-grid a.card");
  await expect(cards).toHaveCount(15);

  for (let index = 0; index < (await cards.count()); index++) {
    const card = cards.nth(index);
    await card.scrollIntoViewIfNeeded();
    const result = await card.locator(".pet-card-media").evaluate((frame) => {
      const image = frame.querySelector("img")!;
      const style = getComputedStyle(image);
      return {
        frameWidth: frame.clientWidth,
        frameHeight: frame.clientHeight,
        ratio: frame.clientWidth / frame.clientHeight,
        fit: style.objectFit,
        position: style.objectPosition,
        frameBackground: getComputedStyle(frame).backgroundColor,
        imageBackground: style.backgroundColor,
        loaded: image.naturalWidth > 0 && image.naturalHeight > 0,
      };
    });

    expect(result.loaded, `card ${index} image did not load`).toBe(true);
    expect(result.fit, `card ${index} clips the full artwork`).toBe("contain");
    expect(result.position, `card ${index} is not centred`).toMatch(/50% 50%|center/);
    expect(result.ratio, `card ${index} frame is not 16:10`).toBeCloseTo(16 / 10, 2);
    expect(result.frameBackground, `card ${index} frame background mismatch`).toBe(
      "rgb(241, 227, 200)",
    );
    expect(result.imageBackground, `card ${index} image background mismatch`).toBe(
      "rgb(241, 227, 200)",
    );
  }
});

test("trick frames follow each clip's natural aspect ratio, poster and GIF", async ({ page }) => {
  await gotoThemed(page, "/pets/ember/", "light", { reducedMotion: true });
  await loadAllImages(page);
  const figures = page.locator('section[aria-labelledby="tricks"] figure');
  await expect(figures).toHaveCount(4);

  for (let index = 0; index < (await figures.count()); index++) {
    const figure = figures.nth(index);
    const media = figure.locator("[data-anim]");
    const image = media.locator("img");
    await figure.scrollIntoViewIfNeeded();
    await expect(image).toHaveAttribute("src", /\.png$/);
    await checkTrickGeometry(media, index, "poster");

    await figure.locator("[data-anim-toggle]").click();
    await expect(image).toHaveAttribute("src", /\.gif$/);
    await loadAllImages(page);
    await checkTrickGeometry(media, index, "GIF");
  }
});

async function checkTrickGeometry(frame: Locator, index: number, state: string) {
  const result = await frame.evaluate((element) => {
    const image = element.querySelector("img")!;
    const frameBox = element.getBoundingClientRect();
    const imageBox = image.getBoundingClientRect();
    return {
      frameWidth: element.clientWidth,
      frameHeight: element.clientHeight,
      frameRatio: element.clientWidth / element.clientHeight,
      imageWidth: imageBox.width,
      imageHeight: imageBox.height,
      imageRatio: imageBox.width / imageBox.height,
      naturalRatio: image.naturalWidth / image.naturalHeight,
      fit: getComputedStyle(image).objectFit,
      contained:
        imageBox.left >= frameBox.left - 1 &&
        imageBox.top >= frameBox.top - 1 &&
        imageBox.right <= frameBox.right + 1 &&
        imageBox.bottom <= frameBox.bottom + 1,
      loaded: image.naturalWidth > 0 && image.naturalHeight > 0,
    };
  });

  expect(result.loaded, `figure ${index} ${state} did not load`).toBe(true);
  expect(result.contained, `figure ${index} ${state} clips its image`).toBe(true);
  expect(result.fit, `figure ${index} ${state} is letterboxed`).not.toBe("contain");
  expect(
    Math.abs(result.frameRatio - result.naturalRatio),
    `figure ${index} ${state} frame is fixed`,
  ).toBeLessThanOrEqual(0.02);
  expect(result.imageRatio, `figure ${index} ${state} is distorted`).toBeCloseTo(
    result.naturalRatio,
    2,
  );
  expect(Math.abs(result.imageWidth - result.frameWidth)).toBeLessThanOrEqual(1);
  expect(Math.abs(result.imageHeight - result.frameHeight)).toBeLessThanOrEqual(1);
}

test("download labels are single-line and buttons share a height in every OS state", async ({
  page,
}) => {
  const widths = [320, 375, 1024, 1280, 1440, 1920];

  for (const width of widths) {
    await page.setViewportSize({ width, height: 900 });
    await gotoThemed(page, "/", "light", { reducedMotion: true });

    const mac = page.locator('a[data-os-link="mac"]');
    const win = page.locator('a[data-os-link="windows"]');
    await expect(mac).toHaveAccessibleName(/Download for macOS/);
    await expect(win).toHaveAccessibleName(/Download for Windows/);

    for (const os of ["mac", "windows", "other"] as const) {
      await page.locator("html").evaluate((element, value) => {
        element.dataset.os = value;
      }, os);

      const heights: number[] = [];
      for (const link of [mac, win]) {
        const label = link.locator(".os-label");
        const meta = link.locator(".os-meta");
        const box = await link.boundingBox();
        const labelBox = await label.boundingBox();
        const metaBox = await meta.boundingBox();
        const lines = await label.evaluate((element) => {
          const range = document.createRange();
          range.selectNodeContents(element);
          return range.getClientRects().length;
        });

        expect(box, `${os} ${width}px button has no box`).not.toBeNull();
        expect(labelBox, `${os} ${width}px label has no box`).not.toBeNull();
        expect(metaBox, `${os} ${width}px meta has no box`).not.toBeNull();
        expect(lines, `${os} ${width}px label wrapped`).toBe(1);
        expect(
          metaBox!.y,
          `${os} ${width}px meta is not on the second line`,
        ).toBeGreaterThanOrEqual(labelBox!.y + labelBox!.height - 1);
        heights.push(box!.height);
      }

      expect(
        Math.abs(heights[0] - heights[1]),
        `${os} ${width}px buttons have different heights`,
      ).toBeLessThanOrEqual(1);

      if (os === "other") {
        await expect(mac.locator(".os-flag")).toBeHidden();
        await expect(win.locator(".os-flag")).toBeHidden();
      } else {
        const active = os === "windows" ? win : mac;
        await expect(active.locator(".os-flag")).toBeVisible();
        const subBox = await active.locator(".os-sub").boundingBox();
        const flagBox = await active.locator(".os-flag").boundingBox();
        expect(subBox, `${os} ${width}px extension has no box`).not.toBeNull();
        expect(flagBox, `${os} ${width}px recommendation has no box`).not.toBeNull();
        expect(
          Math.abs(subBox!.y + subBox!.height / 2 - (flagBox!.y + flagBox!.height / 2)),
          `${os} ${width}px recommendation is not beside the extension`,
        ).toBeLessThanOrEqual(2);
      }

      await expectNoHorizontalOverflow(page);
    }
  }
});

test("footer and donation use honest privacy wording", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  const donate = page.locator('section[aria-labelledby="donate"]');
  await expect(page.locator("footer")).toContainText("no ads, no cookies and no accounts");
  await expect(donate).toContainText("no ads, no cookies and no accounts");
  await expect(page.getByText(/tracking/i)).toHaveCount(0);
});

test("inline text keeps its spaces at element boundaries", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  await expect(
    page.getByText(/no cookies and no accounts\. If DeskPet made your desk a little happier/),
  ).toBeVisible();

  await gotoThemed(page, "/contribute/", "light");
  await expect(page.getByText(/The pet guide \(CONTRIBUTING-PETS\.md\) covers/)).toBeVisible();
});

test.describe("reduced motion", () => {
  test.use({ reducedMotion: "reduce" });

  test("still posters by default; GIF loads only when playback is requested", async ({ page }) => {
    const gifRequests: string[] = [];
    page.on("request", (request) => {
      if (/\.gif(\?|$)/.test(request.url())) gifRequests.push(request.url());
    });

    await gotoThemed(page, "/", "light", { reducedMotion: true });
    const hero = page.locator(".hero-stage [data-anim]");
    const heroImage = hero.locator("img");
    await page.waitForTimeout(700);
    await expect(hero).toHaveAttribute("data-playing", "false");
    await expect(heroImage).toHaveAttribute("src", /hero\.png$/);
    expect(gifRequests).toEqual([]);

    await page.locator(".hero-stage [data-anim-toggle]").click();
    await expect(hero).toHaveAttribute("data-playing", "true");
    await expect(heroImage).toHaveAttribute("src", /hero\.gif$/);
    expect(gifRequests.some((url) => url.endsWith("/pets/mochi/hero.gif"))).toBe(true);

    await page.locator(".hero-stage [data-anim-toggle]").click();
    await expect(heroImage).toHaveAttribute("src", /hero\.png$/);
  });
});

test("keyboard: skip link, visible focus and reachable navigation", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  await page.keyboard.press("Tab");
  const skip = page.locator(".skip-link");
  await expect(skip).toBeFocused();
  await expect.poll(async () => (await skip.boundingBox())?.y ?? -1).toBeGreaterThanOrEqual(0);

  await page.keyboard.press("Tab");
  const brand = page.locator("header a").first();
  await expect(brand).toBeFocused();
  const outline = await brand.evaluate((element) => getComputedStyle(element).outlineWidth);
  expect(outline).not.toBe("0px");

  await page.keyboard.press("Shift+Tab");
  await expect(skip).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/#main$/);

  const main = page.locator("#main");
  await expect(main).toBeFocused();
  const notObscured = await page.evaluate(() => {
    const header = document.querySelector("header")!;
    const target = document.querySelector("#main")!;
    return target.getBoundingClientRect().top >= header.getBoundingClientRect().bottom - 1;
  });
  expect(notObscured).toBe(true);

  await page.keyboard.press("Tab");
  expect(await page.evaluate(() => Boolean(document.activeElement?.closest("main")))).toBe(true);
});

test("layout holds from 320px to 1920px", async ({ page }) => {
  for (const width of [320, 1920]) {
    await page.setViewportSize({ width, height: 900 });
    for (const path of ["/", "/pets/ember/", "/contribute/"]) {
      await gotoThemed(page, path, "dark", { reducedMotion: true });
      await loadAllImages(page);
      await expectNoHorizontalOverflow(page);
    }
  }
});
