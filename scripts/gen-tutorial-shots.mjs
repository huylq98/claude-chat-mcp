// Regenerate the hero tutorial carousel screenshots from the CURRENT app UI.
// Run this whenever the control-panel UI changes, so the website tutorial never
// shows a stale version of the app:
//
//   node scripts/gen-tutorial-shots.mjs
//
// It serves crates/control-panel/ui, injects a mocked Tauri bridge, drives the
// app into each step, and writes site/tutorial/step{1,2,3}.png.

import { chromium } from "playwright";
import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";

const PORT = 4399;
const URL = `http://localhost:${PORT}`;
const conns = JSON.parse(readFileSync("site/registry.json", "utf8")).connectors;

const server = spawn("node", ["scripts/static-server.mjs", "crates/control-panel/ui", String(PORT)], {
  stdio: "ignore",
});
await new Promise((r) => setTimeout(r, 1600));

const browser = await chromium.launch();

// Height chosen to land in a grid row-gap (no half-cards) AND clear the tallest
// expanded "configure" card, so no slide is cut at the bottom.
const CLIP_H = 929;

async function shot(installed, action, path) {
  const page = await browser.newPage({ viewport: { width: 1080, height: 1000 } });
  await page.addInitScript(
    (args) => {
      const [c, inst] = args;
      window.__TAURI__ = {
        core: {
          invoke: async (cmd) => {
            if (cmd === "list_connectors") return c;
            if (cmd === "list_installed") return inst;
            if (cmd === "test_connection") return "Connection OK";
            return null;
          },
        },
      };
    },
    [conns, installed]
  );
  await page.goto(URL, { waitUntil: "networkidle" });
  await page.waitForSelector(".card");
  await page.waitForTimeout(800); // let webfonts settle
  if (action) await action(page);
  await page.screenshot({ path, clip: { x: 0, y: 0, width: 1080, height: CLIP_H } });
  await page.close();
}

// 1: browse the dashboard
await shot([], null, "site/tutorial/step1.png");
// 2: configure a connector (expand the first card)
await shot([], async (p) => {
  await p.locator(".card .expander").first().click();
  await p.waitForTimeout(300);
}, "site/tutorial/step2.png");
// 3: connectors turned on
await shot(
  [
    { id: "confluence", command: "x", env: { CONFLUENCE_URL: "https://wiki.co", CONFLUENCE_MODE: "viewer" } },
    { id: "github", command: "x", env: { GITHUB_MODE: "writer" } },
    { id: "postgres", command: "x", env: { POSTGRES_MODE: "viewer" } },
  ],
  null,
  "site/tutorial/step3.png"
);

await browser.close();
server.kill();
console.log("Wrote site/tutorial/step{1,2,3}.png from the current app UI.");
