import { test, expect } from "@playwright/test";

// Guards the hero tutorial carousel: it must render at the screenshot aspect
// (not stretched tall and cropped), every slide image must actually load, and
// the controls must advance the slide. This catches the flexbox-stretch bug
// where the image's height attribute blew the slide box up to ~675px tall.

test("carousel renders at the right aspect and loads every slide", async ({ page }) => {
  await page.goto("/");
  const tut = page.locator(".tut");
  await expect(tut).toBeVisible();

  // Aspect ~1080/929 = 1.16 (the stretch bug rendered it ~0.9 — much taller).
  const box = await tut.boundingBox();
  expect(box).not.toBeNull();
  const ratio = box!.width / box!.height;
  expect(ratio).toBeGreaterThan(1.05);
  expect(ratio).toBeLessThan(1.3);

  // Three slides, all images decoded (not broken 404s).
  const slides = page.locator(".tut-slide");
  await expect(slides).toHaveCount(3);
  const loaded = await slides.evaluateAll((imgs) =>
    imgs.map((i) => (i as HTMLImageElement).complete && (i as HTMLImageElement).naturalWidth > 0)
  );
  expect(loaded.every(Boolean), "all tutorial slide images load").toBe(true);

  await expect(page.locator(".tut-dot")).toHaveCount(3);
});

test("next advances the slide and updates the caption", async ({ page }) => {
  await page.goto("/");
  const cap = page.locator("#tut-cap");
  const before = (await cap.textContent())!.trim();
  // Clicking a control stops autoplay, so the state is stable after the click.
  await page.locator("#tut-next").click();
  await expect(cap).not.toHaveText(before);
  await expect(page.locator("#tut-track")).toHaveAttribute("style", /translateX/);
});
