#!/usr/bin/env node
// Verifies the Cloudflare Pages Functions against the real local runtime:
// `wrangler pages dev` + local D1. Uses a test-only STATS_TOKEN from the environment.

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { ROOT, initLocalD1, startPages, testToken } from "./lib/server.mjs";

const WRANGLER = join(ROOT, "node_modules", "wrangler", "bin", "wrangler.js");
const token = testToken();
const pets = JSON.parse(readFileSync(join(ROOT, "pets.json"), "utf8"));
const addon = pets.find((pet) => !pet.builtin);
const builtin = pets.find((pet) => pet.builtin);
let failures = 0;

function check(name, condition, detail = "") {
  if (condition) {
    console.log(`  ok   ${name}`);
  } else {
    failures++;
    console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ""}`);
  }
}

function query() {
  const raw = execFileSync(
    process.execPath,
    [
      WRANGLER,
      "d1",
      "execute",
      "deskpet",
      "--local",
      "--json",
      "--command",
      "SELECT pet_id, kind, SUM(n) AS n FROM counts GROUP BY pet_id, kind",
    ],
    { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
  );
  const parsed = JSON.parse(raw.slice(raw.indexOf("[")));
  const map = new Map();
  for (const row of parsed[0]?.results ?? []) map.set(`${row.pet_id}|${row.kind}`, Number(row.n));
  return map;
}

function delta(before, after, pet, kind) {
  return (after.get(`${pet}|${kind}`) ?? 0) - (before.get(`${pet}|${kind}`) ?? 0);
}

async function request(url, options) {
  const response = await fetch(url, options);
  const body = await response.text();
  return { status: response.status, headers: response.headers, body };
}

async function requestBytes(url, options) {
  const response = await fetch(url, options);
  const bytes = Buffer.from(await response.arrayBuffer());
  return { status: response.status, headers: response.headers, bytes };
}

console.log(
  `Backend check: wrangler pages dev + local D1 (token from ${process.env.STATS_TOKEN ? "process.env" : "test default"})`,
);
initLocalD1();
const before = query();
const server = await startPages({ token });

try {
  const home = await request(`${server.url}/`);
  check(
    "GET / serves the built site",
    home.status === 200 && home.body.includes("Tiny pixel pets"),
  );

  const petPage = await request(`${server.url}/pets/${addon.id}/`);
  check(
    `GET /pets/${addon.id}/ serves the pet page`,
    petPage.status === 200 && petPage.body.includes(`/download/${addon.id}`),
  );

  const update = await request(`${server.url}/api/update/${addon.id}`);
  check(
    `GET /api/update/${addon.id} -> version json`,
    update.status === 200 && JSON.parse(update.body).version === addon.version,
    update.body,
  );
  check(
    "update response is not cached",
    (update.headers.get("cache-control") || "").includes("no-store"),
  );

  const updateUnknown = await request(`${server.url}/api/update/not-a-pet`);
  check("GET /api/update/<unknown> -> 404", updateUnknown.status === 404);

  const visit = await request(`${server.url}/api/visit/${addon.id}`, { method: "POST" });
  check(`POST /api/visit/${addon.id} -> 204`, visit.status === 204);

  const visitUnknown = await request(`${server.url}/api/visit/not-a-pet`, { method: "POST" });
  check("POST /api/visit/<unknown> -> 404", visitUnknown.status === 404);

  const download = await requestBytes(`${server.url}/download/${addon.id}`);
  const diskModule = readFileSync(server.fixture.path);
  check(
    `GET /download/${addon.id} -> exact module bytes`,
    download.status === 200 &&
      (download.headers.get("content-disposition") || "").includes(`${addon.id}.deskpet`) &&
      download.bytes.equals(diskModule),
    `status=${download.status} sent=${download.bytes.length} disk=${diskModule.length}`,
  );

  const builtinDownload = await request(`${server.url}/download/${builtin.id}`);
  check("GET /download/<builtin> -> 404", builtinDownload.status === 404);

  const direct = await request(`${server.url}/files/${addon.id}.deskpet`);
  check(`GET /files/${addon.id}.deskpet -> 404`, direct.status === 404);

  const statsNoToken = await request(`${server.url}/stats`);
  check("GET /stats without token -> 404", statsNoToken.status === 404);
  const statsWrong = await request(`${server.url}/stats?token=wrong-${Date.now()}`);
  check("GET /stats with wrong token -> 404", statsWrong.status === 404);
  const stats = await request(`${server.url}/stats?token=${encodeURIComponent(token)}`);
  check(
    "GET /stats with token -> counts page",
    stats.status === 200 && stats.body.includes(addon.name) && stats.body.includes("Downloads"),
  );
} finally {
  server.stop();
  await new Promise((resolve) => setTimeout(resolve, 1500));
}

if (server.fixture.owned) {
  check("test fixture was removed on stop", !existsSync(server.fixture.path));
} else {
  check(
    "pre-existing module is untouched",
    readFileSync(server.fixture.path).equals(server.fixture.bytes),
  );
}

const after = query();
check(
  "visit counter incremented once",
  delta(before, after, addon.id, "visit") === 1,
  `delta=${delta(before, after, addon.id, "visit")}`,
);
check(
  "update counter incremented once",
  delta(before, after, addon.id, "update") === 1,
  `delta=${delta(before, after, addon.id, "update")}`,
);
check(
  "download counter incremented once",
  delta(before, after, addon.id, "download") === 1,
  `delta=${delta(before, after, addon.id, "download")}`,
);
check("unknown pet was not counted", delta(before, after, "not-a-pet", "visit") === 0);

if (failures) {
  console.error(`\nBackend check failed: ${failures} problem(s)`);
  process.exit(1);
}
console.log(
  "\nBackend check passed: visits, downloads, updates, /files 404 and /stats token all verified.",
);
