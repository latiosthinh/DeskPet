// Reads the theme colours from src/styles/global.css so build-time artwork
// (Open Graph images, placeholder QR, contact sheets) uses the same palette as the site.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
const TOKENS = join(ROOT, "src", "styles", "global.css");

const NAMES = {
  canvas: "--canvas",
  surface: "--surface",
  surface2: "--surface-2",
  ink: "--ink",
  muted: "--muted",
  line: "--line",
  brand: "--brand",
  brandStrong: "--brand-strong",
  brandInk: "--brand-ink",
  accent: "--accent",
  highlight: "--highlight",
  badgeBg: "--badge-bg",
  badgeInk: "--badge-ink",
  frame: "--frame",
  focus: "--focus",
};

function readTokens(file, blockPattern, missingBlock) {
  const css = readFileSync(file, "utf8");
  const block = css.match(blockPattern);
  if (!block) throw new Error(missingBlock);

  const values = new Map(
    [...block[1].matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)].map((match) => [
      match[1],
      match[2].trim(),
    ]),
  );

  return Object.fromEntries(
    Object.entries(NAMES).map(([key, token]) => {
      const value = values.get(token);
      if (!value) throw new Error(`${file}: missing ${token}`);
      return [key, value];
    }),
  );
}

export function lightPalette(file = TOKENS) {
  return readTokens(file, /:root\s*\{([\s\S]*?)\n\}/, `${file}: no :root token block`);
}

export function darkPalette(file = TOKENS) {
  return readTokens(file, /\.dark\s*\{([\s\S]*?)\n\}/, `${file}: no .dark token block`);
}

export function hexToRgb(hex) {
  const value = hex.replace("#", "");
  return {
    r: Number.parseInt(value.slice(0, 2), 16),
    g: Number.parseInt(value.slice(2, 4), 16),
    b: Number.parseInt(value.slice(4, 6), 16),
    alpha: 1,
  };
}
