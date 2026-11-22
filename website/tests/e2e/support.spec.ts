import { mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Locator, type Page } from "@playwright/test";
import jsQR from "jsqr";
import sharp from "sharp";
import { gotoThemed, loadAllImages } from "../helpers";

const themes = ["light", "dark"] as const;
const pets = JSON.parse(
  readFileSync(join(process.cwd(), "src", "data", "pets.generated.json"), "utf8"),
).pets as { id: string; shortName: string }[];
const QR_ALT = "MoMo and VietQR donation QR code for DeskPet";
const BNB_QR_ALT = "QR code for the BNB Smart Chain address";
const BNB_ADDRESS = "0x4d809c797169fa32448f075c15c4ad0c500bb109";
const DONATE_SOURCE = join(process.cwd(), "public", "donate-qr.png");
const IMAGE_SCALE = 4;

async function decodeFile(file: string) {
  const { data, info } = await sharp(file)
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  return jsQR(new Uint8ClampedArray(data), info.width, info.height)?.data ?? "";
}

async function decodeRendered(locator: Locator) {
  const box = await locator.boundingBox();
  const shot = await locator.screenshot();
  const meta = await sharp(shot).metadata();
  if (!box || !meta.width) return { payload: "", cssSize: 0 };

  const { data, info } = await sharp(shot)
    .resize({ width: meta.width * IMAGE_SCALE, kernel: "nearest" })
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  const result = jsQR(new Uint8ClampedArray(data), info.width, info.height);
  if (!result) return { payload: "", cssSize: 0 };

  const { topLeftCorner, topRightCorner, bottomLeftCorner, bottomRightCorner } = result.location;
  const distance = (a: { x: number; y: number }, b: { x: number; y: number }) =>
    Math.hypot(a.x - b.x, a.y - b.y);
  const imageEdge = Math.max(
    distance(topLeftCorner, topRightCorner),
    distance(topRightCorner, bottomRightCorner),
    distance(bottomRightCorner, bottomLeftCorner),
    distance(bottomLeftCorner, topLeftCorner),
  );
  return {
    payload: result.data,
    cssSize: (imageEdge / IMAGE_SCALE) * (box.width / meta.width),
  };
}

async function openWallet(page: Page) {
  await page.locator("header .support-btn").click();
  const dialog = page.locator("dialog[open]");
  await expect(dialog).toBeVisible();
  return dialog;
}

async function expectDialogFits(dialog: Locator) {
  const metrics = await dialog.evaluate((element) => {
    const rect = element.getBoundingClientRect();
    return {
      top: rect.top,
      bottom: rect.bottom,
      viewport: window.innerHeight,
      scrollHeight: element.scrollHeight,
      clientHeight: element.clientHeight,
      scrollWidth: element.scrollWidth,
      clientWidth: element.clientWidth,
    };
  });

  expect(metrics.top, "dialog is clipped above the viewport").toBeGreaterThanOrEqual(0);
  expect(metrics.bottom, "dialog is clipped below the viewport").toBeLessThanOrEqual(
    metrics.viewport + 1,
  );
  expect(
    metrics.scrollHeight,
    `dialog scrolls vertically (${metrics.scrollHeight} > ${metrics.clientHeight})`,
  ).toBeLessThanOrEqual(metrics.clientHeight + 1);
  expect(
    metrics.scrollWidth,
    `dialog scrolls horizontally (${metrics.scrollWidth} > ${metrics.clientWidth})`,
  ).toBeLessThanOrEqual(metrics.clientWidth + 1);

  const content = [
    dialog.getByRole("heading", { name: "Support DeskPet" }),
    dialog.locator(".support-intro p").last(),
    dialog.locator(".support-card"),
    dialog.getByRole("heading", { name: "Or send BNB / BEP-20 tokens" }),
    dialog.getByText("BNB Smart Chain (BEP-20)"),
    dialog.getByText(/Send only on BNB Smart Chain/),
    dialog.locator("[data-bnb-address]"),
    dialog.getByRole("button", { name: "Copy BNB address" }),
    dialog.getByRole("img", { name: BNB_QR_ALT }),
  ];

  for (const item of content) {
    await expect(item).toBeVisible();
    await expect(item).toBeInViewport();
    const box = await item.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.y, "content is clipped above the dialog").toBeGreaterThanOrEqual(metrics.top - 1);
    expect(box!.y + box!.height, "content is clipped below the dialog").toBeLessThanOrEqual(
      metrics.bottom + 1,
    );
  }
}

