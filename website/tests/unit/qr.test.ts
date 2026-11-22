import { readFileSync } from "node:fs";
import { join } from "node:path";
import jsQR from "jsqr";
import sharp from "sharp";
import { describe, expect, it } from "vitest";

const ROOT = process.cwd();
const SOURCE = join(ROOT, "tests", "fixtures", "donate-card-source.webp");
const WEBP = join(ROOT, "public", "donate-qr.webp");
const PNG = join(ROOT, "public", "donate-qr.png");
const CARD_PNG = join(ROOT, "public", "support-momo-card.png");
const CARD_WEBP = join(ROOT, "public", "support-momo-card.webp");

async function decode(file: string) {
  const result = await decodeResult(file);
  return result ? result.data : "";
}

async function decodeResult(file: string) {
  const { data, info } = await sharp(file)
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  return jsQR(new Uint8ClampedArray(data), info.width, info.height);
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
});

describe("compact support card built from the poster", () => {
  it("is a 2x portrait card in both formats", async () => {
    for (const file of [CARD_PNG, CARD_WEBP]) {
      const meta = await sharp(file).metadata();
      expect(meta.width).toBe(640);
      expect(meta.height!).toBeGreaterThan(meta.width!);
    }
  });

  it("decodes to exactly the source payload", async () => {
    const source = await decode(SOURCE);
    expect(source.length).toBeGreaterThan(0);
    expect(await decode(CARD_PNG)).toBe(source);
    expect(await decode(CARD_WEBP)).toBe(source);
  });

  it("includes a quiet zone and leaves 200px+ of QR in a 320px card", async () => {
    const result = await decodeResult(CARD_PNG);
    expect(result).not.toBeNull();
    const corners = [
      result!.location.topLeftCorner,
      result!.location.topRightCorner,
      result!.location.bottomLeftCorner,
      result!.location.bottomRightCorner,
    ];
    const meta = await sharp(CARD_PNG).metadata();
    const edge = Math.min(meta.width!, meta.height!);
    const margin = Math.min(
      ...corners.flatMap((corner) => [corner.x, corner.y, edge - corner.x, edge - corner.y]),
    );
    expect(margin).toBeGreaterThanOrEqual(16);

    const qrWidth =
      Math.max(...corners.map((corner) => corner.x)) -
      Math.min(...corners.map((corner) => corner.x));
    expect((320 * qrWidth) / meta.width!).toBeGreaterThanOrEqual(200);
  });
});
