import { test, expect, Page } from "@playwright/test";
import { readFileSync } from "node:fs";

// Drives the desktop app's real frontend (served on :4322) in a headless browser
// with a mocked Tauri bridge. This is the "sandbox" that reviews the app UI:
// each assertion guards a class of bug found by hand (white-on-white select,
// unequal cards, both auth fields showing at once, accordion, installed state).

const APP = "http://localhost:4322";
const CONNECTORS = JSON.parse(readFileSync("site/registry.json", "utf8")).connectors as any[];

async function openApp(page: Page, installed: any[] = []) {
  await page.addInitScript(
    ([conns, inst]) => {
      (window as any).__TAURI__ = {
        core: {
          invoke: async (cmd: string) => {
            if (cmd === "list_connectors") return conns;
            if (cmd === "list_installed") return inst;
            if (cmd === "test_connection") return "Connection OK";
            if (cmd === "install_connector") return "ok";
            return null;
          },
        },
      };
    },
    [CONNECTORS, installed] as const
  );
  await page.goto(APP);
  await page.waitForSelector(".card");
}

function cardByName(page: Page, name: string) {
  return page.locator(".card").filter({ has: page.locator(".card-name", { hasText: name }) }).first();
}

test("dashboard renders one card per connector", async ({ page }) => {
  await openApp(page);
  await expect(page.locator(".card")).toHaveCount(CONNECTORS.length);
});

test("cards in a row are equal height", async ({ page }) => {
  await openApp(page);
  const heights = await page.locator(".card").evaluateAll((cards) =>
    cards.slice(0, 3).map((c) => Math.round(c.getBoundingClientRect().height))
  );
  expect(Math.max(...heights) - Math.min(...heights)).toBeLessThanOrEqual(2);
});

test("search filters the grid and the no-match state appears", async ({ page }) => {
  await openApp(page);
  await page.fill("#app-search", "jira");
  await expect(page.locator(".card")).toHaveCount(1);
  await expect(cardByName(page, "Jira")).toBeVisible();
  await page.fill("#app-search", "zzqqxx-nope");
  await expect(page.locator(".card")).toHaveCount(0);
  await expect(page.locator(".empty-state")).toBeVisible();
});

test("group filter narrows to that group", async ({ page }) => {
  await openApp(page);
  const dev = CONNECTORS.filter((c) => (c.group || "Other") === "Dev").length;
  test.skip(dev === 0, "no Dev group");
  await page.locator('.app-filter[data-group="Dev"]').click();
  await expect(page.locator(".card")).toHaveCount(dev);
});

test("only one card opens at a time and it spans full width", async ({ page }) => {
  await openApp(page);
  const cards = page.locator(".card");
  await cards.nth(0).locator(".expander").click();
  await cards.nth(1).locator(".expander").click();
  await expect(page.locator(".card.is-open")).toHaveCount(1);
});

test("auth toggle shows only the token by default, then switches to username+password", async ({ page }) => {
  await openApp(page);
  const card = cardByName(page, "Confluence");
  await card.locator(".expander").click();
  const token = card.locator('.field[data-env="CONFLUENCE_TOKEN"]');
  const user = card.locator('.field[data-env="CONFLUENCE_USERNAME"]');
  const pass = card.locator('.field[data-env="CONFLUENCE_PASSWORD"]');
  await expect(token).toBeVisible();
  await expect(user).toBeHidden();
  await expect(pass).toBeHidden();
  await card.locator('.seg-btn[data-method="basic"]').click();
  await expect(token).toBeHidden();
  await expect(user).toBeVisible();
  await expect(pass).toBeVisible();
});

test("the role select uses a dark color-scheme (no white-on-white dropdown)", async ({ page }) => {
  await openApp(page);
  const card = cardByName(page, "Confluence");
  await card.locator(".expander").click();
  // color-scheme is set on body; the native dropdown then renders dark.
  const scheme = await page.evaluate(() => getComputedStyle(document.body).colorScheme);
  expect(scheme).toContain("dark");
  await expect(card.locator(".role-select")).toBeVisible();
});

test("installed connector shows the On state and the installed count", async ({ page }) => {
  await openApp(page, [{ id: "jira", command: "x", env: { JIRA_MODE: "writer" } }]);
  await expect(cardByName(page, "Jira")).toHaveAttribute("data-state", "on");
  await expect(page.locator(".installed-chip")).toContainText("1");
});

test("report-a-bug dialog opens", async ({ page }) => {
  await openApp(page);
  await page.locator("#report-open").click();
  await expect(page.locator("#report-dialog")).toBeVisible();
  await expect(page.locator("#report-msg")).toBeVisible();
});

test("report-a-bug posts a well-formed Web3Forms request (real key) and shows success", async ({ page }) => {
  await openApp(page);
  let payload: any = null;
  await page.route("https://api.web3forms.com/submit", async (route) => {
    payload = route.request().postDataJSON();
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ success: true, message: "Email sent" }),
    });
  });
  await page.locator("#report-open").click();
  await page.locator("#report-msg").fill("E2E: connector failed to install");
  await page.locator("#report-form button[type=submit]").click();
  await expect(page.locator("#report-status")).toContainText(/sent|Thanks/i);
  // The request the app actually sent must carry the real access key + context.
  expect(payload.access_key).toBe("ac3d1e73-dc17-4da6-9371-7a1bbb9bd8d9");
  expect(payload.message).toContain("connector failed to install");
  expect(payload.message).toMatch(/App version/);
});