test("the Support link opens a native dialog and returns focus on close", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  const trigger = page.locator("header").getByRole("link", { name: "Support" });
  const dialog = page.locator("dialog[open]");

  await trigger.click();
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("heading", { name: "Support DeskPet" })).toBeVisible();
  await expect(dialog).toContainText("no ads, no cookies and no accounts");
  await expect(dialog).toContainText("a tip keeps new pets coming");
  await expect(dialog.getByRole("img", { name: QR_ALT })).toBeVisible();
  await expect(dialog).toContainText("MoMo / VietQR");

  const close = dialog.getByRole("button", { name: "Close support dialog" });
  await expect(close).toBeFocused();

  for (let index = 0; index < 10; index++) {
    await page.keyboard.press("Tab");
    expect(
      await page.evaluate(() => Boolean(document.activeElement?.closest("dialog[open]"))),
      "focus left the modal dialog",
    ).toBe(true);
  }

  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();

  await trigger.click();
  await expect(dialog).toBeVisible();
  await page.mouse.click(4, 4);
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();

  await trigger.click();
  await expect(dialog).toBeVisible();
  await close.click();
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
});

for (const theme of themes) {
  test(`support dialog has no axe violations (${theme})`, async ({ page }) => {
    await gotoThemed(page, "/", theme);
    await page.locator("header").getByRole("link", { name: "Support" }).click();
    await expect(page.locator("dialog[open]")).toBeVisible();

    const results = await new AxeBuilder({ page })
      .include("dialog[open]")
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22a", "wcag22aa"])
      .analyze();

    expect(
      results.violations.map((violation) => ({
        id: violation.id,
        impact: violation.impact,
        nodes: violation.nodes.map((node) => node.target.join(" ")),
      })),
    ).toEqual([]);
  });
}

test.describe("without JavaScript", () => {
  test.use({ javaScriptEnabled: false });

  test("the Support link navigates to a fallback with both payment options", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 900 });
    await page.goto("/", { waitUntil: "load" });
    const trigger = page.locator("header").getByRole("link", { name: "Support" });
    await expect(trigger).toHaveAttribute("href", "/support/");
    await trigger.click();
    await expect(page).toHaveURL(/\/support\/$/);
    await expect(page.getByRole("heading", { name: "Support DeskPet" })).toBeVisible();
    await expect(page.getByRole("img", { name: QR_ALT })).toBeVisible();
    await expect(page.locator("main .support-content")).toContainText(
      "no ads, no cookies and no accounts",
    );
    await expect(page.locator("main .support-content")).toContainText("MoMo / VietQR");
    await expect(page.getByRole("heading", { name: "Or send BNB / BEP-20 tokens" })).toBeVisible();
    await expect(page.locator("main .support-tabs")).toBeHidden();
    await expect(page.locator("main .support-momo")).toBeVisible();
    await expect(page.locator("main .support-crypto")).toBeVisible();
    await expect(page.locator('main [role="tabpanel"]')).toHaveCount(0);
    await expect(page.locator("main [data-bnb-address]")).toHaveText(BNB_ADDRESS);
    await expect(page.getByRole("img", { name: BNB_QR_ALT })).toBeVisible();
    await expect(page.locator("main [data-bnb-copy]")).toBeHidden();
    await expect(page.locator(".fine-print")).toHaveText(
      "Prefer to help another way? Make a pet or share the project.",
    );
  });
});

