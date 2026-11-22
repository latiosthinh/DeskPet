import { readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test, type Locator, type Page } from "@playwright/test";
import { expectNoHorizontalOverflow, gotoThemed, loadAllImages } from "../helpers";

const pets = JSON.parse(
  readFileSync(join(process.cwd(), "src", "data", "pets.generated.json"), "utf8"),
).pets as { id: string; trickCount: number }[];

const WIDTHS = [320, 375, 768, 1024, 1440, 1920];
const FRAME = "rgb(241, 227, 200)";

const RELEASE = "https://github.com/latiosthinh/DeskPet/releases/latest";

test("home shows a featured shelf and links into the catalogue", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  await expect(page.locator("h1")).toHaveCount(1);
  await expect(page.locator("#gallery")).toHaveCount(1);
  await expect(page.locator(".gallery-grid a.card")).toHaveCount(4);
  await expect(page.locator("#author-deskpet")).toHaveCount(0);
  await expect(page.locator(".author-nav")).toHaveCount(0);
  await expect(page.locator(".group-head")).toHaveCount(0);

  const browse = page.locator('#gallery a[href="/pets/"]', { hasText: "Browse all pets" });
  await expect(browse).toHaveText("Browse all pets");
  await browse.click();
  await expect(page).toHaveURL(/\/pets\/$/);
  await expect(page.locator("h1")).toHaveText("Pets");

  await gotoThemed(page, "/", "light");
  await page.locator('a.card[href="/pets/ember/"]').click();
  await expect(page).toHaveURL(/\/pets\/ember\/$/);
  await expect(page.locator("h1")).toHaveText("Ember the fox");
});

