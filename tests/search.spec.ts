import { test, expect } from "@playwright/test";

// The hero search filters the connector grid live. Contract for the search box.

test("typing a tool name filters the grid", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  await page.locator("#search").fill("mysql");
  await expect(page.locator(".card-shell")).toHaveCount(1);
  await expect(page.locator(".card-shell h3")).toHaveText("MySQL");
});

test("searching a group name shows that group's connectors", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  await page.locator("#search").fill("atlassian");
  await expect(page.locator(".card-shell")).toHaveCount(3);
});

test("clearing the search restores all connectors", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  const total = await page.locator(".card-shell").count();
  await page.locator("#search").fill("jira");
  await expect(page.locator(".card-shell")).toHaveCount(1);
  await page.locator("#search").fill("");
  await expect(page.locator(".card-shell")).toHaveCount(total);
});

test("typing shows suggestions; clicking one filters to it", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  await page.locator("#search").fill("ji");
  const suggestion = page.locator("#suggest .suggest-item", { hasText: "Jira" });
  await expect(suggestion).toBeVisible();
  await suggestion.click();
  await expect(page.locator(".card-shell")).toHaveCount(1);
  await expect(page.locator(".card-shell h3")).toHaveText("Jira");
  await expect(page.locator("#search")).toHaveValue("Jira");
});

test("cards are visible on load without scrolling", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  // The first card must be visible immediately (not gated behind a scroll reveal).
  await expect(page.locator(".card-shell").first()).toBeVisible();
});

test("a query with no matches shows a no-results message", async ({ page }) => {
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  await page.locator("#search").fill("zzzznotathing");
  await expect(page.locator(".card-shell")).toHaveCount(0);
  await expect(page.locator(".no-results")).toBeVisible();
});
