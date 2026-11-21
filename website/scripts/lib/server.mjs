// Starts the real Cloudflare Pages runtime (`wrangler pages dev`) with the local D1
// database for tests, QA and Lighthouse. Test-only values come from the environment.

import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const ROOT = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
export const DIST = join(ROOT, "dist");
const PETS_FILE = join(ROOT, "pets.json");
const WRANGLER = join(ROOT, "node_modules", "wrangler", "bin", "wrangler.js");
const FIXTURE_MARKER = "TEST-ONLY FIXTURE for";

export function testToken() {
  return process.env.STATS_TOKEN || "deskpet-test-only-token";
}

function npxWrangler(args, options = {}) {
  return execFileSync(process.execPath, [WRANGLER, ...args], {
    cwd: ROOT,
    stdio: options.quiet ? "ignore" : "inherit",
    ...options,
  });
}

export function initLocalD1() {
  npxWrangler(["d1", "execute", "deskpet", "--local", "--file=schema.sql"], { quiet: true });
}

/**
 * Fresh checkouts do not have the .deskpet modules (petshot writes them). Tests need one
 * real file to exercise /download; write a clearly marked fixture for the first add-on pet
 * from pets.json into dist/ only. Never touch public/ or a real module.
 *
 * Returns the module that now exists on disk, whether the test owns it, and remove(), which
 * deletes only a fixture the test owns.
 */
export function ensureDownloadFixture({ distDir = DIST, petsFile = PETS_FILE } = {}) {
  const pets = JSON.parse(readFileSync(petsFile, "utf8"));
  const pet = pets.find((candidate) => !candidate.builtin);
  if (!pet) throw new Error("pets.json has no add-on pet to exercise /download");

  const dir = join(distDir, "files");
  mkdirSync(dir, { recursive: true });
  const path = join(dir, `${pet.id}.deskpet`);
  const fixture = Buffer.from(
    `${FIXTURE_MARKER} ${pet.id}.deskpet (version ${pet.version}), written by scripts/lib/server.mjs.\n` +
      "It is not a real pet module and is removed when the test stops.\n",
  );

  let owned = false;
  let created = false;
  let bytes = fixture;
  if (existsSync(path)) {
    const existing = readFileSync(path);
    owned = existing.subarray(0, FIXTURE_MARKER.length).toString("latin1") === FIXTURE_MARKER;
    bytes = owned ? fixture : existing;
    if (owned) writeFileSync(path, fixture);
  } else {
    writeFileSync(path, fixture);
    owned = true;
    created = true;
  }

  return {
    id: pet.id,
    version: pet.version,
    name: pet.name,
    path,
    bytes,
    owned,
    created,
    remove() {
      if (owned) rmSync(path, { force: true });
    },
  };
}

export async function waitFor(url, timeoutMs = 90000) {
  const start = Date.now();
  let lastError = null;
  while (Date.now() - start < timeoutMs) {
    try {
      const response = await fetch(url, { redirect: "manual" });
      if (response.status < 500) return response;
      lastError = new Error(`status ${response.status}`);
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(`Server did not become ready: ${lastError?.message ?? "timeout"}`);
}

export async function startPages({ port = 8788, token = testToken(), quiet = true } = {}) {
  if (!existsSync(DIST)) throw new Error("dist/ is missing; run `npm run build` first");
  const fixture = ensureDownloadFixture();

  const child = spawn(
    process.execPath,
    [
      WRANGLER,
      "pages",
      "dev",
      "dist",
      "--port",
      String(port),
      "--binding",
      `STATS_TOKEN=${token}`,
      "--log-level",
      "warn",
    ],
    { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] },
  );

  let output = "";
  child.stdout.on("data", (chunk) => {
    output += chunk;
    if (!quiet) process.stdout.write(chunk);
  });
  child.stderr.on("data", (chunk) => {
    output += chunk;
    if (!quiet) process.stderr.write(chunk);
  });

  const exited = new Promise((resolve) => {
    child.once("exit", resolve);
    child.once("error", () => resolve("spawn error"));
  });
  const url = `http://127.0.0.1:${port}`;
  let stopped = false;

  function stop() {
    if (!stopped) {
      stopped = true;
      if (child.pid && !child.killed) {
        try {
          if (process.platform === "win32") {
            execFileSync("taskkill", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore" });
          } else {
            child.kill("SIGTERM");
          }
        } catch {
          /* already gone */
        }
      }
      fixture.remove();
    }
  }

  try {
    await Promise.race([
      waitFor(`${url}/`),
      exited.then((code) => {
        throw new Error(`wrangler exited early (${code}):\n${output}`);
      }),
    ]);
  } catch (error) {
    stop();
    throw error;
  }

  return { url, port, token, fixture, stop, output: () => output };
}
