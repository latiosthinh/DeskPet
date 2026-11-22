#!/usr/bin/env node
// Build-time asset step. Runs before `astro dev` / `astro build`.
//
//   * reads pets.json (source of truth, written by petshot)
//   * keeps the real GIFs under public/pets/ when they exist
//   * creates small, clearly marked build-only fallback art when they do not, so a
//     fresh checkout (CI) can build without the generated players
//   * derives clean, normalised `.sprite.gif` clips and `.sprite.png` posters next
//     to every raw GIF (the raw GIFs are never edited)
//   * writes one Open Graph image per pet and src/data/pets.generated.json with
//     the true canvas sizes, durations and poster paths
//   * derives square favicons, app icons and site.webmanifest from Mochi's poster
//
// It never overwrites donate-qr.png, raw GIFs or .deskpet modules produced by petshot.

import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";
import { CATEGORY_ID, deriveCategories } from "./lib/categories.mjs";
import { makeFallbackGif, readGifInfo } from "./lib/gif.mjs";
import { hexToRgb, lightPalette } from "./lib/palette.mjs";
import {
  CANVAS_MARGIN,
  GALLERY_HEIGHT,
  GALLERY_WIDTH,
  contentBounds,
  detectPixelGrid,
  encodeFrames,
  findBubble,
  frameView,
  galleryFit,
  heroBodyHeight,
  isContent,
  largestBody,
  posterFrom,
  readFrames,
  scaleFrames,
  settledFrameIndex,
  targetScale,
} from "./lib/sprite.mjs";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const PUBLIC = join(ROOT, "public");
const PETS_FILE = join(ROOT, "pets.json");
const DATA_FILE = join(ROOT, "src", "data", "pets.generated.json");
const ID = /^[a-z0-9][a-z0-9-]{0,40}$/;
const palette = lightPalette();

const pets = JSON.parse(readFileSync(PETS_FILE, "utf8"));
const fallbacks = [];
validate(pets);

mkdirSync(join(PUBLIC, "pets"), { recursive: true });
mkdirSync(join(PUBLIC, "files"), { recursive: true });
mkdirSync(dirname(DATA_FILE), { recursive: true });

const rawPets = [];
for (const pet of pets) {
  const dir = join(PUBLIC, "pets", pet.id);
  mkdirSync(dir, { recursive: true });

  const hero = rawMedia(pet, "hero");
  const heroFrames = await readFrames(hero.path);
  const tricks = pet.tricks.map((trick) => ({ trick, raw: rawMedia(pet, trick.id) }));
  rawPets.push({ pet, hero, heroFrames, tricks });

  const marker = join(dir, "FALLBACK-ART.txt");
  const petFallbacks = [hero, ...tricks.map((item) => item.raw)].filter((item) => item.fallback);
  if (petFallbacks.length) {
    writeFileSync(
      marker,
      `Build-only fallback art for ${pet.id}. The real GIFs were not present in this checkout.\n` +
        `Run petshot site before deploying. Placeholder files:\n${petFallbacks
          .map((item) => item.path)
          .join("\n")}\n`,
    );
  } else if (existsSync(marker)) {
    rmSync(marker);
  }
}

// One body size, one canvas for every clip: the pet never changes size between
// a gallery card, a hero or a trick tile, whatever props or bubbles a clip has.
// The enlargement is an integer multiple of the source pixel grid, never a
// fractional nearest-neighbour resize.
const heroScale = new Map();
const jobs = [];
for (const { pet, hero, heroFrames, tricks } of rawPets) {
  const body = heroBodyHeight(heroFrames);
  const grid = detectPixelGrid(heroFrames);
  const scale = targetScale(body);
  heroScale.set(pet.id, { body, grid, scale });
  jobs.push({ pet, name: "hero", raw: hero, greeting: true, scale });
  for (const { trick, raw } of tricks) {
    jobs.push({ pet, name: trick.id, raw, greeting: false, scale, trick });
  }
}

let canvasWidth = 1;
let canvasHeight = 1;
for (const job of jobs) {
  const frames = await readFrames(job.raw.path);
  const bounds = contentBounds(frames, { greeting: job.greeting });
  if (!bounds) continue;
  canvasWidth = Math.max(canvasWidth, Math.ceil(bounds.width * job.scale) + CANVAS_MARGIN * 2);
  canvasHeight = Math.max(canvasHeight, Math.ceil(bounds.height * job.scale) + CANVAS_MARGIN * 2);
}

