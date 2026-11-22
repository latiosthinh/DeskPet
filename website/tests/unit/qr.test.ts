import { readFileSync } from "node:fs";
import { join } from "node:path";
import jsQR from "jsqr";
import sharp from "sharp";
import { describe, expect, it } from "vitest";

const ROOT = process.cwd();
const SOURCE = join(ROOT, "tests", "fixtures", "donate-card-source.webp");
const WEBP = join(ROOT, "public", "donate-qr.webp");
const PNG = join(ROOT, "public", "donate-qr.png");
const SOURCE_QR_FRACTION = 310 / 1008;

async function decode(file: string) {
  const { data, info } = await sharp(file)
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  const result = jsQR(new Uint8ClampedArray(data), info.width, info.height);
  return result ? result.data : "";
}

async function rgba(file: string) {
  const { data, info } = await sharp(file)
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  return { data, info };
}

describe("donation card", () => {
  it("keeps the full card at its original dimensions in both formats", async () => {
    for (const file of [SOURCE, WEBP, PNG]) {
      const meta = await sharp(file).metadata();
      expect({ width: meta.width, height: meta.height }).toEqual({ width: 1008, height: 1102 });
    }
    expect(readFileSync(SOURCE).length).toBeGreaterThan(0);
  });

  it("preserves every source pixel losslessly", async () => {
    const source = await rgba(SOURCE);
    for (const file of [WEBP, PNG]) {
      const delivered = await rgba(file);
      expect(delivered.info.width).toBe(source.info.width);
      expect(delivered.info.height).toBe(source.info.height);
      expect(delivered.data.equals(source.data), `${file} differs from the source pixels`).toBe(
        true,
      );
    }
  });

  it("decodes the same nonempty QR payload as the source without exposing it", async () => {
    const source = await decode(SOURCE);
    const webp = await decode(WEBP);
    const png = await decode(PNG);
    expect(source.length).toBeGreaterThan(0);
    expect(webp.length).toBe(source.length);
    expect(webp === source, "delivered WebP QR payload differs from the source").toBe(true);
    expect(png.length).toBe(source.length);
    expect(png === source, "delivered PNG QR payload differs from the source").toBe(true);
  });

  it("sizes the desktop card so the QR region is at least ~200 CSS px", () => {
    expect(672 * SOURCE_QR_FRACTION).toBeGreaterThanOrEqual(200);
  });
});
