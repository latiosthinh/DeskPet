#!/usr/bin/env node
// Playwright webServer: real Pages runtime + local D1, test-only STATS_TOKEN.
import { initLocalD1, startPages, testToken } from "./lib/server.mjs";

initLocalD1();
const server = await startPages({
  port: Number(process.env.PORT || 8788),
  token: testToken(),
  quiet: false,
});
console.log(`[serve-for-tests] ready at ${server.url}`);

function shutdown() {
  server.stop();
  process.exit(0);
}

process.on("SIGTERM", shutdown);
process.on("SIGINT", shutdown);
process.on("SIGHUP", shutdown);
process.on("exit", () => server.stop());

await new Promise(() => {});
