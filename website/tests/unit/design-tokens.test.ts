import { describe, expect, it } from "vitest";
import { checkTokenSheet, scanSource } from "../../scripts/design-tokens.mjs";

const rules = (source: string) => scanSource(source).map((violation) => violation.rule);

describe("design-token lint", () => {
  it("rejects raw CSS colour literals outside global.css", () => {
    expect(rules('<a style="color: #a75d2b">link</a>')).toContain("raw-color");
    expect(rules('class="bg-[rgb(167,93,43)]"')).toContain("raw-color");
    expect(rules("border-color: oklch(0.6 0.1 40);")).toContain("raw-color");
  });

  it("rejects arbitrary Tailwind values", () => {
    expect(rules('class="p-[13px]"')).toContain("arbitrary-value");
    expect(rules('class="text-[#fff]"')).toContain("arbitrary-value");
    expect(rules('class="rounded-[10px]"')).toContain("arbitrary-value");
  });

  it("rejects default Tailwind palette colours", () => {
    expect(rules('class="bg-white text-slate-500 border-red-300"')).toContain("default-color");
    expect(rules('class="hover:bg-gray-100"')).toContain("default-color");
  });

  it("rejects default and arbitrary radii and spacing", () => {
    expect(rules('class="rounded-lg"')).toContain("default-radius");
    expect(rules('class="rounded-full"')).toContain("default-radius");
    expect(rules('class="rounded"')).toContain("default-radius");
    expect(rules('class="p-4 mt-2 gap-6"')).toContain("default-spacing");
    expect(rules('class="mx-[2rem]"')).toContain("arbitrary-value");
  });

  it("rejects raw CSS radius and spacing declarations", () => {
    expect(rules("<style>.x { border-radius: 10px; margin: 1rem; gap: 8px }</style>")).toEqual(
      expect.arrayContaining(["raw-radius", "raw-spacing"]),
    );
    expect(rules("<style>.x { margin: var(--spacing-cozy) }</style>")).toEqual([]);
    expect(rules("<style>.x { padding: 0; margin: auto; gap: 0 }</style>")).toEqual([]);
  });

  it("catches raw spacing after a token and longhand radii", () => {
    expect(rules("<style>.x { padding: var(--spacing-cozy) 13px }</style>")).toContain(
      "raw-spacing",
    );
    expect(rules("<style>.x { margin: 0 auto 1.5rem }</style>")).toContain("raw-spacing");
    expect(rules("<style>.x { gap: var(--spacing-snug) 0.5em }</style>")).toContain("raw-spacing");
    expect(rules("<style>.x { border-top-left-radius: 8px }</style>")).toContain("raw-radius");
    expect(rules("<style>.x { border-start-start-radius: 8px }</style>")).toContain("raw-radius");
    expect(rules("<style>.x { border-radius: var(--radius-card) }</style>")).toEqual([]);
  });

  it("rejects raw named colours in UI styles", () => {
    expect(rules("<style>.x { color: red; background: white }</style>")).toContain(
      "raw-named-color",
    );
    expect(rules("<style>.x { border: 1px solid black }</style>")).toContain("raw-named-color");
    expect(rules("<style>.x { outline: transparent }</style>")).toContain("raw-named-color");
    expect(rules("<style>.x { color: var(--ink) }</style>")).toEqual([]);
  });

  it("rejects inline style attributes and expressions", () => {
    expect(rules('<div style="padding: 1rem"></div>')).toContain("inline-style");
    expect(rules("<div style={styles.card}></div>")).toContain("inline-style");
    expect(rules("<div style={{ padding: '1rem' }}></div>")).toContain("inline-style");
    expect(rules("<div style = { styles.card }></div>")).toContain("inline-style");
  });

  it("allows semantic classes, token variables, anchors and intrinsic SVG geometry", () => {
    const source = `
      <div class="card media-frame frame-gallery rounded-card bg-surface text-ink">
        <svg viewBox="0 0 8 8" width="32" height="32"><path fill="var(--brand)" /></svg>
        <a href="#main">Skip</a>
        <a href="#author-deskpet">Author</a>
        <img width="181" height="113" alt="" />
        <p class="badge">15 pets</p>
      </div>
    `;
    expect(scanSource(source)).toEqual([]);
  });

  it("reports missing shared tokens", () => {
    const missing = checkTokenSheet(":root { --canvas: #fff; }").map((item) => item.text);
    expect(missing).toContain("--brand");
    expect(missing).toContain("--radius-card");
    expect(missing).toContain("--spacing-section");
    expect(missing).not.toContain("--canvas");
  });
});
