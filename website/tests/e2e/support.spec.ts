import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { gotoThemed } from "../helpers";

const themes = ["light", "dark"] as const;

test("the Support link opens a native dialog and returns focus on close", async ({ page }) => {
  await gotoThemed(page, "/", "light");
  const trigger = page.locator("header").getByRole("link", { name: "Support" });
  const dialog = page.locator("dialog[open]");

  await trigger.click();
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("heading", { name: "Support DeskPet" })).toBeVisible();
  await expect(dialog).toContainText("no ads, no cookies and no accounts");
  await expect(dialog).toContainText("Thank you for keeping DeskPet free for everyone.");
  await expect(dialog.getByRole("img", { name: /Donation QR code/ })).toBeVisible();

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
    await expect(page.getByRole("img", { name: /Donation QR code/ })).toBeVisible();
    await expect(page.locator("main .support-content")).toContainText(
      "no ads, no cookies and no accounts",
    );
    await expect(page.locator("main .support-content")).toContainText(
      "Thank you for keeping DeskPet free for everyone.",
    );
    await expect(page.locator(".fine-print")).toHaveText(
      "Prefer to help another way? Make a pet or share the project.",
    );
  });
});