for (const width of [320, 375, 1440]) {
  test(`header Support sits in the action group and stays visible at ${width}px`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    await gotoThemed(page, "/", "light", { reducedMotion: true });

    const support = page.locator("header .header-actions .support-btn");
    await expect(support).toHaveCount(1);
    await expect(support).toBeVisible();
    await expect(support).toHaveAttribute("href", "/support/");
    await expect(page.locator("header .main-nav .support-btn")).toHaveCount(0);
    await expect(page.locator("header .nav-list")).not.toContainText("Support");

    const box = await support.boundingBox();
    expect(box, `${width}px Support button has no box`).not.toBeNull();
    expect(box!.width, `${width}px Support target width`).toBeGreaterThanOrEqual(24);
    expect(box!.height, `${width}px Support target height`).toBeGreaterThanOrEqual(24);
    await expect(support).toHaveAccessibleName(/Support/);

    await support.focus();
    const outline = await support.evaluate((element) => getComputedStyle(element).outlineWidth);
    expect(outline, `${width}px Support focus outline`).not.toBe("0px");
  });
}

for (const theme of themes) {
  for (const viewport of [
    { width: 1280, height: 720 },
    { width: 1440, height: 900 },
  ]) {
    test(`desktop support dialog fits ${viewport.width}x${viewport.height} without scrolling (${theme})`, async ({
      page,
    }) => {
      await page.setViewportSize(viewport);
      await gotoThemed(page, "/", theme, { reducedMotion: true });
      const dialog = await openWallet(page);
      await loadAllImages(page);

      await expectDialogFits(dialog);

      const card = dialog.locator(".support-card");
      const cardBox = await card.boundingBox();
      expect(cardBox).not.toBeNull();
      expect(cardBox!.width).toBeGreaterThanOrEqual(320);
      expect(cardBox!.width).toBeLessThanOrEqual(360);

      const momo = await decodeRendered(dialog.getByRole("img", { name: QR_ALT }));
      expect(momo.payload, "rendered MoMo QR payload").toBe(await decodeFile(DONATE_SOURCE));
      expect(momo.cssSize, "rendered MoMo QR physical width").toBeGreaterThanOrEqual(200);

      const bnb = await decodeRendered(dialog.getByRole("img", { name: BNB_QR_ALT }));
      expect(bnb.payload, "rendered BNB QR payload").toBe(BNB_ADDRESS);
      expect(bnb.cssSize, "rendered BNB QR physical width").toBeGreaterThan(60);
    });
  }
}

test("mobile support dialog fits 375x667 without scrolling", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 667 });
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  const dialog = await openWallet(page);
  await loadAllImages(page);

  const metrics = await dialog.evaluate((element) => {
    const rect = element.getBoundingClientRect();
    return {
      top: rect.top,
      bottom: rect.bottom,
      viewport: window.innerHeight,
      scrollHeight: element.scrollHeight,
      clientHeight: element.clientHeight,
    };
  });
  expect(metrics.top, "dialog is clipped above the viewport").toBeGreaterThanOrEqual(0);
  expect(metrics.bottom, "dialog is clipped below the viewport").toBeLessThanOrEqual(
    metrics.viewport + 1,
  );
  expect(
    metrics.scrollHeight,
    `mobile dialog scrolls (${metrics.scrollHeight} > ${metrics.clientHeight})`,
  ).toBeLessThanOrEqual(metrics.clientHeight + 1);

  for (const item of [
    dialog.getByRole("heading", { name: "Support DeskPet" }),
    dialog.locator(".support-card"),
  ]) {
    await expect(item).toBeVisible();
    await expect(item).toBeInViewport();
    const box = await item.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.y + box!.height, "content is clipped below the dialog").toBeLessThanOrEqual(
      metrics.bottom + 1,
    );
  }
});