test("More pets navigation keeps the page inset at every breakpoint", async ({ page }) => {
  test.setTimeout(120_000);
  for (const width of [375, 768, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await gotoThemed(page, "/pets/mochi/", "light", { reducedMotion: true });
    await loadAllImages(page);

    const inset = await page.locator(".crumbs").boundingBox();
    const card = await page
      .locator('nav[aria-label="More pets"] .pet-nav-card')
      .first()
      .boundingBox();
    expect(inset, `${width}px content inset has no box`).not.toBeNull();
    expect(card, `${width}px More pets card has no box`).not.toBeNull();
    expect(inset!.x, `${width}px content is flush to the viewport`).toBeGreaterThan(0);
    expect(
      Math.abs(card!.x - inset!.x),
      `${width}px More pets nav is not aligned with the page inset`,
    ).toBeLessThanOrEqual(1);
  }
});

test("gallery card metadata and status rows share one reserved geometry", async ({ page }) => {
  test.setTimeout(180_000);
  for (const width of [375, 768, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await gotoThemed(page, "/", "light", { reducedMotion: true });
    await loadAllImages(page);

    const metrics = await page.locator(".gallery-grid a.card").evaluateAll((cards) =>
      cards.map((card) => {
        const box = card.getBoundingClientRect();
        const title = card.querySelector(".card-title")!.getBoundingClientRect();
        const meta = card.querySelector(".card-meta")!.getBoundingClientRect();
        const status = card.querySelector(".card-status")!.getBoundingClientRect();
        return {
          titleTop: title.top - box.top,
          metaTop: meta.top - box.top,
          statusTop: status.top - box.top,
          statusHeight: status.height,
          footHeight: card.querySelector(".card-foot")!.getBoundingClientRect().height,
        };
      }),
    );

    expect(metrics.length, `${width}px gallery is empty`).toBe(4);
    const first = metrics[0];
    for (const [index, metric] of metrics.entries()) {
      expect(metric.titleTop, `${width}px card ${index} title row`).toBeCloseTo(first.titleTop, 0);
      expect(metric.metaTop, `${width}px card ${index} meta row`).toBeCloseTo(first.metaTop, 0);
      expect(metric.statusTop, `${width}px card ${index} status row`).toBeCloseTo(
        first.statusTop,
        0,
      );
      expect(metric.statusHeight, `${width}px card ${index} status height`).toBeCloseTo(
        first.statusHeight,
        0,
      );
      expect(metric.footHeight, `${width}px card ${index} foot height`).toBeCloseTo(
        first.footHeight,
        0,
      );
    }
  }
});

test("404 pet card matches the home gallery card at requested breakpoints", async ({ page }) => {
  test.setTimeout(120_000);
  for (const width of [375, 768, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await gotoThemed(page, "/", "light", { reducedMotion: true });
    await loadAllImages(page);
    const home = await cardShape(page.locator(".gallery-grid a.card").first(), 0);

    await gotoThemed(page, "/this-page-does-not-exist/", "light", { reducedMotion: true });
    await loadAllImages(page);
    const lostCard = page.locator(".lost-pet a.card");
    const lost = await cardShape(lostCard, 0);
    const box = await lostCard.boundingBox();

    for (const key of ["outerWidth", "outerHeight", "mediaWidth", "mediaHeight"] as const) {
      expect(Math.abs(lost[key] - home[key]), `${width}px 404 card ${key}`).toBeLessThanOrEqual(1);
    }
    expect(box, `${width}px 404 card has no box`).not.toBeNull();
    expect(
      Math.abs(box!.x + box!.width / 2 - width / 2),
      `${width}px 404 card is not centred`,
    ).toBeLessThanOrEqual(1);
  }
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

test("header Pets and Authors links reach the catalogue and directory", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  const header = page.locator("header");

  await header.getByRole("link", { name: "Pets", exact: true }).click();
  await expect(page).toHaveURL(/\/pets\/$/);
  await expect(page.locator("h1")).toHaveText("Pets");

  await gotoThemed(page, "/", "light");
  await header.getByRole("link", { name: "Authors", exact: true }).click();
  await expect(page).toHaveURL(/\/authors\/$/);
  await expect(page.locator("h1")).toHaveText("Authors");
});

test("footer link targets meet the 24px minimum", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  const links = page.locator("footer nav a");
  await expect(links).toHaveCount(6);
  for (const link of await links.all()) {
    const box = await link.boundingBox();
    expect(box, "footer link has no layout box").not.toBeNull();
    expect(box!.width).toBeGreaterThanOrEqual(24);
    expect(box!.height).toBeGreaterThanOrEqual(24);
  }
});

test("download link targets: add-ons download, every built-in is included", async ({ page }) => {
  await gotoThemed(page, "/pets/ember/", "light");
  const download = page.locator('a[href="/download/ember"]');
  await expect(download).toBeVisible();
  await expect(download).toHaveAttribute("download", "ember.deskpet");
  await expect(page.getByText("version 1.0.0").first()).toBeVisible();

  for (const id of ["mochi", "ninja", "moona"]) {
    await gotoThemed(page, `/pets/${id}/`, "light");
    await expect(page.getByText("Included with DeskPet").first()).toBeVisible();
    await expect(page.locator(`a[href="/download/${id}"]`)).toHaveCount(0);
    await expect(page.locator(`a[download="${id}.deskpet"]`)).toHaveCount(0);
  }
});

test("pet pages link the author to their profile", async ({ page }) => {
  await gotoThemed(page, "/pets/mochi/", "light");
  const author = page.locator(".pet-meta a", { hasText: "DeskPet" });
  await expect(author).toHaveAttribute("href", "/authors/deskpet/");
  await author.click();
  await expect(page).toHaveURL(/\/authors\/deskpet\/$/);
  await expect(page.locator("h1")).toHaveText("DeskPet");
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
  await expect(heroImage).toHaveAttribute("src", /hero\.sprite\.gif$/);

  await page.locator(".hero-stage [data-anim-toggle]").click();
  await expect(hero).toHaveAttribute("data-playing", "false");
  await expect(heroImage).toHaveAttribute("src", /hero\.sprite\.png$/);
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
  await expect(media.locator("img")).toHaveAttribute("src", /firework\.sprite\.gif$/);

  await figure.locator("[data-anim-toggle]").click();
  await expect(media).toHaveAttribute("data-playing", "false");
  await expect(media.locator("img")).toHaveAttribute("src", /firework\.sprite\.png$/);
  await expect(figure.locator("figcaption")).toContainText(/\d+ s/);
});

test("trick play controls sit on the picture and captions sit below it", async ({ page }) => {
  await gotoThemed(page, "/pets/ember/", "light", { reducedMotion: true });
  const figure = page.locator(".trick-grid figure").first();
  const frame = figure.locator(".media-frame");
  const button = figure.locator("[data-anim-toggle]");
  const image = frame.locator("img");
  const title = figure.locator(".tile-title");
  const duration = figure.locator(".duration");

  await expect(button).toHaveAccessibleName(/Play Ember the fox/);
  const frameBox = await frame.boundingBox();
  const buttonBox = await button.boundingBox();
  const imageBox = await image.boundingBox();
  const titleBox = await title.boundingBox();
  const durationBox = await duration.boundingBox();

  expect(frameBox, "picture frame has no box").not.toBeNull();
  expect(buttonBox, "play control has no box").not.toBeNull();
  expect(imageBox, "picture has no box").not.toBeNull();
  expect(titleBox, "title has no box").not.toBeNull();
  expect(durationBox, "duration has no box").not.toBeNull();

  expect(buttonBox!.width, "play control is narrower than 24px").toBeGreaterThanOrEqual(24);
  expect(buttonBox!.height, "play control is shorter than 24px").toBeGreaterThanOrEqual(24);
  expect(buttonBox!.x, "play control starts outside the picture").toBeGreaterThanOrEqual(
    frameBox!.x,
  );
  expect(buttonBox!.y, "play control starts outside the picture").toBeGreaterThanOrEqual(
    frameBox!.y,
  );
  expect(buttonBox!.x + buttonBox!.width, "play control overflows the picture").toBeLessThanOrEqual(
    frameBox!.x + frameBox!.width + 1,
  );
  expect(
    buttonBox!.y + buttonBox!.height,
    "play control overflows the picture",
  ).toBeLessThanOrEqual(frameBox!.y + frameBox!.height + 1);
  expect(titleBox!.y, "title is not below the picture").toBeGreaterThanOrEqual(
    imageBox!.y + imageBox!.height - 1,
  );
  expect(durationBox!.y, "duration is not below the title").toBeGreaterThanOrEqual(
    titleBox!.y + titleBox!.height - 1,
  );
  await expect(duration).toHaveText(/\d+ s/);
});

test("every gallery card has one fixed image box at every breakpoint", async ({ page }) => {
  test.setTimeout(180_000);
  for (const width of WIDTHS) {
    await page.setViewportSize({ width, height: 900 });
    await gotoThemed(page, "/", "light", { reducedMotion: true });
    await loadAllImages(page);
    const cards = page.locator(".gallery-grid a.card");
    await expect(cards, `${width}px card count`).toHaveCount(4);

    const shapes: Shape[] = [];
    for (let index = 0; index < (await cards.count()); index++) {
      shapes.push(await cardShape(cards.nth(index), index));
    }
    assertSameBoxes(shapes, `${width}px gallery`);
  }
});

test("every trick tile on every pet has one fixed image box at every breakpoint", async ({
  page,
}) => {
  test.setTimeout(300_000);
  for (const width of WIDTHS) {
    for (const pet of pets) {
      await page.setViewportSize({ width, height: 900 });
      await gotoThemed(page, `/pets/${pet.id}/`, "light", { reducedMotion: true });
      await loadAllImages(page);
      const figures = page.locator(".trick-grid figure");
      await expect(figures, `${width}px ${pet.id} trick count`).toHaveCount(pet.trickCount);

      const shapes: Shape[] = [];
      for (let index = 0; index < (await figures.count()); index++) {
        shapes.push(await tileShape(figures.nth(index), index));
      }
      assertSameBoxes(shapes, `${width}px ${pet.id} tricks`);
    }
  }

  await page.setViewportSize({ width: 1024, height: 900 });
  await gotoThemed(page, "/pets/ember/", "light", { reducedMotion: true });
  const figure = page.locator(".trick-grid figure").first();
  const media = figure.locator("[data-anim]");
  const image = media.locator("img");
  const before = await tileShape(figure, 0);
  await expect(image).toHaveAttribute("src", /\.png$/);
  await figure.locator("[data-anim-toggle]").click();
  await expect(image).toHaveAttribute("src", /\.gif$/);
  await loadAllImages(page);
  const after = await tileShape(figure, 0);
  expect(after.mediaWidth).toBeCloseTo(before.mediaWidth, 1);
  expect(after.mediaHeight).toBeCloseTo(before.mediaHeight, 1);
  await expect(figure.locator("figcaption")).toContainText(/\d+ s/);
});

interface Shape {
  label: string;
  outerWidth: number;
  outerHeight: number;
  mediaWidth: number;
  mediaHeight: number;
  mediaRatio: number;
  fit: string;
  position: string;
  frameBackground: string;
  imageBackground: string;
  contained: boolean;
  loaded: boolean;
}

async function cardShape(card: Locator, index: number): Promise<Shape> {
  const shape = await card.evaluate((element, label) => {
    const media = element.querySelector(".media-frame") as HTMLElement;
    const image = media.querySelector("img") as HTMLImageElement;
    const box = element.getBoundingClientRect();
    const frame = media.getBoundingClientRect();
    const picture = image.getBoundingClientRect();
    const style = getComputedStyle(image);
    return {
      label,
      outerWidth: box.width,
      outerHeight: box.height,
      mediaWidth: frame.width,
      mediaHeight: frame.height,
      mediaRatio: frame.width / frame.height,
      fit: style.objectFit,
      position: style.objectPosition,
      frameBackground: getComputedStyle(media).backgroundColor,
      imageBackground: style.backgroundColor,
      contained:
        picture.left >= frame.left - 1 &&
        picture.top >= frame.top - 1 &&
        picture.right <= frame.right + 1 &&
        picture.bottom <= frame.bottom + 1,
      loaded: image.naturalWidth > 0 && image.naturalHeight > 0,
    };
  }, `card ${index}`);
  return shape;
}

async function tileShape(figure: Locator, index: number): Promise<Shape> {
  const shape = await figure.evaluate((element, label) => {
    const media = element.querySelector(".media-frame") as HTMLElement;
    const image = media.querySelector("img") as HTMLImageElement;
    const box = element.getBoundingClientRect();
    const frame = media.getBoundingClientRect();
    const picture = image.getBoundingClientRect();
    const style = getComputedStyle(image);
    return {
      label,
      outerWidth: box.width,
      outerHeight: box.height,
      mediaWidth: frame.width,
      mediaHeight: frame.height,
      mediaRatio: frame.width / frame.height,
      fit: style.objectFit,
      position: style.objectPosition,
      frameBackground: getComputedStyle(media).backgroundColor,
      imageBackground: style.backgroundColor,
      contained:
        picture.left >= frame.left - 1 &&
        picture.top >= frame.top - 1 &&
        picture.right <= frame.right + 1 &&
        picture.bottom <= frame.bottom + 1,
      loaded: image.naturalWidth > 0 && image.naturalHeight > 0,
    };
  }, `tile ${index}`);
  return shape;
}

function assertSameBoxes(shapes: readonly Shape[], where: string) {
  expect(shapes.length, `${where}: no boxes measured`).toBeGreaterThan(0);
  const first = shapes[0];
  for (const shape of shapes) {
    expect(shape.loaded, `${where} ${shape.label}: image did not load`).toBe(true);
    expect(shape.fit, `${where} ${shape.label}: artwork is not contained`).toBe("contain");
    expect(shape.position, `${where} ${shape.label}: artwork is not centred`).toMatch(
      /50% 50%|center/,
    );
    expect(shape.mediaRatio, `${where} ${shape.label}: image box is not 4:3`).toBeCloseTo(4 / 3, 2);
    expect(shape.contained, `${where} ${shape.label}: image is clipped`).toBe(true);
    expect(shape.frameBackground, `${where} ${shape.label}: frame background mismatch`).toBe(FRAME);
    expect(shape.imageBackground, `${where} ${shape.label}: image background mismatch`).toBe(FRAME);
    expect(
      Math.abs(shape.outerWidth - first.outerWidth),
      `${where} ${shape.label}: outer width differs`,
    ).toBeLessThanOrEqual(1);
    expect(
      Math.abs(shape.outerHeight - first.outerHeight),
      `${where} ${shape.label}: outer height differs`,
    ).toBeLessThanOrEqual(1);
    expect(
      Math.abs(shape.mediaWidth - first.mediaWidth),
      `${where} ${shape.label}: image box width differs`,
    ).toBeLessThanOrEqual(1);
    expect(
      Math.abs(shape.mediaHeight - first.mediaHeight),
      `${where} ${shape.label}: image box height differs`,
    ).toBeLessThanOrEqual(1);
  }
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

test("support and footer share the honest privacy wording", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  await expect(page.locator("footer")).toContainText("no ads, no cookies and no accounts");
  await expect(page.locator("#donate")).toHaveCount(0);
  await expect(page.locator('section[aria-labelledby="donate"]')).toHaveCount(0);

  await gotoThemed(page, "/support/", "light");
  await expect(page.locator("main .support-content")).toContainText(
    "no ads, no cookies and no accounts",
  );
  await expect(page.getByText(/tracking/i)).toHaveCount(0);
});

test("inline text keeps its spaces at element boundaries", async ({ page }) => {
  await gotoThemed(page, "/support/", "light");
  await expect(
    page
      .locator("main")
      .getByText(/no cookies and no accounts\. If DeskPet made your desk a little happier/),
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
    await expect(heroImage).toHaveAttribute("src", /hero\.sprite\.png$/);
    expect(gifRequests).toEqual([]);

    await page.locator(".hero-stage [data-anim-toggle]").click();
    await expect(hero).toHaveAttribute("data-playing", "true");
    await expect(heroImage).toHaveAttribute("src", /hero\.sprite\.gif$/);
    expect(gifRequests.some((url) => url.endsWith("/pets/mochi/hero.sprite.gif"))).toBe(true);

    await page.locator(".hero-stage [data-anim-toggle]").click();
    await expect(heroImage).toHaveAttribute("src", /hero\.sprite\.png$/);
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

const MOBILE_HEADER_WIDTHS = [320, 375, 639];

async function expectMobileHeader(page: Page, width: number) {
  const headerBox = await page.locator(".header-inner").boundingBox();
  expect(headerBox, `${width}px header has no box`).not.toBeNull();
  expect(headerBox!.height, `${width}px header is taller than 120px`).toBeLessThanOrEqual(120);

  await expect(
    page.locator("header .header-download"),
    `${width}px header Download should be hidden`,
  ).toBeHidden();

  const brandBox = await page.locator(".brand").boundingBox();
  const themeBox = await page.locator("[data-theme-toggle]").boundingBox();
  const animBox = await page.locator("[data-anim-global]").boundingBox();
  const supportBox = await page.locator("header .support-btn").boundingBox();
  const navBox = await page.locator(".main-nav").boundingBox();
  expect(brandBox, `${width}px brand has no box`).not.toBeNull();
  expect(themeBox, `${width}px theme toggle has no box`).not.toBeNull();
  expect(animBox, `${width}px animation toggle has no box`).not.toBeNull();
  expect(supportBox, `${width}px support button has no box`).not.toBeNull();
  expect(navBox, `${width}px nav has no box`).not.toBeNull();

  const rowOneBottom = Math.max(
    brandBox!.y + brandBox!.height,
    themeBox!.y + themeBox!.height,
    animBox!.y + animBox!.height,
    supportBox!.y + supportBox!.height,
  );
  expect(
    Math.abs(themeBox!.y - brandBox!.y),
    `${width}px brand and theme toggle should share the first row`,
  ).toBeLessThanOrEqual(2);
  expect(
    Math.abs(animBox!.y - brandBox!.y),
    `${width}px brand and animation toggle should share the first row`,
  ).toBeLessThanOrEqual(2);
  expect(
    Math.abs(supportBox!.y - brandBox!.y),
    `${width}px brand and support button should share the first row`,
  ).toBeLessThanOrEqual(2);
  expect(navBox!.y, `${width}px nav should be below the brand row`).toBeGreaterThanOrEqual(
    rowOneBottom - 2,
  );

  const nav = await page.locator(".nav-list").evaluate((list) => {
    const listBox = list.getBoundingClientRect();
    return {
      scrollWidth: list.scrollWidth,
      clientWidth: list.clientWidth,
      left: listBox.left,
      right: listBox.right,
      links: [...list.querySelectorAll<HTMLElement>(".nav-link")].map((link) => {
        const box = link.getBoundingClientRect();
        const range = document.createRange();
        range.selectNodeContents(link);
        return {
          text: link.textContent?.trim() ?? "",
          x: box.x,
          right: box.right,
          y: box.y,
          height: box.height,
          lines: range.getClientRects().length,
          scrollWidth: link.scrollWidth,
          clientWidth: link.clientWidth,
        };
      }),
    };
  });

  expect(nav.scrollWidth, `${width}px nav list overflows horizontally`).toBeLessThanOrEqual(
    nav.clientWidth + 1,
  );
  expect(
    nav.links.map((link) => link.text),
    `${width}px nav labels`,
  ).toEqual(["Pets", "Authors", "Make a pet", "GitHub"]);

  const rowY = nav.links[0].y;
  for (const [index, link] of nav.links.entries()) {
    expect(link.height, `${width}px ${link.text} touch height`).toBeGreaterThanOrEqual(44);
    expect(
      Math.abs(link.y - rowY),
      `${width}px ${link.text} is on the nav row`,
    ).toBeLessThanOrEqual(1);
    expect(link.lines, `${width}px ${link.text} label wrapped`).toBe(1);
    expect(link.scrollWidth, `${width}px ${link.text} label is clipped`).toBeLessThanOrEqual(
      link.clientWidth + 1,
    );
    expect(link.x, `${width}px ${link.text} starts inside the nav`).toBeGreaterThanOrEqual(
      nav.left - 1,
    );
    expect(link.right, `${width}px ${link.text} ends inside the nav`).toBeLessThanOrEqual(
      nav.right + 1,
    );
    if (index > 0) {
      expect(link.x, `${width}px ${link.text} overlaps the previous link`).toBeGreaterThanOrEqual(
        nav.links[index - 1].right,
      );
    }
  }
}

test("mobile header: brand and both controls row 1, every nav link row 2 at 320-639px", async ({
  page,
}) => {
  for (const width of MOBILE_HEADER_WIDTHS) {
    await page.setViewportSize({ width, height: 900 });
    await gotoThemed(page, "/", "light", { reducedMotion: true });
    await loadAllImages(page);
    await expectMobileHeader(page, width);
  }
});

test("mobile header survives deterministic wide font metrics at 320-639px", async ({ page }) => {
  for (const width of MOBILE_HEADER_WIDTHS) {
    await page.setViewportSize({ width, height: 900 });
    await gotoThemed(page, "/", "light", { reducedMotion: true });
    await loadAllImages(page);
    await page.addStyleTag({ content: "body { font-family: monospace !important; }" });
    await expectMobileHeader(page, width);
  }
});

test("desktop header keeps Download visible at 1440px", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  await loadAllImages(page);

  const download = page.locator("header .header-download");
  await expect(download, "header Download should be visible at 1440px").toBeVisible();
});

test("hero scene image drives frame height at 375 and 1440px", async ({ page }) => {
  for (const width of [375, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await gotoThemed(page, "/", "light", { reducedMotion: true });
    await loadAllImages(page);

    const frame = page.locator(".scene-pet .frame-scene");
    const image = frame.locator("img");
    const frameBox = await frame.boundingBox();
    const imageBox = await image.boundingBox();
    expect(frameBox, `${width}px frame has no box`).not.toBeNull();
    expect(imageBox, `${width}px image has no box`).not.toBeNull();

    const frameAspect = frameBox!.width / frameBox!.height;
    const imageNatural = await image.evaluate((img: HTMLImageElement) => ({
      w: img.naturalWidth,
      h: img.naturalHeight,
    }));
    const expectedAspect = imageNatural.w / imageNatural.h;
    expect(
      Math.abs(frameAspect - expectedAspect),
      `${width}px frame ratio does not match image intrinsic ratio`,
    ).toBeLessThan(0.02);

    expect(
      imageBox!.y + imageBox!.height,
      `${width}px image is not bottom-anchored`,
    ).toBeGreaterThanOrEqual(frameBox!.y + frameBox!.height - 1);
  }
});
