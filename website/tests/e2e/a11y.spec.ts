import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { PAGES, expectTheme, gotoThemed } from "../helpers";

const themes = ["light", "dark"] as const;

for (const theme of themes) {
  test.describe(`axe (${theme})`, () => {
    test.use({ colorScheme: theme });

    for (const target of PAGES) {
      test(`${target.name} has no WCAG 2.2 AA violations`, async ({ page }) => {
        await gotoThemed(page, target.path, theme);
        await expectTheme(page, theme);

        const results = await new AxeBuilder({ page })
          .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22a", "wcag22aa"])
          .analyze();

        const summary = results.violations.map((violation) => ({
          id: violation.id,
          impact: violation.impact,
          help: violation.help,
          nodes: violation.nodes.map((node) => node.target.join(" ")),
        }));
        expect(summary).toEqual([]);
      });
    }
  });
}
