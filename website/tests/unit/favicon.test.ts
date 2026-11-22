import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import sharp from "sharp";
import { describe, expect, it } from "vitest";
import generated from "../../src/data/pets.generated.json";
import { hexToRgb, lightPalette } from "../../scripts/lib/palette.mjs";

const PUBLIC = join(process.cwd(), "public");
const publicFile = (url: string) => join(PUBLIC, url.replace(/^\//, ""));
const canvas = lightPalette().canvas;
const brand = hexToRgb(canvas);
const mochi = generated.pets.find((pet) => pet.id === "mochi");

const TAB_ICONS = [
  ["favicon-16.png", 16],
  ["favicon-32.png", 32],
  ["favicon-48.png", 48],
] as const;

const APP_ICONS = [
  ["apple-touch-icon.png", 180],
  ["icon-512.png", 512],
] as const;

describe("favicons and app icons", () => {
  it("writes square PNG icons at the expected sizes", async () => {
    for (const [name, size] of [...TAB_ICONS, ...APP_ICONS]) {
      const file = publicFile(`/${name}`);
      expect(existsSync(file), name).toBe(true);
      const meta = await sharp(file).metadata();
      expect(meta.width, name).toBe(size);
      expect(meta.height, name).toBe(size);
      expect(meta.width, name).toBe(meta.height);
    }
  });

  it("keeps the tab icons on a transparent background", async () => {
    for (const [name, size] of TAB_ICONS) {
      const file = publicFile(`/${name}`);
      const meta = await sharp(file).metadata();
      expect(meta.hasAlpha, name).toBe(true);
      const { data } = await sharp(file).ensureAlpha().raw().toBuffer({ resolveWithObject: true });
      expect(data.length).toBe(size * size * 4);
      expect(data[3], name).toBe(0);
    }
  });

  it("puts the app icons on the opaque brand canvas", async () => {
    for (const [name, size] of APP_ICONS) {
      const file = publicFile(`/${name}`);
      const { data } = await sharp(file).ensureAlpha().raw().toBuffer({ resolveWithObject: true });
      expect(data.length).toBe(size * size * 4);
      expect(data[0], `${name} red`).toBe(brand.r);
      expect(data[1], `${name} green`).toBe(brand.g);
      expect(data[2], `${name} blue`).toBe(brand.b);
      expect(data[3], `${name} first alpha`).toBe(255);
      for (let i = 3; i < data.length; i += 4) {
        expect(data[i], `${name} pixel ${i >> 2} is not opaque`).toBe(255);
      }
    }
  });

  it("fills the 32px tab icon with Mochi", async () => {
    const file = publicFile("/favicon-32.png");
    const { data, info } = await sharp(file)
      .ensureAlpha()
      .raw()
      .toBuffer({ resolveWithObject: true });
    const box = alphaBox(data, info.width, info.height);
    expect(box, "favicon-32 has no visible pixels").not.toBeNull();
    expect(box!.width).toBeLessThanOrEqual(32);
    expect(box!.height).toBeLessThanOrEqual(32);
    if (mochi && !mochi.hero.fallback) {
      expect(box!.height / 32).toBeGreaterThanOrEqual(0.8);
    }
  });

  it("packs favicon.ico with 16/32/48 PNG entries", () => {
    const ico = readFileSync(publicFile("/favicon.ico"));
    expect(ico.readUInt16LE(0), "reserved").toBe(0);
    expect(ico.readUInt16LE(2), "type").toBe(1);
    expect(ico.readUInt16LE(4), "count").toBe(3);

    const sizes = [16, 32, 48];
    for (const [index, size] of sizes.entries()) {
      const entry = 6 + index * 16;
      expect(ico[entry], `entry ${index} width`).toBe(size);
      expect(ico[entry + 1], `entry ${index} height`).toBe(size);
      expect(ico[entry + 2], `entry ${index} color count`).toBe(0);
      expect(ico[entry + 3], `entry ${index} reserved`).toBe(0);
      expect(ico.readUInt16LE(entry + 4), `entry ${index} planes`).toBe(1);
      expect(ico.readUInt16LE(entry + 6), `entry ${index} bit count`).toBe(32);

      const length = ico.readUInt32LE(entry + 8);
      const offset = ico.readUInt32LE(entry + 12);
      expect(length, `entry ${index} payload length`).toBeGreaterThan(0);
      expect(offset + length, `entry ${index} payload range`).toBeLessThanOrEqual(ico.length);
      expect(ico.readUInt32BE(offset), `entry ${index} PNG magic`).toBe(0x89504e47);
    }
  });

  it("serves a token-coloured webmanifest", () => {
    const manifest = JSON.parse(readFileSync(publicFile("/site.webmanifest"), "utf8"));
    expect(manifest.name).toBe("DeskPet");
    expect(manifest.short_name).toBe("DeskPet");

    for (const [src, sizes] of [
      ["/apple-touch-icon.png", "180x180"],
      ["/icon-512.png", "512x512"],
    ] as const) {
      expect(manifest.icons).toContainEqual({ src, sizes, type: "image/png" });
      expect(existsSync(publicFile(src)), src).toBe(true);
    }

    expect(manifest.theme_color).toBe(canvas);
    expect(manifest.background_color).toBe(canvas);
  });
});

/** Bounding box of pixels with alpha > 16, or null when the image is empty. */
function alphaBox(data: Buffer, width: number, height: number) {
  let minX = width;
  let minY = height;
  let maxX = -1;
  let maxY = -1;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      if (data[(y * width + x) * 4 + 3] <= 16) continue;
      if (x < minX) minX = x;
      if (y < minY) minY = y;
      if (x > maxX) maxX = x;
      if (y > maxY) maxY = y;
    }
  }
  if (maxX < 0) return null;
  return { minX, minY, maxX, maxY, width: maxX - minX + 1, height: maxY - minY + 1 };
}
