#!/usr/bin/env node
// Build-time asset step. Runs before `astro dev` / `astro build`.
//
//   * reads pets.json (source of truth, written by petshot)
//   * keeps the real GIFs under public/pets/ when they exist
//   * creates small, clearly marked build-only fallback art when they do not, so a fresh
//     checkout (CI) can build without the generated players
//   * writes a still PNG poster next to every GIF, plus one Open Graph image per pet
//   * writes src/data/pets.generated.json: true sizes, durations and poster paths
//
// It never overwrites donate-qr.png, GIFs or .deskpet modules produced by petshot.

import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";
import { makeFallbackGif, readGifInfo } from "./lib/gif.mjs";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const PUBLIC = join(ROOT, "public");
const PETS_FILE = join(ROOT, "pets.json");
const DATA_FILE = join(ROOT, "src", "data", "pets.generated.json");
const ID = /^[a-z0-9][a-z0-9-]{0,40}$/;

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

const built = [];
for (const pet of pets) {
  const dir = join(PUBLIC, "pets", pet.id);
  mkdirSync(dir, { recursive: true });

  const hero = await media(pet, "hero");
  const tricks = [];
  for (const trick of pet.tricks) tricks.push({ ...trick, ...(await media(pet, trick.id)) });

  const marker = join(dir, "FALLBACK-ART.txt");
  const petFallbacks = [hero, ...tricks].filter((item) => item.fallback).map((item) => item.src);
  if (petFallbacks.length) {
    writeFileSync(
      marker,
      `Build-only fallback art for ${pet.id}. The real GIFs were not present in this checkout.\n` +
        `Run petshot site before deploying. Placeholder files:\n${petFallbacks.join("\n")}\n`,
    );
  } else if (existsSync(marker)) {
    rmSync(marker);
  }

  const ogRel = `pets/${pet.id}/og.png`;
  await makeOg([join(PUBLIC, hero.poster.slice(1))], join(PUBLIC, ogRel), pet.shortName);

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
    hero,
    og: `/${ogRel}`,
    tricks,
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
    `[prepare-assets] ${built.length} pets: real GIFs, posters and Open Graph images ready`,
  );
}

// ---------------------------------------------------------------- helpers

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

async function media(pet, name) {
  const rel = `pets/${pet.id}/${name}.gif`;
  const gif = join(PUBLIC, rel);
  if (!existsSync(gif)) {
    mkdirSync(dirname(gif), { recursive: true });
    writeFileSync(gif, makeFallbackGif());
  }

  const bytes = readFileSync(gif);
  const fallback = bytes.toString("latin1").includes("BUILD-ONLY FALLBACK ART");
  if (fallback) fallbacks.push(gif);
  const info = readGifInfo(bytes);
  const posterRel = `pets/${pet.id}/${name}.png`;
  const poster = join(PUBLIC, posterRel);
  await sharp(gif, { page: 0 }).png({ compressionLevel: 9 }).toFile(poster);

  return {
    src: `/${rel}`,
    poster: `/${posterRel}`,
    width: info.width,
    height: info.height,
    durationMs: info.durationMs,
    fallback,
  };
}

async function ensureQr() {
  const file = join(PUBLIC, "donate-qr.png");
  if (!existsSync(file)) {
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="360" height="360">
      <rect width="360" height="360" fill="#fff"/>
      <g fill="#241d3a">
        <rect x="24" y="24" width="84" height="84"/><rect x="252" y="24" width="84" height="84"/>
        <rect x="24" y="252" width="84" height="84"/><rect x="48" y="48" width="36" height="36" fill="#fff"/>
        <rect x="276" y="48" width="36" height="36" fill="#fff"/><rect x="48" y="276" width="36" height="36" fill="#fff"/>
      </g>
      <text x="180" y="196" font-family="monospace" font-size="22" text-anchor="middle" fill="#241d3a">DONATION QR</text>
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
      return `<rect x="${x}" y="${y}" width="${size}" height="${size}" rx="18" fill="#f3efff" stroke="#0b0718" stroke-width="6"/>`;
    })
    .join("");

  const background = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630">
    <defs>
      <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#1b1140"/><stop offset="0.55" stop-color="#312e81"/><stop offset="1" stop-color="#4c1d95"/>
      </linearGradient>
      <pattern id="dots" width="30" height="30" patternUnits="userSpaceOnUse">
        <circle cx="2" cy="2" r="2" fill="#a78bfa" opacity="0.22"/>
      </pattern>
    </defs>
    <rect width="1200" height="630" fill="url(#g)"/>
    <rect width="1200" height="630" fill="url(#dots)"/>
    <circle cx="1030" cy="90" r="150" fill="#7c3aed" opacity="0.28"/>
    <circle cx="140" cy="560" r="170" fill="#a78bfa" opacity="0.16"/>
    ${frames}
    <text x="600" y="74" font-family="system-ui, Segoe UI, sans-serif" font-size="40" font-weight="700"
      fill="#efeaf9" text-anchor="middle" letter-spacing="1">DeskPet</text>
    <text x="600" y="596" font-family="system-ui, Segoe UI, sans-serif" font-size="24"
      fill="#d8ccff" text-anchor="middle">${escapeXml(label)} · tiny pixel pets for your desktop</text>
  </svg>`;

  const layers = [];
  for (const [index, poster] of posterPaths.entries()) {
    const x = startX + index * (size + gap) + 18;
    const inner = size - 36;
    const image = await sharp(poster)
      .resize(inner, inner, {
        fit: "contain",
        kernel: "nearest",
        background: { r: 243, g: 239, b: 255, alpha: 1 },
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