const normalised = new Map();
for (const job of jobs) {
  const media = await normalise(job, canvasWidth, canvasHeight);
  normalised.set(`${job.pet.id}/${job.name}`, media);
}

const built = [];
for (const { pet, heroFrames, tricks } of rawPets) {
  const heroMedia = normalised.get(`${pet.id}/hero`);
  const trickMedia = tricks.map(({ trick }) => ({
    ...trick,
    ...normalised.get(`${pet.id}/${trick.id}`),
  }));
  const { grid, scale } = heroScale.get(pet.id);
  const gallery = await galleryPoster(pet, heroFrames, scale, grid);

  const ogRel = `pets/${pet.id}/og.png`;
  await makeOg([join(PUBLIC, heroMedia.poster.slice(1))], join(PUBLIC, ogRel), pet.shortName);

  built.push({
    id: pet.id,
    name: pet.name,
    shortName: pet.shortName,
    category: pet.category,
    author: pet.author,
    version: pet.version,
    description: pet.description,
    builtin: Boolean(pet.builtin),
    fileSize: fileSize(pet),
    trickCount: pet.tricks.length,
    hero: heroMedia,
    gallery,
    og: `/${ogRel}`,
    tricks: trickMedia,
  });
}

const builtinPosters = built
  .filter((p) => p.builtin)
  .map((p) => join(PUBLIC, p.hero.poster.slice(1)));
if (builtinPosters.length) await makeOg(builtinPosters, join(PUBLIC, "og-default.png"), "DeskPet");

// donate-qr.png is pixel-locked (see tests/unit/qr.test.ts), so the crops that build
// the compact support card are fixed source rectangles. The QR card is lifted to a
// 640px-wide canvas (2x a 320px card) so the QR stays sharp and decodes at 200px+.
const MOMO_CARD_CROPS = {
  logo: { left: 430, top: 60, width: 180, height: 120 },
  qr: { left: 286, top: 305, width: 436, height: 475 },
  payee: { left: 230, top: 900, width: 550, height: 140 },
};
const MOMO_CARD_WIDTH = 640;

const qr = await ensureQr();
const momoCard = await ensureMomoCard();
const generated = {
  categories: deriveCategories(pets),
  qr,
  momoCard,
  builtinShortNames: built.filter((p) => p.builtin).map((p) => p.shortName),
  pets: built,
};
writeFileSync(DATA_FILE, `${JSON.stringify(generated, null, 2)}\n`);

await makeIcons();

if (fallbacks.length) {
  console.warn(
    `[prepare-assets] ${fallbacks.length} build-only fallback GIF(s) in use (real pet art is missing):\n  ` +
      fallbacks.map((f) => relative(ROOT, f)).join("\n  "),
  );
} else {
  console.log(
    `[prepare-assets] ${built.length} pets: ${normalised.size} clean sprite clips, posters and Open Graph images ready`,
  );
}
if (!fallbacks.length) {
  console.log(`[prepare-assets] one sprite canvas: ${canvasWidth}x${canvasHeight}px`);
}

// ---------------------------------------------------------------- helpers

/**
 * Static gallery poster: one settled hero frame, cropped to the pet body (no
 * bubble, no detached hearts/props), integer-enlarged and centred on the shared
 * 4:3 canvas. Kept separate from the animation clips so tricks stay uncropped.
 * 2x art stays integer-scaled; 1x art gets a fractional display zoom baked in at
 * build so every pet fills a similar area.
 */
