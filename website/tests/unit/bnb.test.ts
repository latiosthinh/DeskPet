import jsQR from "jsqr";
import sharp from "sharp";
import { renderSVG } from "uqr";
import { describe, expect, it } from "vitest";
import { BNB_ADDRESS } from "../../src/lib/site";

const EXPECTED = "0x4d809c797169fa32448f075c15c4ad0c500bb109";

async function decode(svg: string) {
  const { data, info } = await sharp(Buffer.from(svg))
    .resize({ width: 720, kernel: "nearest" })
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  return jsQR(new Uint8ClampedArray(data), info.width, info.height)?.data ?? "";
}

describe("BNB support address", () => {
  it("matches the owner's address byte for byte", () => {
    expect(BNB_ADDRESS).toBe(EXPECTED);
    expect(BNB_ADDRESS).toHaveLength(42);
    expect(BNB_ADDRESS).toMatch(/^0x[0-9a-f]{40}$/);
  });

  it("generates a local QR code that decodes to exactly that address", async () => {
    const svg = renderSVG(BNB_ADDRESS, { ecc: "M", border: 2 });
    expect(svg.startsWith("<svg")).toBe(true);
    expect(await decode(svg)).toBe(EXPECTED);
  });
});
