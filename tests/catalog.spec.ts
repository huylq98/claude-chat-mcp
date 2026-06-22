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

test("rendered page contains no em-dash or en-dash", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  const text = await page.locator("body").innerText();
  expect(text).not.toMatch(/[–—]/);
});