async function galleryPoster(pet, frames, scale, grid) {
  const median = heroBodyHeight(frames);
  const index = settledFrameIndex(frames, median);
  const data = frameView(frames, index);
  const bubble = findBubble(data, frames.width, frames.height);
  const { box: body, mask } = largestBody(data, frames.width, frames.height, bubble);
  if (!body || !mask) throw new Error(`prepare-assets: no body found for gallery poster ${pet.id}`);

  const width = body.width;
  const height = body.height;
  const { zoom, scale: finalScale } = galleryFit(width, height, scale, grid);
  const cropped = Buffer.alloc(width * height * 4);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const sx = body.minX + x;
      const sy = body.minY + y;
      const source = sy * frames.width + sx;
      if (!mask[source]) continue;
      const from = source * 4;
      const target = (y * width + x) * 4;
      cropped[target] = data[from];
      cropped[target + 1] = data[from + 1];
      cropped[target + 2] = data[from + 2];
      cropped[target + 3] = 255;
    }
  }

  const scaledWidth = Math.round(width * finalScale);
  const scaledHeight = Math.round(height * finalScale);
  const scaled = await sharp(cropped, { raw: { width, height, channels: 4 } })
    .resize(scaledWidth, scaledHeight, { kernel: "nearest" })
    .png()
    .toBuffer();
  const rel = `pets/${pet.id}/gallery.png`;
  await sharp({
    create: {
      width: GALLERY_WIDTH,
      height: GALLERY_HEIGHT,
      channels: 4,
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    },
  })
    .composite([
      {
        input: scaled,
        left: Math.round((GALLERY_WIDTH - scaledWidth) / 2),
        top: Math.round((GALLERY_HEIGHT - scaledHeight) / 2),
      },
    ])
    .png({ compressionLevel: 9 })
    .toFile(join(PUBLIC, rel));

  return { src: `/${rel}`, width: GALLERY_WIDTH, height: GALLERY_HEIGHT, zoom };
}

async function normalise(job, width, height) {
  const frames = await readFrames(job.raw.path);

  // Clear greeting bubble, border and detached components (hearts/tail/orbs)
  // from every source frame independently before scaling. The bubble moves
  // between frames and its border is not white, so findBubble+largestBody
  // keeps only the main pet body per frame.
  if (job.greeting) {
    for (let f = 0; f < frames.pages; f++) {
      const data = frameView(frames, f);
      const bubble = findBubble(data, frames.width, frames.height);
      const { mask } = largestBody(data, frames.width, frames.height, bubble);
      if (!mask) continue;
      for (let y = 0; y < frames.height; y++) {
        for (let x = 0; x < frames.width; x++) {
          const offset = (y * frames.width + x) * 4;
          if (isContent(data, offset) && !mask[y * frames.width + x]) {
            data[offset] = 0;
            data[offset + 1] = 0;
            data[offset + 2] = 0;
            data[offset + 3] = 0;
          }
        }
      }
    }
  }

  const scaled = await scaleFrames(
    frames,
    Math.max(1, Math.round(frames.width * job.scale)),
    Math.max(1, Math.round(frames.height * job.scale)),
  );
  const bounds = contentBounds(scaled, { greeting: false });
  const pageBytes = width * height * 4;
  const data = Buffer.alloc(pageBytes * scaled.pages);
  if (bounds) {
    const offsetX = Math.round((width - bounds.width) / 2) - bounds.minX;
    const offsetY = height - CANVAS_MARGIN - bounds.maxY - 1;
    for (let frame = 0; frame < scaled.pages; frame++) {
      const source = frameView(scaled, frame);
      const frameStart = frame * pageBytes;
      for (let y = 0; y < scaled.height; y++) {
        const targetY = y + offsetY;
        if (targetY < 0 || targetY >= height) continue;
        for (let x = 0; x < scaled.width; x++) {
          const targetX = x + offsetX;
          if (targetX < 0 || targetX >= width) continue;
          const sourceOffset = (y * scaled.width + x) * 4;
          if (!isContent(source, sourceOffset)) continue;
          const targetOffset = frameStart + (targetY * width + targetX) * 4;
          data[targetOffset] = source[sourceOffset];
          data[targetOffset + 1] = source[sourceOffset + 1];
          data[targetOffset + 2] = source[sourceOffset + 2];
          data[targetOffset + 3] = 255;
        }
      }
    }
  }

  const canvas = {
    data,
    width,
    height,
    pages: scaled.pages,
    channels: 4,
    stride: 4,
    pageBytes,
    delays: scaled.delays,
  };
  const gifRel = `pets/${job.pet.id}/${job.name}.sprite.gif`;
  const posterRel = `pets/${job.pet.id}/${job.name}.sprite.png`;
  writeFileSync(join(PUBLIC, gifRel), await encodeFrames(canvas));
  await sharp(await posterFrom(canvas)).toFile(join(PUBLIC, posterRel));

  return {
    src: `/${gifRel}`,
    poster: `/${posterRel}`,
    width,
    height,
    durationMs: job.raw.info.durationMs,
    fallback: job.raw.fallback,
  };
}