test("MoMo card restores the branded poster with a magnified QR", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  const dialog = await openWallet(page);
  await loadAllImages(page);

  const card = dialog.locator(".support-card");
  const cardBox = await card.boundingBox();
  expect(cardBox).not.toBeNull();
  expect(cardBox!.width).toBeGreaterThanOrEqual(320);
  expect(cardBox!.width).toBeLessThanOrEqual(360);

  const image = dialog.getByRole("img", { name: QR_ALT });
  const shape = await image.evaluate((element) => {
    const img = element as HTMLImageElement;
    const box = img.getBoundingClientRect();
    return {
      width: box.width,
      height: box.height,
      naturalWidth: img.naturalWidth,
      naturalHeight: img.naturalHeight,
      objectFit: getComputedStyle(img).objectFit,
    };
  });

  expect(shape.naturalWidth).toBe(640);
  expect(shape.naturalHeight).toBeGreaterThan(shape.naturalWidth);
  expect(shape.objectFit).not.toBe("cover");
  expect(shape.height).toBeGreaterThan(shape.width);
  await expect(dialog.locator(".support-card-caption")).toContainText("NGUYEN QUOC THINH");
  await expect(dialog.locator(".support-card-caption")).toContainText("napas 247");
  await expect(dialog.locator("picture source")).toHaveAttribute("type", "image/webp");
});

for (const theme of themes) {
  test(`BNB wallet decodes exactly and stays small (${theme})`, async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await gotoThemed(page, "/", theme, { reducedMotion: true });
    const dialog = await openWallet(page);

    await expect(
      dialog.getByRole("heading", { name: "Or send BNB / BEP-20 tokens" }),
    ).toBeVisible();
    await expect(dialog).toContainText("BNB Smart Chain (BEP-20)");
    await expect(dialog).toContainText(
      "Send only on BNB Smart Chain. Funds sent on another network can't be recovered.",
    );
    await expect(dialog.locator("[data-bnb-address]")).toHaveText(BNB_ADDRESS);
    await expect(dialog.getByRole("button", { name: "Copy BNB address" })).toBeVisible();

    expect(
      await dialog
        .locator(".support-momo")
        .evaluate((momo) => momo.nextElementSibling?.classList.contains("support-crypto")),
      "BNB wallet is not the payment panel next to MoMo",
    ).toBe(true);

    const bnb = await decodeRendered(dialog.getByRole("img", { name: BNB_QR_ALT }));
    expect(bnb.payload, "rendered BNB QR payload").toBe(BNB_ADDRESS);

    const cardBox = await dialog.locator(".support-card").boundingBox();
    const qrBox = await dialog.getByRole("img", { name: BNB_QR_ALT }).boundingBox();
    expect(cardBox).not.toBeNull();
    expect(qrBox).not.toBeNull();
    expect(qrBox!.width, "BNB QR is not clearly smaller than the MoMo card").toBeLessThan(
      cardBox!.width / 2,
    );
  });
}

test("Copy writes the exact address to the real clipboard and announces it", async ({
  page,
  context,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  const dialog = await openWallet(page);
  const copy = dialog.getByRole("button", { name: "Copy BNB address" });

  await copy.click();
  await expect(copy).toHaveText("Copied");
  await expect(dialog.getByRole("status")).toHaveText("Address copied to the clipboard.");
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(BNB_ADDRESS);
});

for (const [name, override] of [
  [
    "the Clipboard API is missing",
    () => {
      Object.defineProperty(navigator, "clipboard", { configurable: true, value: undefined });
    },
  ],
  [
    "the clipboard write is denied",
    () => {
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: { writeText: () => Promise.reject(new Error("denied")) },
      });
    },
  ],
] as const) {
  test(`fallback selects the address when ${name}`, async ({ page }) => {
    await page.addInitScript(override);
    await gotoThemed(page, "/", "light", { reducedMotion: true });
    const dialog = await openWallet(page);

    await dialog.getByRole("button", { name: "Copy BNB address" }).click();
    await expect(dialog.getByRole("status")).toContainText(
      "Clipboard unavailable - the address is selected",
    );
    expect(await page.evaluate(() => window.getSelection()?.toString() ?? "")).toBe(BNB_ADDRESS);
  });
}

