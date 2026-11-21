import { existsSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import sharp from "sharp";
import { describe, expect, it } from "vitest";
import generated from "../../src/data/pets.generated.json";
import {
  builtinShortNames,
  categories,
  formatBytes,
  formatDuration,
  neighbors,
  pets,
  petsInCategory,
} from "../../src/lib/site";
import { makeFallbackGif, readGifInfo } from "../../scripts/lib/gif.mjs";

const PUBLIC = join(process.cwd(), "public");
const publicFile = (url: string) => join(PUBLIC, url.replace(/^\//, ""));

describe("pets data", () => {
  it("has the three shelves and 15 unique pets", () => {
    expect(categories.map((category) => category.id)).toEqual(["critter", "hero", "legend"]);
    expect(pets).toHaveLength(15);
    expect(new Set(pets.map((pet) => pet.id)).size).toBe(pets.length);
    for (const pet of pets) {
      expect(categories.map((category) => category.id)).toContain(pet.category);
      expect(petsInCategory(pet.category)).toContainEqual(pet);
    }
  });

  it("ships exactly Mochi, Kage and Moona with the app", () => {
    expect(
      pets
        .filter((pet) => pet.builtin)
        .map((pet) => pet.id)
        .sort(),
    ).toEqual(["mochi", "moona", "ninja"]);
    expect(builtinShortNames).toEqual(["Mochi", "Kage", "Moona"]);
  });

  it("records true sizes, durations and existing poster paths for every animation", async () => {
    for (const pet of pets) {
      for (const media of [pet.hero, ...pet.tricks]) {
        expect(media.width).toBeGreaterThan(0);
        expect(media.height).toBeGreaterThan(0);
        expect(media.durationMs).toBeGreaterThan(0);
        expect(media.src).toMatch(/^\/pets\/.+\/(hero|[a-z0-9-]+)\.gif$/);
        expect(media.poster).toMatch(/^\/pets\/.+\/(hero|[a-z0-9-]+)\.png$/);

        expect(existsSync(publicFile(media.src)), media.src).toBe(true);
        expect(existsSync(publicFile(media.poster)), media.poster).toBe(true);
        expect(statSync(publicFile(media.poster)).size).toBeGreaterThan(0);

        const png = await sharp(publicFile(media.poster)).metadata();
        expect(png.width).toBe(media.width);
        expect(png.height).toBe(media.height);

        const info = readGifInfo(readFileSync(publicFile(media.src)));
        expect(info.durationMs).toBe(media.durationMs);
      }

      expect(existsSync(publicFile(pet.og)), pet.og).toBe(true);
      const og = await sharp(publicFile(pet.og)).metadata();
      expect(og.width).toBe(1200);
      expect(og.height).toBe(630);

      expect(pet.tricks).toHaveLength(pet.trickCount);
      expect(new Set(pet.tricks.map((trick) => trick.id)).size).toBe(pet.tricks.length);
      if (!pet.builtin) expect(pet.fileSize).toBeGreaterThan(0);
    }
    expect(existsSync(publicFile(generated.qr.src))).toBe(true);
  });

  it("orders previous/next without wrapping", () => {
    expect(neighbors(pets[0].id).previous).toBeNull();
    expect(neighbors(pets.at(-1)!.id).next).toBeNull();
    for (let index = 0; index < pets.length; index++) {
      const links = neighbors(pets[index].id);
      if (index > 0) expect(links.previous?.id).toBe(pets[index - 1].id);
      if (index < pets.length - 1) expect(links.next?.id).toBe(pets[index + 1].id);
    }
  });
});

describe("formatting", () => {
  it("formats bytes and durations for the page", () => {
    expect(formatBytes(null)).toBeNull();
    expect(formatBytes(512)).toBe("1 KB");
    expect(formatBytes(130_911)).toBe("128 KB");
    expect(formatBytes(2 * 1024 * 1024)).toBe("2.0 MB");
    expect(formatDuration(2400)).toBe("2.4s");
    expect(formatDuration(12_400)).toBe("12s");
  });
});

describe("fallback art", () => {
  it("is a valid, animated GIF with a build-only marker", () => {
    const bytes = makeFallbackGif();
    expect(bytes.toString("ascii", 0, 6)).toBe("GIF89a");
    expect(bytes.toString("latin1")).toContain("BUILD-ONLY FALLBACK ART");
    const info = readGifInfo(bytes);
    expect(info).toEqual({ width: 64, height: 40, frames: 3, durationMs: 1350 });
  });

  it("rejects files that are not GIFs", () => {
    expect(() => readGifInfo(Buffer.from("not a gif at all"))).toThrow(/Not a GIF|too short/);
  });
});
