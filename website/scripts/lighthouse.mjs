#!/usr/bin/env node
// Lighthouse (mobile, default throttling) on the home page and an add-on pet page.
// Reports are written to qa/lighthouse/, which is git-ignored.

import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { chromium } from "@playwright/test";
import * as chromeLauncher from "chrome-launcher";
import lighthouse from "lighthouse";
import { ROOT, initLocalD1, startPages, testToken } from "./lib/server.mjs";

const TARGETS = [
  { name: "home", path: "/" },
  { name: "pet-builtin", path: "/pets/mochi/" },
  { name: "pet-addon", path: "/pets/ember/" },
  { name: "catalogue", path: "/pets/" },
  { name: "category-hero", path: "/pets/category/hero/" },
  { name: "author-deskpet", path: "/authors/deskpet/" },
];
const THRESHOLDS = { performance: 100, accessibility: 100, "best-practices": 100, seo: 100 };
const OUT = join(ROOT, "qa", "lighthouse");
const PORT = Number(process.env.LIGHTHOUSE_PORT || 8790);

mkdirSync(OUT, { recursive: true });
initLocalD1();
const server = await startPages({ port: PORT, token: testToken() });
const chrome = await chromeLauncher.launch({
  chromePath: chromium.executablePath(),
  chromeFlags: ["--headless=new", "--disable-gpu", "--no-sandbox"],
});

const failures = [];
try {
  for (const target of TARGETS) {
    const url = `${server.url}${target.path}`;
    const result = await lighthouse(url, {
      port: chrome.port,
      output: ["html", "json"],
      logLevel: "error",
      onlyCategories: Object.keys(THRESHOLDS),
    });
    if (!result) throw new Error(`Lighthouse returned no result for ${url}`);
    const [html, json] = result.report;
    writeFileSync(join(OUT, `${target.name}.report.html`), html);
    writeFileSync(join(OUT, `${target.name}.report.json`), json);

    const lhr = result.lhr;
    const scores = {};
    for (const [category, minimum] of Object.entries(THRESHOLDS)) {
      const score = Math.round((lhr.categories[category]?.score ?? 0) * 100);
      scores[category] = score;
      if (score < minimum) failures.push(`${target.name} ${category}: ${score} < ${minimum}`);
    }
    const cls = lhr.audits["cumulative-layout-shift"]?.numericValue ?? Number.NaN;
    if (cls !== 0) failures.push(`${target.name} CLS: ${cls} is not exactly 0`);
    const lcp = lhr.audits["largest-contentful-paint"]?.displayValue ?? "?";
    const tbt = lhr.audits["total-blocking-time"]?.displayValue ?? "?";

    console.log(
      `${target.name.padEnd(10)} perf ${String(scores.performance).padStart(3)} | a11y ${String(
        scores.accessibility,
      ).padStart(3)} | best-practices ${String(scores["best-practices"]).padStart(
        3,
      )} | seo ${String(scores.seo).padStart(3)} | CLS ${cls.toFixed(4)} | LCP ${lcp} | TBT ${tbt}`,
    );
  }
} finally {
  await chrome.kill();
  server.stop();
}

if (failures.length) {
  console.error(`\nLighthouse failed:\n  ${failures.join("\n  ")}`);
  process.exit(1);
}
console.log(`\nLighthouse passed. Reports: ${OUT}`);
