#!/usr/bin/env node
// Builds full-page contact sheets from the Playwright screenshots, one per viewport
// and theme. Uses the installed sharp; safe to run more than once.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import sharp from "sharp";
import { lightPalette } from "./lib/palette.mjs";

const ROOT = process.cwd();
const SHOTS = join(ROOT, "qa", "screenshots");
const PAGES = ["home", "pet-builtin", "pet-addon", "contribute", "not-found", "support"];
const palette = lightPalette();

const SHEETS = [
  { name: "contact-desktop-1440", viewport: 1440, theme: "light", width: 380 },
  { name: "contact-desktop-1440-dark", viewport: 1440, theme: "dark", width: 380 },
  { name: "contact-mobile-375", viewport: 375, theme: "light", width: 250 },
  { name: "contact-mobile-375-dark", viewport: 375, theme: "dark", width: 250 },
];

async function makeSheet({ name, viewport, theme, width }) {
  const files = PAGES.map((page) => join(SHOTS, `${page}-${viewport}-${theme}.png`));
  const missing = files.filter((file) => !existsSync(file));
  if (missing.length) {
    console.warn(`[contact-sheet] skipping ${name}; missing:\n  ${missing.join("\n  ")}`);
    return;
  }

  const cols = 3;
  const gap = 20;
  const pad = 24;
  const label = 28;
  const previews = [];
  for (const [index, file] of files.entries()) {
    const image = await sharp(file)
      .resize({ width, withoutEnlargement: true })
      .toBuffer({ resolveWithObject: true });
    previews.push({ page: PAGES[index], ...image });
  }

  const rows = [];
  for (let index = 0; index < previews.length; index += cols) {
    const row = previews.slice(index, index + cols);
    rows.push({ items: row, height: Math.max(...row.map((item) => item.info.height)) });
  }

  const sheetWidth = pad * 2 + cols * width + (cols - 1) * gap;
  const sheetHeight =
    pad * 2 + rows.reduce((total, row) => total + label + row.height, 0) + (rows.length - 1) * gap;

  const layers = [];
  let y = pad;
  for (const row of rows) {
    for (const [column, item] of row.items.entries()) {
      const x = pad + column * (width + gap);
      const caption = Buffer.from(
        `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${label}">
          <text x="0" y="20" font-family="system-ui, Segoe UI, sans-serif" font-size="15" font-weight="700" fill="${palette.ink}">${item.page}</text>
        </svg>`,
      );
      layers.push({ input: caption, left: x, top: y });
      layers.push({ input: item.data, left: x, top: y + label });
    }
    y += label + row.height + gap;
  }

  await sharp({
    create: { width: sheetWidth, height: sheetHeight, channels: 3, background: palette.canvas },
  })
    .composite(layers)
    .png({ compressionLevel: 9 })
    .toFile(join(ROOT, "qa", `${name}.png`));

  console.log(`[contact-sheet] wrote qa/${name}.png (${sheetWidth}x${sheetHeight})`);
}

for (const sheet of SHEETS) await makeSheet(sheet);

// All 15 pet pages, one overview sheet per viewport/theme (60 views in total).
const PET_SHOTS = join(ROOT, "qa", "pet-pages");
const { pets } = JSON.parse(readFileSync(join(ROOT, "src", "data", "pets.generated.json"), "utf8"));

async function makePetSheet({ viewport, theme, width }) {
  const name = `pets-${viewport}-${theme}`;
  const files = pets.map((pet) => ({
    pet,
    file: join(PET_SHOTS, `${pet.id}-${viewport}-${theme}.png`),
  }));
  const missing = files.filter(({ file }) => !existsSync(file));
  if (missing.length) {
    console.warn(
      `[contact-sheet] skipping ${name}; missing:\n  ${missing.map((item) => item.file).join("\n  ")}`,
    );
    return;
  }

  const cols = 5;
  const gap = 16;
  const pad = 24;
  const label = 26;
  const previews = [];
  for (const item of files) {
    const image = await sharp(item.file)
      .resize({ width, withoutEnlargement: true })
      .toBuffer({ resolveWithObject: true });
    previews.push({ ...item, ...image });
  }

  const rows = [];
  for (let index = 0; index < previews.length; index += cols) {
    const row = previews.slice(index, index + cols);
    rows.push({ items: row, height: Math.max(...row.map((item) => item.info.height)) });
  }

  const sheetWidth = pad * 2 + cols * width + (cols - 1) * gap;
  const sheetHeight =
    pad * 2 + rows.reduce((total, row) => total + label + row.height, 0) + (rows.length - 1) * gap;

  const layers = [];
  let y = pad;
  for (const row of rows) {
    for (const [column, item] of row.items.entries()) {
      const x = pad + column * (width + gap);
      const caption = Buffer.from(
        `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${label}">
          <text x="0" y="18" font-family="system-ui, Segoe UI, sans-serif" font-size="14" font-weight="700" fill="${palette.ink}">${item.pet.shortName} (${item.pet.id})</text>
        </svg>`,
      );
      layers.push({ input: caption, left: x, top: y });
      layers.push({ input: item.data, left: x, top: y + label });
    }
    y += label + row.height + gap;
  }

  await sharp({
    create: { width: sheetWidth, height: sheetHeight, channels: 3, background: palette.canvas },
  })
    .composite(layers)
    .png({ compressionLevel: 9 })
    .toFile(join(ROOT, "qa", `${name}.png`));

  console.log(`[contact-sheet] wrote qa/${name}.png (${sheetWidth}x${sheetHeight})`);
}

for (const theme of ["light", "dark"]) {
  for (const viewport of [375, 1440]) {
    await makePetSheet({ viewport, theme, width: viewport === 375 ? 160 : 220 });
  }
}
