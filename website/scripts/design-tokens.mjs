#!/usr/bin/env node
// Design-token gate for the website. Every colour, radius, spacing step, type size and
// shadow lives in src/styles/global.css. Components may only use the semantic classes and
// token variables defined there.
//
// Rejected outside src/styles/global.css:
//   * raw CSS colour literals (#rrggbb, rgb(), hsl(), oklch(), color-mix() ...)
//   * arbitrary Tailwind values (p-[13px], text-[#fff], rounded-[10px] ...)
//   * default Tailwind palette colours (bg-white, text-slate-500, border-red-300 ...)
//   * default or arbitrary Tailwind radii (rounded-lg, rounded-full, rounded-[8px])
//   * default or arbitrary Tailwind spacing (p-4, mt-2, gap-6, p-[1rem] ...)
//   * inline style attributes
//
// Intrinsic SVG attributes (viewBox, width, height), image width/height attributes,
// numbers in tests and generated data are not CSS values, so they are not scanned.

import { readFileSync, readdirSync, statSync } from "node:fs";
import { extname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(fileURLToPath(new URL(".", import.meta.url)), "..");
const SRC = join(ROOT, "src");
const TOKENS = join(SRC, "styles", "global.css");
const SCANNED = new Set([".astro", ".ts", ".css", ".html"]);
const REQUIRED_TOKENS = [
  "--canvas",
  "--surface",
  "--ink",
  "--muted",
  "--line",
  "--brand",
  "--accent",
  "--frame",
  "--focus",
  "--radius-card",
  "--radius-control",
  "--radius-mini",
  "--spacing-section",
  "--spacing-grid",
  "--text-display",
  "--text-title",
  "--text-heading",
  "--text-body",
  "--text-small",
  "--elevation-soft",
  "--elevation-lift",
];

const RULES = [
  {
    id: "raw-color",
    pattern:
      /#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3,4})\b|\b(?:rgba?|hsla?|oklch|oklab|lch|lab|color-mix)\(/g,
  },
  {
    id: "arbitrary-value",
    pattern: /\b[a-z][a-z0-9-]*-\[[^\]]+\]/g,
  },
  {
    id: "default-color",
    pattern:
      /\b(?:bg|text|border|ring|outline|divide|decoration|accent|caret|fill|stroke|from|via|to)-(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose|black|white)(?:-\d{1,3})?(?:\/\d{1,3})?\b/g,
  },
  {
    id: "default-radius",
    pattern: /(?<![\w-])rounded(?:-(?:none|sm|md|lg|xl|2xl|3xl|full))?(?![\w-])/g,
  },
  {
    id: "default-spacing",
    pattern:
      /\b(?:p|px|py|pt|pr|pb|pl|m|mx|my|mt|mr|mb|ml|gap|gap-x|gap-y|space-x|space-y|inset|inset-x|inset-y|top|right|bottom|left)-(?:\d+(?:\.\d+)?)\b/g,
  },
  {
    id: "raw-radius",
    pattern:
      /(?:^|[^-\w])(?:border(?:-(?:top|bottom|start|end)(?:-(?:left|right|start|end))?)?-)?radius(?:-[a-z-]+)?\s*:\s*(?=\S)(?!var\()/g,
  },
  {
    id: "raw-spacing",
    pattern:
      /\b(?:margin|padding|gap)(?:-[a-z-]+)?\s*:\s*[^;{}]*-?\d*\.?\d+(?:px|rem|em|ex|ch|vh|vw|vmin|vmax|%|pt|pc|cm|mm|in)\b/gi,
  },
  {
    id: "raw-named-color",
    pattern:
      /(?:^|[;{\s])(?:background|border|outline|color|fill|stroke|box-shadow|text-shadow)(?:-[a-z-]+)?\s*:\s*[^;{}]*\b(?:transparent|currentcolor|white|black|red|orange|yellow|lime|green|teal|cyan|blue|navy|purple|fuchsia|magenta|pink|rose|brown|gray|grey|silver|maroon|olive|aqua)\b/gi,
  },
  {
    id: "inline-style",
    pattern: /\bstyle\s*=\s*[{"']/g,
  },
];

function linesOf(source) {
  return source.split(/\r?\n/);
}

/** Returns every violation in a single source file. Exported for unit tests. */
export function scanSource(source, file = "source") {
  const violations = [];
  const lines = linesOf(source);
  for (const [index, line] of lines.entries()) {
    for (const rule of RULES) {
      rule.pattern.lastIndex = 0;
      for (const match of line.matchAll(rule.pattern)) {
        // Anchor/hash links such as #main or #author-deskpet are not colours.
        if (rule.id === "raw-color" && /(?:href|id|aria-labelledby)=\s*["']?#/.test(line)) {
          const tail = line.slice(Math.max(0, match.index - 12), match.index);
          if (/(?:href|id|aria-labelledby)=["']?$/.test(tail)) continue;
        }
        violations.push({
          file,
          line: index + 1,
          rule: rule.id,
          text: match[0],
          source: line.trim().slice(0, 120),
        });
      }
    }
  }
  return violations;
}

/** Shared token sheet may define colours but must expose the required token names. */
export function checkTokenSheet(css) {
  return REQUIRED_TOKENS.filter((token) => !new RegExp(`${token}\\s*:`).test(css)).map((token) => ({
    file: relative(ROOT, TOKENS),
    line: 0,
    rule: "missing-token",
    text: token,
    source: `global.css must define ${token}`,
  }));
}

export function collectFiles(dir = SRC, out = []) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      collectFiles(full, out);
    } else if (SCANNED.has(extname(entry)) && full !== TOKENS) {
      out.push(full);
    }
  }
  return out;
}

function main() {
  const violations = checkTokenSheet(readFileSync(TOKENS, "utf8"));
  for (const file of collectFiles()) {
    violations.push(...scanSource(readFileSync(file, "utf8"), relative(ROOT, file)));
  }

  if (violations.length) {
    console.error("Design-token lint failed:\n");
    for (const violation of violations) {
      console.error(
        `  ${violation.file}:${violation.line} [${violation.rule}] ${violation.text}\n    ${violation.source}`,
      );
    }
    console.error(
      "\nUse the semantic classes and token variables from src/styles/global.css only.",
    );
    process.exit(1);
  }
  console.log(`Design-token lint passed (${collectFiles().length} source files).`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) main();