function rawMedia(pet, name) {
  const rel = `pets/${pet.id}/${name}.gif`;
  const gif = join(PUBLIC, rel);
  if (!existsSync(gif)) {
    mkdirSync(dirname(gif), { recursive: true });
    writeFileSync(gif, makeFallbackGif());
  }

  const bytes = readFileSync(gif);
  const fallback = bytes.toString("latin1").includes("BUILD-ONLY FALLBACK ART");
  if (fallback) fallbacks.push(gif);
  return { path: gif, info: readGifInfo(bytes), fallback };
}

function validate(list) {
  const seen = new Set();
  for (const pet of list) {
    const where = `pets.json: pet "${pet.id}"`;
    if (!ID.test(pet.id ?? "")) throw new Error(`${where}: id must match ${ID}`);
    if (seen.has(pet.id)) throw new Error(`${where}: duplicate id`);
    seen.add(pet.id);
    for (const key of ["name", "author", "version", "description"]) {
      if (typeof pet[key] !== "string" || !pet[key]) throw new Error(`${where}: missing ${key}`);
    }
    if (typeof pet.category !== "string" || !CATEGORY_ID.test(pet.category)) {
      throw new Error(`${where}: bad category "${pet.category}"`);
    }
    if (!Array.isArray(pet.tricks)) throw new Error(`${where}: tricks must be a list`);
    for (const trick of pet.tricks) {
      if (!ID.test(trick.id ?? "") || trick.id === "hero")
        throw new Error(`${where}: bad trick id "${trick.id}"`);
      if (typeof trick.title !== "string" || !trick.title)
        throw new Error(`${where}: trick "${trick.id}" needs a title`);
    }
    pet.shortName = pet.name.split(" the ")[0];
  }
}

function fileSize(pet) {
  if (pet.file_size) return pet.file_size;
  const module = join(PUBLIC, "files", `${pet.id}.deskpet`);
  return existsSync(module) ? statSync(module).size : null;
}

async function ensureQr() {
  const file = join(PUBLIC, "donate-qr.png");
  const webp = join(PUBLIC, "donate-qr.webp");
  if (!existsSync(file) || !existsSync(webp)) {
    throw new Error("public/donate-qr.png and public/donate-qr.webp are required (no placeholder)");
  }
  const info = await sharp(file).metadata();
  const webpInfo = await sharp(webp).metadata();
  if (!info.width || !info.height) throw new Error("donate-qr.png cannot be read");
  if (webpInfo.width !== info.width || webpInfo.height !== info.height) {
    throw new Error("donate-qr.webp must have the same dimensions as donate-qr.png");
  }
  return {
    src: "/donate-qr.png",
    webp: "/donate-qr.webp",
    width: info.width,
    height: info.height,
  };
}

async function ensureMomoCard() {
  const file = join(PUBLIC, "donate-qr.png");
  const out = join(PUBLIC, "support-momo-card.png");
  const webp = join(PUBLIC, "support-momo-card.webp");

  const logoWidth = 120;
  const logoHeight = Math.round(
    (MOMO_CARD_CROPS.logo.height * logoWidth) / MOMO_CARD_CROPS.logo.width,
  );
  const logo = await sharp(file)
    .extract(MOMO_CARD_CROPS.logo)
    .resize({ width: logoWidth })
    .png()
    .toBuffer();

  const qrWidth = 566;
  const qrHeight = Math.round((MOMO_CARD_CROPS.qr.height * qrWidth) / MOMO_CARD_CROPS.qr.width);
  const qrMask = Buffer.from(
    `<svg width="${qrWidth}" height="${qrHeight}"><rect width="${qrWidth}" height="${qrHeight}" rx="${Math.round((qrWidth * 28) / 436)}" ry="${Math.round((qrWidth * 28) / 436)}" fill="#fff"/></svg>`,
  );
  const qrCard = await sharp(
    await sharp(file)
      .extract(MOMO_CARD_CROPS.qr)
      .resize({ width: qrWidth })
      .ensureAlpha()
      .png()
      .toBuffer(),
  )
    .composite([{ input: qrMask, blend: "dest-in" }])
    .png()
    .toBuffer();

  const payeeWidth = 410;
  const payeeHeight = Math.round(
    (MOMO_CARD_CROPS.payee.height * payeeWidth) / MOMO_CARD_CROPS.payee.width,
  );
  const payee = await sharp(file)
    .extract(MOMO_CARD_CROPS.payee)
    .resize({ width: payeeWidth })
    .png()
    .toBuffer();

  const pad = 8;
  const height = pad + logoHeight + pad + qrHeight + pad + payeeHeight + pad;
  const background = Buffer.from(
    `<svg xmlns="http://www.w3.org/2000/svg" width="${MOMO_CARD_WIDTH}" height="${height}"><defs><linearGradient id="pm" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffec77"/><stop offset="0.4" stop-color="#ffec77"/><stop offset="0.8" stop-color="#fdfad7"/><stop offset="1" stop-color="#fdfad7"/></linearGradient></defs><rect width="${MOMO_CARD_WIDTH}" height="${height}" fill="url(#pm)"/></svg>`,
  );

  await sharp({ create: { width: MOMO_CARD_WIDTH, height, channels: 3, background: "#fdfad7" } })
    .composite([
      { input: background, left: 0, top: 0 },
      { input: logo, left: Math.round((MOMO_CARD_WIDTH - logoWidth) / 2), top: pad },
      {
        input: qrCard,
        left: Math.round((MOMO_CARD_WIDTH - qrWidth) / 2),
        top: pad + logoHeight + pad,
      },
      {
        input: payee,
        left: Math.round((MOMO_CARD_WIDTH - payeeWidth) / 2),
        top: pad + logoHeight + pad + qrHeight + pad,
      },
    ])
    .png()
    .toFile(out);
  await sharp(out).webp({ lossless: true }).toFile(webp);
  return {
    src: "/support-momo-card.png",
    webp: "/support-momo-card.webp",
    width: MOMO_CARD_WIDTH,
    height,
  };
}