test("mobile payment tabs are keyboard operable and default to MoMo", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 900 });
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  const dialog = await openWallet(page);

  const tablist = dialog.getByRole("tablist", { name: "Payment method" });
  const momoTab = dialog.getByRole("tab", { name: "MoMo / VietQR" });
  const bnbTab = dialog.getByRole("tab", { name: "BNB" });
  const momoPanel = dialog.locator(".support-momo");
  const bnbPanel = dialog.locator(".support-crypto");

  await expect(tablist).toBeVisible();
  await expect(momoTab).toBeVisible();
  await expect(bnbTab).toBeVisible();
  await expect(momoTab).toHaveAttribute("aria-selected", "true");
  await expect(bnbTab).toHaveAttribute("aria-selected", "false");
  await expect(momoPanel).toBeVisible();
  await expect(bnbPanel).toBeHidden();

  await momoTab.focus();
  await page.keyboard.press("ArrowRight");
  await expect(bnbTab).toBeFocused();
  await expect(bnbTab).toHaveAttribute("aria-selected", "true");
  await expect(momoTab).toHaveAttribute("aria-selected", "false");
  await expect(momoTab).toHaveAttribute("tabindex", "-1");
  await expect(bnbTab).toHaveAttribute("tabindex", "0");
  await expect(momoPanel).toBeHidden();
  await expect(bnbPanel).toBeVisible();

  await page.keyboard.press("Home");
  await expect(momoTab).toBeFocused();
  await expect(momoTab).toHaveAttribute("aria-selected", "true");
  await expect(momoPanel).toBeVisible();
  await expect(bnbPanel).toBeHidden();

  await page.keyboard.press("End");
  await expect(bnbTab).toBeFocused();
  await expect(bnbTab).toHaveAttribute("aria-selected", "true");
  await page.keyboard.press("ArrowLeft");
  await expect(momoTab).toBeFocused();
  await expect(momoTab).toHaveAttribute("aria-selected", "true");
});

test("tabpanel roles are only exposed while the tab UI is active", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  const dialog = await openWallet(page);

  await expect(dialog.locator('[data-support-panel][role="tabpanel"]')).toHaveCount(0);
  await expect(dialog.locator(".support-momo")).toBeVisible();
  await expect(dialog.locator(".support-crypto")).toBeVisible();

  await page.setViewportSize({ width: 375, height: 900 });
  await expect(dialog.locator(".support-tabs")).toBeVisible();
  await expect(dialog.locator('[data-support-panel][role="tabpanel"]')).toHaveCount(2);
  await expect(dialog.locator(".support-momo")).toHaveAttribute("role", "tabpanel");
  await expect(dialog.locator(".support-crypto")).toHaveAttribute("role", "tabpanel");
});

test("empty clipboard status adds no layout and Copy shares the address row at 375", async ({
  page,
}) => {
  await page.setViewportSize({ width: 375, height: 900 });
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  const dialog = await openWallet(page);
  await dialog.getByRole("tab", { name: "BNB" }).click();

  const status = dialog.locator("[data-bnb-status]");
  expect(await status.evaluate((element) => getComputedStyle(element).position)).toBe("absolute");

  const row = await dialog.locator("[data-bnb]").evaluate((element) => {
    const text = element.querySelector("[data-bnb-address]")!.getBoundingClientRect();
    const copy = element.querySelector("[data-bnb-copy]")!.getBoundingClientRect();
    const box = element.getBoundingClientRect();
    return {
      copySameRow: copy.top < text.bottom && text.top < copy.bottom && copy.left > text.left,
      bottomGap: box.bottom - Math.max(text.bottom, copy.bottom),
    };
  });

  expect(row.copySameRow, "Copy is not on the address row").toBe(true);
  expect(row.bottomGap, "empty status still adds a line").toBeLessThanOrEqual(9);
});

