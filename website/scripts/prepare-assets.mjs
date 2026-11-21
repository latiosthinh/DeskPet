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
//
// It never overwrites donate-qr.png, raw GIFs or .deskpet modules produced by petshot.

import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";
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

export const CATEGORIES = [
  { id: "critter", title: "Critters", blurb: "Small, soft and always up to something." },
  { id: "hero", title: "Heroes", blurb: "Tiny adventurers with big plans." },
  { id: "legend", title: "Legends", blurb: "Detailed, dramatic and a little bit extra." },
];

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
  const gallery = await galleryPoster(pet, heroFrames, heroScale.get(pet.id).scale);

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

const qr = await ensureQr();
const generated = {
  categories: CATEGORIES,
  qr,
  builtinShortNames: built.filter((p) => p.builtin).map((p) => p.shortName),
  pets: built,
};
writeFileSync(DATA_FILE, `${JSON.stringify(generated, null, 2)}\n`);

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
 */
async function galleryPoster(pet, frames, scale) {
  const median = heroBodyHeight(frames);
  const index = settledFrameIndex(frames, median);
  const data = frameView(frames, index);
  const bubble = findBubble(data, frames.width, frames.height);
  const { box: body, mask } = largestBody(data, frames.width, frames.height, bubble);
  if (!body || !mask) throw new Error(`prepare-assets: no body found for gallery poster ${pet.id}`);

  const width = body.width;
  const height = body.height;
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

  const maxScale = Math.max(
    1,
    Math.floor(
      Math.min(
        (GALLERY_WIDTH - CANVAS_MARGIN * 2) / width,
        (GALLERY_HEIGHT - CANVAS_MARGIN * 2) / height,
      ),
    ),
  );
  const galleryScale = Math.min(scale, maxScale);
  const scaledWidth = width * galleryScale;
  const scaledHeight = height * galleryScale;
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

  return { src: `/${rel}`, width: GALLERY_WIDTH, height: GALLERY_HEIGHT };
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
    if (!CATEGORIES.some((category) => category.id === pet.category)) {
      throw new Error(`${where}: unknown category "${pet.category}"`);
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
  if (!existsSync(file)) {
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="360" height="360">
      <rect width="360" height="360" fill="${palette.surface}"/>
      <g fill="${palette.ink}">
        <rect x="24" y="24" width="84" height="84"/><rect x="252" y="24" width="84" height="84"/>
        <rect x="24" y="252" width="84" height="84"/><rect x="48" y="48" width="36" height="36" fill="${palette.surface}"/>
        <rect x="276" y="48" width="36" height="36" fill="${palette.surface}"/><rect x="48" y="276" width="36" height="36" fill="${palette.surface}"/>
      </g>
      <text x="180" y="196" font-family="monospace" font-size="22" text-anchor="middle" fill="${palette.ink}">DONATION QR</text>
    </svg>`;
    await sharp(Buffer.from(svg)).png().toFile(file);
    console.warn(
      "[prepare-assets] public/donate-qr.png was missing; wrote a placeholder (never overwritten)",
    );
  }
  const info = await sharp(file).metadata();
  if (!info.width || !info.height) throw new Error("donate-qr.png cannot be read");
  return { src: "/donate-qr.png", width: info.width, height: info.height };
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

function escapeXml(value) {
  return String(value).replace(
    /[&<>"']/g,
    (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&apos;" })[c],
  );
}