async function makeOg(posterPaths, out, label) {
  const many = posterPaths.length > 1;
  const size = many ? 300 : 400;
  const gap = many ? 40 : 0;
  const totalWidth = posterPaths.length * size + (posterPaths.length - 1) * gap;
  const startX = Math.round((1200 - totalWidth) / 2);
  const y = Math.round((630 - size) / 2) + 14;

  const frames = posterPaths
    .map((_, index) => {
      const x = startX + index * (size + gap);
      return `<rect x="${x}" y="${y}" width="${size}" height="${size}" rx="18" fill="${palette.surface}" stroke="${palette.ink}" stroke-width="6"/>`;
    })
    .join("");

  const background = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630">
    <defs>
      <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="${palette.canvas}"/><stop offset="0.55" stop-color="${palette.frame}"/><stop offset="1" stop-color="${palette.accent}"/>
      </linearGradient>
      <pattern id="dots" width="30" height="30" patternUnits="userSpaceOnUse">
        <circle cx="2" cy="2" r="2" fill="${palette.brand}" opacity="0.22"/>
      </pattern>
    </defs>
    <rect width="1200" height="630" fill="url(#g)"/>
    <rect width="1200" height="630" fill="url(#dots)"/>
    <circle cx="1030" cy="90" r="150" fill="${palette.accent}" opacity="0.35"/>
    <circle cx="140" cy="560" r="170" fill="${palette.highlight}" opacity="0.3"/>
    ${frames}
    <text x="600" y="74" font-family="system-ui, Segoe UI, sans-serif" font-size="40" font-weight="700"
      fill="${palette.ink}" text-anchor="middle" letter-spacing="1">DeskPet</text>
    <text x="600" y="596" font-family="system-ui, Segoe UI, sans-serif" font-size="24"
      fill="${palette.muted}" text-anchor="middle">${escapeXml(label)} · tiny pixel pets for your desktop</text>
  </svg>`;

  const layers = [];
  for (const [index, poster] of posterPaths.entries()) {
    const x = startX + index * (size + gap) + 18;
    const inner = size - 36;
    const image = await sharp(poster)
      .resize(inner, inner, {
        fit: "contain",
        kernel: "nearest",
        background: hexToRgb(palette.frame),
      })
      .png()
      .toBuffer();
    const meta = await sharp(image).metadata();
    layers.push({
      input: image,
      left: x + Math.round((inner - meta.width) / 2),
      top: y + 18 + Math.round((inner - meta.height) / 2),
    });
  }

  await sharp(Buffer.from(background)).composite(layers).png({ compressionLevel: 9 }).toFile(out);
}

/**
 * Square favicon/app icons from Mochi's hero poster. The poster is mostly empty
 * canvas, so a browser tab would shrink the pet to a few pixels. Crop to the
 * opaque bbox, pad to a square with a margin and scale nearest-neighbour so the
 * pixel art stays crisp at every size.
 */
async function makeIcons() {
  const mochi = built.find((p) => p.id === "mochi");
  if (!mochi?.hero?.poster) {
    throw new Error("prepare-assets: mochi hero poster is required for icons");
  }

  const poster = join(PUBLIC, mochi.hero.poster.slice(1));
  const { data, info } = await sharp(poster)
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  const box = opaqueBox(data, info.width, info.height);
  if (!box) throw new Error(`prepare-assets: ${mochi.hero.poster} has no opaque pixels`);
  const body = await sharp(poster).extract(box).png().toBuffer();

  const clear = { r: 0, g: 0, b: 0, alpha: 0 };
  const brand = hexToRgb(palette.canvas);
  const sizes = [16, 32, 48];
  const favicons = [];
  for (const size of sizes) {
    const png = await iconPng(body, box, size, 0.06, clear, false);
    writeFileSync(join(PUBLIC, `favicon-${size}.png`), png);
    favicons.push(png);
  }
  for (const [name, size] of [
    ["apple-touch-icon.png", 180],
    ["icon-512.png", 512],
  ]) {
    const png = await iconPng(body, box, size, 0.12, brand, true);
    writeFileSync(join(PUBLIC, name), png);
  }

  // PNG-in-ICO container; no extra dependency needed for three small entries.
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(sizes.length, 4);
  let offset = 6 + sizes.length * 16;
  const entries = favicons.map((png, index) => {
    const entry = Buffer.alloc(16);
    entry[0] = sizes[index];
    entry[1] = sizes[index];
    entry[2] = 0;
    entry[3] = 0;
    entry.writeUInt16LE(1, 4);
    entry.writeUInt16LE(32, 6);
    entry.writeUInt32LE(png.length, 8);
    entry.writeUInt32LE(offset, 12);
    offset += png.length;
    return entry;
  });
  writeFileSync(join(PUBLIC, "favicon.ico"), Buffer.concat([header, ...entries, ...favicons]));

  const manifest = {
    name: "DeskPet",
    short_name: "DeskPet",
    icons: [
      { src: "/apple-touch-icon.png", sizes: "180x180", type: "image/png" },
      { src: "/icon-512.png", sizes: "512x512", type: "image/png" },
    ],
    theme_color: palette.canvas,
    background_color: palette.canvas,
  };
  writeFileSync(join(PUBLIC, "site.webmanifest"), `${JSON.stringify(manifest, null, 2)}\n`);

  console.log(
    "[prepare-assets] icons: favicon 16/32/48, apple-touch 180, icon 512, favicon.ico, site.webmanifest",
  );
}

/** Bounding box of pixels with alpha > 16, or null when the image is empty. */
function opaqueBox(data, width, height) {
  let minX = width;
  let minY = height;
  let maxX = -1;
  let maxY = -1;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      if (data[(y * width + x) * 4 + 3] <= 16) continue;
      if (x < minX) minX = x;
      if (x > maxX) maxX = x;
      if (y < minY) minY = y;
      if (y > maxY) maxY = y;
    }
  }
  if (maxX < 0) return null;
  return { left: minX, top: minY, width: maxX - minX + 1, height: maxY - minY + 1 };
}

/** Fit the body crop into a square SxS canvas at a margin, nearest-neighbour. */
async function iconPng(body, box, size, marginRatio, background, opaque) {
  const margin = Math.round(size * marginRatio);
  const inner = Math.max(1, size - 2 * margin);
  const maxDim = Math.max(box.width, box.height);
  const scaledWidth = Math.max(1, Math.round((box.width * inner) / maxDim));
  const scaledHeight = Math.max(1, Math.round((box.height * inner) / maxDim));
  const scaled = await sharp(body)
    .resize(scaledWidth, scaledHeight, { kernel: "nearest" })
    .png()
    .toBuffer();

  let canvas = sharp({ create: { width: size, height: size, channels: 4, background } }).composite([
    {
      input: scaled,
      left: Math.round((size - scaledWidth) / 2),
      top: Math.round((size - scaledHeight) / 2),
    },
  ]);
  if (opaque) canvas = canvas.flatten({ background: palette.canvas });
  return canvas.png().toBuffer();
}

function escapeXml(value) {
  return String(value).replace(
    /[&<>"']/g,
    (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&apos;" })[c],
  );
}
