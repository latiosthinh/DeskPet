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
const SOURCE_QR_FRACTION = 310 / 1008;
const BNB_ADDRESS = "0x4d809c797169fa32448f075c15c4ad0c500bb109";

async function decodeQr(locator: Locator) {
  const shot = await locator.screenshot();
  const { data, info } = await sharp(shot)
    .resize({ width: 720, kernel: "nearest" })
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  const result = jsQR(new Uint8ClampedArray(data), info.width, info.height);
  return result?.data ?? "";
}

async function openWallet(page: Page) {
  await page.locator("header .support-btn").click();
  const dialog = page.locator("dialog[open]");
  await expect(dialog).toBeVisible();
  return dialog;
}

test("the Support link opens a native dialog and returns focus on close", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  const trigger = page.locator("header").getByRole("link", { name: "Support" });
  const dialog = page.locator("dialog[open]");

  await trigger.click();
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("heading", { name: "Support DeskPet" })).toBeVisible();
  await expect(dialog).toContainText("no ads, no cookies and no accounts");
  await expect(dialog).toContainText("Thank you for keeping DeskPet free for everyone.");
  await expect(dialog.getByRole("img", { name: QR_ALT })).toBeVisible();
  await expect(dialog).toContainText("VietQR / napas 247");

  const close = dialog.getByRole("button", { name: "Close support dialog" });
  await expect(close).toBeFocused();

  for (let index = 0; index < 8; index++) {
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

  test("the Support link navigates to the fallback page", async ({ page }) => {
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
    await expect(page.locator("main .support-content")).toContainText(
      "Thank you for keeping DeskPet free for everyone.",
    );
    await expect(page.getByRole("heading", { name: "Or send BNB / BEP-20 tokens" })).toBeVisible();
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

test("desktop dialog shows the whole QR card, uncropped, with a 200px+ QR region", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await gotoThemed(page, "/", "light", { reducedMotion: true });
  await page.locator("header .support-btn").click();
  const dialog = page.locator("dialog[open]");
  await expect(dialog).toBeVisible();
  await loadAllImages(page);

  const image = dialog.getByRole("img", { name: QR_ALT });
  await expect(image).toBeVisible();
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

  expect(shape.naturalWidth).toBe(1008);
  expect(shape.naturalHeight).toBe(1102);
  expect(shape.objectFit).not.toBe("cover");
  expect(
    Math.abs(shape.width / shape.height - shape.naturalWidth / shape.naturalHeight),
    "rendered card ratio differs from the source card",
  ).toBeLessThan(0.01);
  expect(
    Math.abs(shape.height - (shape.width * shape.naturalHeight) / shape.naturalWidth),
    "card is cropped vertically",
  ).toBeLessThanOrEqual(1);
  expect(
    shape.width * SOURCE_QR_FRACTION,
    "desktop QR region is smaller than 200 CSS px",
  ).toBeGreaterThanOrEqual(200);
  await expect(dialog.locator("picture source")).toHaveAttribute("type", "image/webp");
});

for (const theme of themes) {
  test(`BNB wallet sits below MoMo in the dialog and decodes exactly (${theme})`, async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1440, height: 1000 });
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
      "BNB wallet is not the section directly below MoMo",
    ).toBe(true);

    const qr = dialog.getByRole("img", { name: BNB_QR_ALT });
    await expect(qr).toBeVisible();
    expect(await decodeQr(qr), "rendered BNB QR payload").toBe(BNB_ADDRESS);

    const momoBox = await dialog.locator(".support-card").boundingBox();
    const qrBox = await qr.boundingBox();
    expect(momoBox).not.toBeNull();
    expect(qrBox).not.toBeNull();
    expect(qrBox!.width, "BNB QR is not clearly smaller than the MoMo card").toBeLessThan(
      momoBox!.width / 2,
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

test("every pet page shows the support prompt right after the action and opens the dialog", async ({
  page,
}) => {
  test.setTimeout(120_000);
  for (const pet of pets) {
    await gotoThemed(page, `/pets/${pet.id}/`, "light", { reducedMotion: true });
    const prompt = page.locator(".pet-copy .support-prompt");
    await expect(prompt).toBeVisible();
    await expect(prompt).toContainText(`Enjoying ${pet.shortName}? Support DeskPet`);
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

const DIALOG_SHOTS = join(process.cwd(), "qa", "support-dialog");
mkdirSync(DIALOG_SHOTS, { recursive: true });

for (const theme of themes) {
  for (const width of [375, 1440]) {
    test(`dialog open screenshot ${width}px (${theme})`, async ({ page }) => {
      await page.setViewportSize({ width, height: width === 1440 ? 1000 : 900 });
      await gotoThemed(page, "/", theme, { reducedMotion: true });
      await page.locator("header .support-btn").click();
      await expect(page.locator("dialog[open]")).toBeVisible();
      await loadAllImages(page);
      await page.screenshot({
        path: join(DIALOG_SHOTS, `dialog-${width}-${theme}.png`),
      });
    });
  }
}

const WALLET_SHOTS = join(process.cwd(), "qa", "support-wallet");
mkdirSync(WALLET_SHOTS, { recursive: true });

for (const theme of themes) {
  for (const width of [375, 1440]) {
    test(`wallet screenshot ${width}px (${theme})`, async ({ page }) => {
      await page.setViewportSize({ width, height: width === 1440 ? 1000 : 900 });
      await gotoThemed(page, "/", theme, { reducedMotion: true });
      const dialog = await openWallet(page);
      const wallet = dialog.locator(".support-crypto");
      await wallet.scrollIntoViewIfNeeded();
      await expect(wallet).toBeInViewport();
      await expect(dialog.getByRole("img", { name: BNB_QR_ALT })).toBeInViewport();
      await loadAllImages(page);
      await page.screenshot({
        path: join(WALLET_SHOTS, `wallet-${width}-${theme}.png`),
      });
    });
  }
}