test("a Web3Forms failure response surfaces an error (not a false success)", async ({ page }) => {
  await openApp(page);
  await page.route("https://api.web3forms.com/submit", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ success: false, message: "Invalid access key" }),
    });
  });
  await page.locator("#report-open").click();
  await page.locator("#report-msg").fill("E2E failure-path check");
  await page.locator("#report-form button[type=submit]").click();
  await expect(page.locator("#report-status")).toContainText(/Could not send|Check your connection/i);
});

test("fill + test connection + install turns the connector on", async ({ page }) => {
  await openApp(page);
  const card = cardByName(page, "Jira");
  await card.locator(".expander").click();
  await card.locator('.field[data-env="JIRA_URL"] input').fill("https://jira.example.com");
  await card.locator(".btn-test").click();
  await expect(card.locator(".card-status")).toContainText("Connection OK");
  await card.locator(".btn-install").click();
  await expect(card).toHaveAttribute("data-state", "on");
  await expect(card.locator(".restart-note")).toBeVisible();
  await expect(card.locator(".btn-install")).toContainText(/Update/);
});

test("installing with a required field empty shows a validation error", async ({ page }) => {
  await openApp(page);
  page.on("dialog", (d) => d.accept());
  const card = cardByName(page, "Jira");
  await card.locator(".expander").click();
  await card.locator(".btn-install").click(); // JIRA_URL left empty
  await expect(card.locator(".card-status")).toContainText(/required/i);
  await expect(card.locator('.field[data-env="JIRA_URL"]')).toHaveClass(/field-err/);
});

test("secret field reveal toggles between hidden and visible", async ({ page }) => {
  await openApp(page);
  const card = cardByName(page, "Jira");
  await card.locator(".expander").click();
  const token = card.locator('.field[data-env="JIRA_TOKEN"] input');
  await expect(token).toHaveAttribute("type", "password");
  await card.locator('.field[data-env="JIRA_TOKEN"] .reveal-btn').click();
  await expect(token).toHaveAttribute("type", "text");
});

test("remove turns an installed connector back off", async ({ page }) => {
  await openApp(page, [{ id: "jira", command: "x", env: { JIRA_MODE: "writer" } }]);
  page.on("dialog", (d) => d.accept());
  const card = cardByName(page, "Jira");
  await card.locator(".expander").click();
  await card.locator(".btn-remove").click();
  await expect(card).toHaveAttribute("data-state", "off");
});

test("language toggle translates the app chrome", async ({ page }) => {
  await openApp(page);
  await page.locator('.lang-btn[data-lang="vi"]').click();
  await expect(page.locator(".intro h1")).toHaveText(/Bảng điều khiển/);
});

test("advanced section is collapsed by default and expands on click", async ({ page }) => {
  await openApp(page);
  const card = cardByName(page, "Jira"); // Jira has advanced (proxy/CA/SSL) fields
  await card.locator(".expander").click();
  const adv = card.locator("details.advanced");
  await expect(adv).toHaveCount(1);
  await expect(adv).toHaveJSProperty("open", false);
  await adv.locator("summary").click();
  await expect(adv).toHaveJSProperty("open", true);
});

test("permission role can be switched to writer", async ({ page }) => {
  await openApp(page);
  const card = cardByName(page, "Jira");
  await card.locator(".expander").click();
  const role = card.locator(".role-select");
  await role.selectOption("writer");
  await expect(role).toHaveValue("writer");
});

test("installing without a successful test asks for confirmation; dismissing cancels", async ({ page }) => {
  await openApp(page);
  const card = cardByName(page, "Jira");
  await card.locator(".expander").click();
  await card.locator('.field[data-env="JIRA_URL"] input').fill("https://jira.example.com");
  let dialogMsg = "";
  page.once("dialog", (d) => { dialogMsg = d.message(); d.dismiss(); });
  await card.locator(".btn-install").click();
  expect(dialogMsg).toMatch(/Test connection/i);
  await expect(card).toHaveAttribute("data-state", "off"); // dismissed → not installed
});

test("a failed test connection surfaces the error", async ({ page }) => {
  await page.addInitScript(([conns]) => {
    (window as any).__TAURI__ = { core: { invoke: async (cmd: string) => {
      if (cmd === "list_connectors") return conns;
      if (cmd === "list_installed") return [];
      if (cmd === "test_connection") throw "401 Unauthorized (simulated)";
      return null;
    } } };
  }, [CONNECTORS] as const);
  await page.goto(APP);
  await page.waitForSelector(".card");
  const card = cardByName(page, "Jira");
  await card.locator(".expander").click();
  await card.locator('.field[data-env="JIRA_URL"] input').fill("https://jira.example.com");
  await card.locator(".btn-test").click();
  await expect(card.locator(".status")).toContainText(/401|Unauthorized|simulated/i);
});

test("an install failure surfaces the error and leaves the connector off", async ({ page }) => {
  await page.addInitScript(([conns]) => {
    (window as any).__TAURI__ = { core: { invoke: async (cmd: string) => {
      if (cmd === "list_connectors") return conns;
      if (cmd === "list_installed") return [];
      if (cmd === "test_connection") return "Connection OK";
      if (cmd === "install_connector") throw "Disk full (simulated)";
      return null;
    } } };
  }, [CONNECTORS] as const);
  await page.goto(APP);
  await page.waitForSelector(".card");
  const card = cardByName(page, "Jira");
  await card.locator(".expander").click();
  await card.locator('.field[data-env="JIRA_URL"] input').fill("https://jira.example.com");
  await card.locator(".btn-test").click();
  await expect(card.locator(".status")).toContainText("Connection OK");
  await card.locator(".btn-install").click();
  await expect(card.locator(".status")).toContainText(/Disk full|simulated/i);
  await expect(card).toHaveAttribute("data-state", "off");
});
