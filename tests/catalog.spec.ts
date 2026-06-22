import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import { join } from "node:path";

function registry() {
  return JSON.parse(readFileSync(join(process.cwd(), "registry.json"), "utf8"));
}

test("renders one card per connector in the registry", async ({ page }) => {
  await page.goto("/");
  const { connectors } = registry();
  await expect(page.locator(".card-shell")).toHaveCount(connectors.length);
});

test("cards stay simple: no tool chips or tool-count clutter", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  await expect(page.locator(".tool-chip")).toHaveCount(0);
  await expect(page.locator(".tool-count")).toHaveCount(0);
});

test("each connector card shows a name and description", async ({ page }) => {
  await page.goto("/");
  const first = page.locator(".card-shell").first();
  await expect(first.locator("h3")).toBeVisible();
  await expect(first.locator(".card-desc")).toBeVisible();
});

test("each card offers a one-click .mcpb download", async ({ page }) => {
  await page.goto("/");
  const dl = page.locator(".card-shell").first().locator("a.card-dl");
  await expect(dl).toBeVisible();
  await expect(dl).toHaveAttribute("href", /\.mcpb$/);
});

test("download buttons are bottom-aligned across all cards", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  // Gap from each download button's bottom to its card's bottom should be the
  // same for every card (the action area is pinned to the bottom).
  const gaps = await page.$$eval(".card-shell", (cards) =>
    cards.map((card) => {
      const btn = card.querySelector(".card-dl")!.getBoundingClientRect();
      const c = card.getBoundingClientRect();
      return Math.round(c.bottom - btn.bottom);
    })
  );
  expect(Math.max(...gaps) - Math.min(...gaps)).toBeLessThanOrEqual(3);
});

test("group filter narrows the visible connectors", async ({ page }) => {
  await page.goto("/");
  const { connectors } = registry();
  const groups = [...new Set(connectors.map((c: any) => c.group || "Other"))];
  test.skip(groups.length < 2, "needs at least two groups to filter");
  const group = groups[0] as string;
  const expected = connectors.filter((c: any) => (c.group || "Other") === group).length;
  await page.locator(`.filter[data-group="${group}"]`).click();
  await expect(page.locator(".card-shell")).toHaveCount(expected);
});

test("the Dev group filter shows only the code connectors", async ({ page }) => {
  await page.goto("/");
  const { connectors } = registry();
  const devCount = connectors.filter((c: any) => (c.group || "Other") === "Dev").length;
  test.skip(devCount === 0, "no Dev connectors");
  await page.locator('.filter[data-group="Dev"]').click();
  await expect(page.locator(".card-shell")).toHaveCount(devCount);
});

test("page stays em-dash free in Vietnamese", async ({ page }) => {
  await page.goto("/");
  await page.locator('[data-lang="vi"]').click();
  await page.waitForSelector(".card-shell");
  const text = await page.locator("body").innerText();
  expect(text).not.toMatch(/[–—]/);
});

test("page exposes core SEO metadata", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle(/Claude Chat MCP/);
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute("href", /claudechatmcp\.com/);
  await expect(page.locator('meta[property="og:title"]')).toHaveCount(1);
  const ld = await page.locator('script[type="application/ld+json"]').first().textContent();
  const graph = JSON.parse(ld || "{}")["@graph"];
  expect(Array.isArray(graph) && graph.length).toBeGreaterThan(0);
});

test("no-results state offers a reset that restores the grid", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  await page.fill("#search", "zzqqxx-no-such-tool");
  await expect(page.locator(".no-results")).toBeVisible();
  await page.locator("#empty-reset").click();
  await expect(page.locator(".card-shell").first()).toBeVisible();
});

test("rendered page contains no em-dash or en-dash", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  const text = await page.locator("body").innerText();
  expect(text).not.toMatch(/[–—]/);
});
