import { test, expect } from "@playwright/test";

// Bilingual contract (EN default, VI on toggle). These define the i18n behavior
// to implement; they are expected to FAIL until the language toggle exists.

test("defaults to English", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  await expect(page.locator(".hero h1")).toContainText("Connect Claude");
});

test("switching to Vietnamese translates the hero", async ({ page }) => {
  await page.goto("/");
  await page.locator('[data-lang="vi"]').click();
  await expect(page.locator("html")).toHaveAttribute("lang", "vi");
  await expect(page.locator(".hero h1")).toContainText("Kết nối Claude");
  await expect(page.locator(".btn-primary")).toContainText("Xem trình kết nối");
});

test("language choice persists across reload", async ({ page }) => {
  await page.goto("/");
  await page.locator('[data-lang="vi"]').click();
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("lang", "vi");
  await expect(page.locator(".hero h1")).toContainText("Kết nối Claude");
});
