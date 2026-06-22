import { test, expect } from "@playwright/test";

// The install section reflects the .mcpb flow, in both languages, and the
// catalog stays usable on mobile.

test("install section shows the three-step Claude Desktop flow", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("#install .step")).toHaveCount(3);
  await expect(page.locator("#install")).toContainText("Claude Desktop");
  // No leftover developer command on the page.
  await expect(page.locator("#install")).not.toContainText("cargo");
});

test("Vietnamese translates the install heading", async ({ page }) => {
  await page.goto("/");
  await page.locator('[data-lang="vi"]').click();
  await expect(page.locator("#install h2")).toContainText("Hoặc cài từng cái một");
});

test("Vietnamese translates the get-the-app section", async ({ page }) => {
  await page.goto("/");
  await page.locator('[data-lang="vi"]').click();
  await expect(page.locator("#app h2")).toContainText("Tải ứng dụng");
});

test("mobile viewport stacks the connector cards in one column", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await page.waitForSelector(".card-shell");
  const xs = await page.$$eval(".card-shell", (els) =>
    els.map((e) => Math.round(e.getBoundingClientRect().x))
  );
  expect(new Set(xs).size).toBe(1);
});
