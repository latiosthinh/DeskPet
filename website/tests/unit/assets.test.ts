import { existsSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import sharp from "sharp";
import { describe, expect, it } from "vitest";
import generated from "../../src/data/pets.generated.json";
import {
  authorGroups,
  authorSlug,
  builtinShortNames,
  categories,
  formatBytes,
  formatDuration,
  neighbors,
  pets,
  petsInCategory,
  type Pet,
} from "../../src/lib/site";
import { deriveCategories } from "../../scripts/lib/categories.mjs";
import { makeFallbackGif, readGifInfo } from "../../scripts/lib/gif.mjs";
import {
  GALLERY_HEIGHT,
  GALLERY_WIDTH,
  connectedComponents,
  detectPixelGrid,
  findBubble,
  frameView,
  heroBodyHeight,
  isContent,
  largestBody,
  readFrames,
  targetScale,
} from "../../scripts/lib/sprite.mjs";

const PUBLIC = join(process.cwd(), "public");
const publicFile = (url: string) => join(PUBLIC, url.replace(/^\//, ""));
const source = JSON.parse(readFileSync(join(process.cwd(), "pets.json"), "utf8"));

describe("pets data", () => {
  it("derives the shelves and 15 unique pets from the source inventory", () => {
    expect(categories.map((category) => category.id)).toEqual(
      deriveCategories(source).map((category) => category.id),
    );
    expect(pets).toHaveLength(source.length);
    expect(new Set(pets.map((pet) => pet.id)).size).toBe(pets.length);
    for (const pet of pets) {
      expect(categories.map((category) => category.id)).toContain(pet.category);
      expect(petsInCategory(pet.category)).toContainEqual(pet);
    }
  });

  it("ships exactly the builtin pets from the source inventory with the app", () => {
    const builtin = source.filter((pet: { builtin?: boolean }) => pet.builtin);
    expect(
      pets
        .filter((pet) => pet.builtin)
        .map((pet) => pet.id)
        .sort(),
    ).toEqual(builtin.map((pet: { id: string }) => pet.id).sort());
    expect(builtinShortNames).toEqual(
      builtin.map((pet: { name: string }) => pet.name.split(" the ")[0]),
    );
  });

  it("records true sizes, durations and existing poster paths for every animation", async () => {
    for (const pet of pets) {
      for (const media of [pet.hero, ...pet.tricks]) {
        expect(media.width).toBeGreaterThan(0);
        expect(media.height).toBeGreaterThan(0);
        expect(media.durationMs).toBeGreaterThan(0);
        expect(media.src).toMatch(/^\/pets\/.+\/(hero|[a-z0-9-]+)\.sprite\.gif$/);
        expect(media.poster).toMatch(/^\/pets\/.+\/(hero|[a-z0-9-]+)\.sprite\.png$/);

        expect(existsSync(publicFile(media.src)), media.src).toBe(true);
        expect(existsSync(publicFile(media.poster)), media.poster).toBe(true);
        expect(existsSync(publicFile(media.src.replace(/\.sprite\.gif$/, ".gif")))).toBe(true);
        expect(statSync(publicFile(media.poster)).size).toBeGreaterThan(0);

        const png = await sharp(publicFile(media.poster)).metadata();
        expect(png.width).toBe(media.width);
        expect(png.height).toBe(media.height);
        expect(png.hasAlpha).toBe(true);

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

  it("groups authors with DeskPet first, then alphabetical names", () => {
    const synthetic = [
      { id: "z", author: "Zed" },
      { id: "a", author: "Alice" },
      { id: "m", author: "DeskPet" },
      { id: "b", author: "DeskPet" },
    ] as unknown as Pet[];
    const groups = authorGroups(synthetic);
    expect(groups.map((group) => group.name)).toEqual(["DeskPet", "Alice", "Zed"]);
    expect(groups.map((group) => group.pets.length)).toEqual([2, 1, 1]);
    expect(groups.map((group) => group.slug)).toEqual([
      "deskpet",
      authorSlug("Alice"),
      authorSlug("Zed"),
    ]);
    expect(authorGroups(pets).map((group) => group.name)).toEqual(["DeskPet"]);
    expect(authorSlug("Desk Pet Studio")).toMatch(/^desk-pet-studio-[A-Za-z0-9_-]+$/);
  });

  it("makes deterministic, collision-free anchors for any nonempty author name", () => {
    const names = [
      "DeskPet",
      "Desk Pet Studio",
      "A B",
      "A-B",
      "A  B",
      "A B ",
      "alice",
      "Alice",
      "ALICE",
      "Mochi & Friends",
      "Mochi, Friends",
      "日本",
      "日本語",
      "Автор",
      "作者",
    ];
    const slugs = names.map((name) => authorSlug(name));

    expect(authorSlug("DeskPet")).toBe("deskpet");
    expect(new Set(slugs).size).toBe(names.length);
    expect(names.map((name) => authorSlug(name))).toEqual(slugs);
    for (const slug of slugs) expect(slug).toMatch(/^[A-Za-z0-9_-]+$/);
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

describe("normalised sprites", () => {
  it("puts every clip on one transparent canvas", async () => {
    const media = pets.flatMap((pet) => [pet.hero, ...pet.tricks]);
    const canvas = new Set(media.map((item) => `${item.width}x${item.height}`));
    expect(canvas.size, `different sprite canvases: ${[...canvas].join(", ")}`).toBe(1);

    for (const item of media) {
      const gif = await sharp(publicFile(item.src), { animated: true }).metadata();
      expect(gif.width).toBe(item.width);
      expect(gif.height / (gif.pages ?? 1)).toBe(item.height);
      expect(gif.hasAlpha).toBe(true);

      const poster = await sharp(publicFile(item.poster)).metadata();
      expect(poster.hasAlpha).toBe(true);
      const corner = await sharp(publicFile(item.poster)).ensureAlpha().raw().toBuffer();
      expect(corner[3], `${item.poster} background is not transparent`).toBe(0);
    }
  });

  it("enlarges every hero by a whole multiple of its source pixel grid", async () => {
    const medians: number[] = [];
    for (const pet of pets) {
      const raw = await readFrames(publicFile(`/pets/${pet.id}/hero.gif`));
      const rawBody = heroBodyHeight(raw);
      const scale = targetScale(rawBody);
      const grid = detectPixelGrid(raw);
      const heights = await bodyHeights(publicFile(pet.hero.src));
      const median = heights[heights.length >> 1];

      expect(scale % grid, `${pet.id} scale ${scale} does not respect its ${grid}x grid`).toBe(0);
      expect(
        Math.abs(median - rawBody * scale),
        `${pet.id} hero body is ${median}px, expected ${rawBody * scale}px (${scale}x ${rawBody}px)`,
      ).toBeLessThanOrEqual(1);
      if (!pet.hero.fallback) {
        expect(median, `${pet.id} hero body is too small`).toBeGreaterThanOrEqual(84);
        expect(median, `${pet.id} hero body is too large`).toBeLessThanOrEqual(112);
        medians.push(median);
      }
    }
    if (medians.length > 0) {
      expect(Math.max(...medians) / Math.min(...medians)).toBeLessThanOrEqual(1.35);
    }
  });

  it("builds a centred 4:3 gallery poster with comparable body sizes", async () => {
    const bodies: number[] = [];
    for (const pet of pets) {
      expect(pet.gallery.src).toMatch(/^\/pets\/.+\/gallery\.png$/);
      expect(pet.gallery.width).toBe(GALLERY_WIDTH);
      expect(pet.gallery.height).toBe(GALLERY_HEIGHT);
      const file = publicFile(pet.gallery.src);
      expect(existsSync(file), pet.gallery.src).toBe(true);

      const { data, info } = await sharp(file)
        .ensureAlpha()
        .raw()
        .toBuffer({ resolveWithObject: true });
      expect(info.width).toBe(GALLERY_WIDTH);
      expect(info.height).toBe(GALLERY_HEIGHT);
      expect(info.width / info.height).toBeCloseTo(4 / 3, 5);

      let minX = info.width;
      let minY = info.height;
      let maxX = -1;
      let maxY = -1;
      for (let y = 0; y < info.height; y++) {
        for (let x = 0; x < info.width; x++) {
          if (data[(y * info.width + x) * 4 + 3] <= 16) continue;
          if (x < minX) minX = x;
          if (y < minY) minY = y;
          if (x > maxX) maxX = x;
          if (y > maxY) maxY = y;
        }
      }
      expect(maxX, `${pet.id} gallery is empty`).toBeGreaterThanOrEqual(0);
      const boxWidth = maxX - minX + 1;
      const boxHeight = maxY - minY + 1;

      expect(
        Math.abs((minX + maxX + 1) / 2 - info.width / 2),
        `${pet.id} gallery body is not horizontally centred`,
      ).toBeLessThanOrEqual(1);
      expect(
        Math.abs((minY + maxY + 1) / 2 - info.height / 2),
        `${pet.id} gallery body is not vertically centred`,
      ).toBeLessThanOrEqual(1);
      expect(boxWidth, `${pet.id} gallery body is wider than its canvas`).toBeLessThanOrEqual(
        info.width,
      );
      if (!pet.hero.fallback) {
        expect(
          boxHeight / info.height,
          `${pet.id} gallery body is too small`,
        ).toBeGreaterThanOrEqual(0.5);
        expect(boxHeight / info.height, `${pet.id} gallery body is too large`).toBeLessThanOrEqual(
          0.67,
        );
        bodies.push(boxHeight);
      }
    }
    if (bodies.length > 0) {
      expect(Math.max(...bodies) / Math.min(...bodies)).toBeLessThanOrEqual(1.35);
    }
  });

  it("keeps the raw GIFs beside the clean derivatives", async () => {
    for (const pet of pets) {
      const raw = publicFile(`/pets/${pet.id}/hero.gif`);
      expect(existsSync(raw), raw).toBe(true);
      const derivative = publicFile(pet.hero.src);
      expect(derivative).not.toBe(raw);
      const rawInfo = readGifInfo(readFileSync(raw));
      expect(rawInfo.durationMs).toBe(pet.hero.durationMs);
    }
  });

  it("hero sprites have no greeting bubble or detached remnants in any frame", async () => {
    for (const id of ["mochi", "wispaling", "moona", "witcher"]) {
      const file = publicFile(`/pets/${id}/hero.sprite.gif`);
      const frames = await readFrames(file);
      let frameBubble = -1;
      let frameDetached = -1;
      for (let f = 0; f < frames.pages && frameBubble < 0 && frameDetached < 0; f++) {
        const data = frameView(frames, f);
        const bubble = findBubble(data, frames.width, frames.height);
        if (bubble) {
          frameBubble = f;
          break;
        }
        const { mask } = largestBody(data, frames.width, frames.height, null);
        for (let y = 0; y < frames.height; y++) {
          for (let x = 0; x < frames.width; x++) {
            const p = y * frames.width + x;
            if (!isContent(data, p * 4)) continue;
            if (!mask || !mask[p]) {
              frameDetached = f;
              break;
            }
          }
          if (frameDetached >= 0) break;
        }
      }
      expect(frameBubble, `${id} hero frame ${frameBubble} still has greeting bubble`).toBe(-1);
      expect(
        frameDetached,
        `${id} hero frame ${frameDetached} has detached content outside body`,
      ).toBe(-1);
    }
  });
});

async function bodyHeights(file: string): Promise<number[]> {
  const meta = await sharp(file).metadata();
  const { data, info } = await sharp(file, { animated: true })
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  const pageHeight = info.height / meta.pages;
  const pageBytes = info.width * pageHeight * 4;
  const heights: number[] = [];
  for (let page = 0; page < meta.pages; page++) {
    const mask = new Uint8Array(info.width * pageHeight);
    for (let i = page * pageBytes, p = 0; i < (page + 1) * pageBytes; i += 4, p++) {
      if (data[i + 3] > 16) mask[p] = 1;
    }
    const body = connectedComponents(mask, info.width, pageHeight)[0];
    if (body) heights.push(body.height);
  }
  return heights.sort((a, b) => a - b);
}

describe("formatting", () => {
  it("formats bytes and durations for the page", () => {
    expect(formatBytes(null)).toBeNull();
    expect(formatBytes(512)).toBe("1 KB");
    expect(formatBytes(130_911)).toBe("128 KB");
    expect(formatBytes(2 * 1024 * 1024)).toBe("2.0 MB");
    expect(formatDuration(2400)).toBe("2 s");
    expect(formatDuration(8600)).toBe("9 s");
    expect(formatDuration(12_400)).toBe("12 s");
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