test("dialog and page support instances use unique element ids", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await gotoThemed(page, "/support/", "light", { reducedMotion: true });
  await openWallet(page);

  const duplicates = await page.evaluate(() => {
    const seen = new Set<string>();
    const repeated = new Set<string>();
    for (const element of document.querySelectorAll<HTMLElement>("[id]")) {
      if (seen.has(element.id)) repeated.add(element.id);
      seen.add(element.id);
    }
    return [...repeated];
  });
  expect(duplicates).toEqual([]);
});

test("every pet page shows the exact support prompt and opens the dialog", async ({ page }) => {
  test.setTimeout(120_000);
  for (const pet of pets) {
    await gotoThemed(page, `/pets/${pet.id}/`, "light", { reducedMotion: true });
    const prompt = page.locator(".pet-copy .support-prompt");
    await expect(prompt).toBeVisible();
    await expect(prompt.locator("p")).toHaveText(
      `Enjoying ${pet.shortName}? Help keep DeskPet free.`,
    );
    await expect(prompt.locator(".support-btn")).toHaveCount(1);
    expect(
      await page
        .locator(".pet-copy .pet-action")
        .evaluate((action) => action.nextElementSibling?.classList.contains("support-prompt")),
      `${pet.id} support prompt is not directly after the pet action`,
    ).toBe(true);

    await prompt.locator(".support-btn").click();
    const dialog = page.locator("dialog[open]");
    await expect(dialog).toBeVisible();
    await expect(dialog.getByRole("heading", { name: "Support DeskPet" })).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();
  }
});

test("pet art and copy columns align at the top on Ember at 1440", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await gotoThemed(page, "/pets/ember/", "light", { reducedMotion: true });
  const stage = await page.locator(".pet-stage").boundingBox();
  const copy = await page.locator(".pet-copy").boundingBox();
  expect(stage).not.toBeNull();
  expect(copy).not.toBeNull();
  expect(Math.abs(stage!.y - copy!.y), "pet art and copy are not top aligned").toBeLessThanOrEqual(
    2,
  );
});

const DIALOG_SHOTS = join(process.cwd(), "qa", "support-dialog");
mkdirSync(DIALOG_SHOTS, { recursive: true });

for (const theme of themes) {
  test(`dialog screenshot 1440x900 (${theme})`, async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await gotoThemed(page, "/", theme, { reducedMotion: true });
    await openWallet(page);
    await loadAllImages(page);
    await page.screenshot({ path: join(DIALOG_SHOTS, `dialog-1440-${theme}.png`) });
  });
}

test("dialog screenshot 1280x720 (light)", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  await openWallet(page);
  await loadAllImages(page);
  await page.screenshot({ path: join(DIALOG_SHOTS, "dialog-1280-light.png") });
});

test("mobile dialog screenshots 375px, both tabs", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 900 });
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  const dialog = await openWallet(page);
  await loadAllImages(page);
  await page.screenshot({ path: join(DIALOG_SHOTS, "dialog-375-momo-light.png") });

  await dialog.getByRole("tab", { name: "BNB" }).click();
  await expect(dialog.locator(".support-crypto")).toBeVisible();
  await loadAllImages(page);
  await page.screenshot({ path: join(DIALOG_SHOTS, "dialog-375-bnb-light.png") });
});

test("Ember pet page screenshot 1440x900", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await gotoThemed(page, "/pets/ember/", "light", { reducedMotion: true });
  await loadAllImages(page);
  await page.screenshot({ path: join(DIALOG_SHOTS, "ember-1440-light.png") });
});
