import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { ensureDownloadFixture } from "../../scripts/lib/server.mjs";

const temps: string[] = [];

function tempDist() {
  const dir = mkdtempSync(join(tmpdir(), "deskpet-fixture-"));
  temps.push(dir);
  mkdirSync(join(dir, "files"), { recursive: true });
  return dir;
}

afterEach(() => {
  for (const dir of temps.splice(0)) rmSync(dir, { recursive: true, force: true });
});

describe("download fixture", () => {
  it("never deletes or edits a real module", () => {
    const dist = tempDist();
    const real = join(dist, "files", "ember.deskpet");
    writeFileSync(real, "real module bytes");

    const fixture = ensureDownloadFixture({ distDir: dist });
    expect(fixture.owned).toBe(false);
    expect(fixture.created).toBe(false);
    expect(readFileSync(real, "utf8")).toBe("real module bytes");

    fixture.remove();
    expect(readFileSync(real, "utf8")).toBe("real module bytes");
  });

  it("creates a missing fixture and removes it on stop", () => {
    const dist = tempDist();

    const fixture = ensureDownloadFixture({ distDir: dist });
    expect(fixture.owned).toBe(true);
    expect(fixture.created).toBe(true);
    expect(existsSync(fixture.path)).toBe(true);
    expect(readFileSync(fixture.path).equals(fixture.bytes)).toBe(true);

    fixture.remove();
    expect(existsSync(fixture.path)).toBe(false);
  });

  it("cleans up a leftover test fixture without touching real modules", () => {
    const dist = tempDist();
    const leftover = join(dist, "files", "ember.deskpet");
    writeFileSync(leftover, "TEST-ONLY FIXTURE for ember.deskpet, left by an earlier run\n");

    const fixture = ensureDownloadFixture({ distDir: dist });
    expect(fixture.owned).toBe(true);
    fixture.remove();
    expect(existsSync(leftover)).toBe(false);
  });
});
