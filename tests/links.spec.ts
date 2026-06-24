import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";

// Guard the site's installer download links: they must advertise the same
// version the app is actually built at, include the smaller Linux .deb (not only
// the heavy AppImage), and every advertised URL must resolve. This is the test
// that catches a stale CP_TAG or a missing release asset before users hit it.

function readConst(src: string, name: string): string {
  const m = src.match(new RegExp(`${name}\\s*=\\s*"([^"]+)"`));
  if (!m) throw new Error(`${name} not found in site/app.js`);
  return m[1];
}

test("site installer links match the app version, include .deb, and resolve", async ({ request }) => {
  const appJs = readFileSync("site/app.js", "utf8");
  const conf = JSON.parse(readFileSync("crates/control-panel/tauri.conf.json", "utf8"));
  const cpVersion = readConst(appJs, "CP_VERSION");
  const cpTag = readConst(appJs, "CP_TAG");

  // The site must advertise the same version the app is built at.
  expect(cpVersion).toBe(conf.version);
  expect(cpTag).toBe(`cp-v${conf.version}`);

  // A Linux .deb link must be present, not only the heavy AppImage.
  expect(appJs).toMatch(/\.deb/);

  // Every advertised installer URL must resolve (200 after redirects).
  const base = `https://github.com/huylq98/claude-chat-mcp/releases/download/${cpTag}`;
  const files = [
    `Claude.Chat.MCP_${cpVersion}_x64-setup.exe`,
    `Claude.Chat.MCP_${cpVersion}_universal.dmg`,
    `Claude.Chat.MCP_${cpVersion}_amd64.AppImage`,
    `Claude.Chat.MCP_${cpVersion}_amd64.deb`,
  ];
  for (const f of files) {
    const res = await request.get(`${base}/${f}`, { maxRedirects: 5 });
    expect(res.status(), `${f} should resolve`).toBe(200);
  }
});
