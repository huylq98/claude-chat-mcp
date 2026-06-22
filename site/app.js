// Renders the connector catalog from registry.json (the single source of truth
// shared with the future configurator wizard). Bilingual: EN default, VI toggle.
// No framework, no build.

const grid = document.getElementById("grid");
const filtersEl = document.getElementById("filters");
const countEl = document.getElementById("conn-count");

const STRINGS = {
  en: {
    nav_connectors: "Connectors", nav_install: "Install",
    eyebrow: "Open-source · MCP for Claude Desktop",
    hero_title: "Connect Claude to the tools<br />your company actually runs.",
    hero_lede: "Local connectors for the self-hosted and on-prem editions Anthropic's cloud connectors skip. No cloud, no hosting, behind your firewall.",
    cta_browse: "Browse connectors", cta_install: "How it installs",
    h2_connectors: "Connectors", h2_install: "Install locally",
    step1_h: "Build", step1_p: "Compile the connectors once.",
    step2_h: "Register", step2_p: "Point Claude Desktop at the binary with the snippet on each connector card.",
    step3_h: "Restart", step3_p: "Fully quit Claude Desktop (tray, then Exit) and reopen it. Ask Claude to use the new tools.",
    footer_license: "Open-source · MIT", footer_src: "Generated from <code>registry.json</code>",
    loading: "Loading registry…",
    all: "All", connectors_word: "connectors",
    setup: "Setup & install", docs: "Docs", required: "required", optional: "optional",
  },
  vi: {
    nav_connectors: "Trình kết nối", nav_install: "Cài đặt",
    eyebrow: "Mã nguồn mở · MCP cho Claude Desktop",
    hero_title: "Kết nối Claude tới những công cụ<br />nội bộ công ty bạn đang dùng.",
    hero_lede: "Trình kết nối chạy nội bộ cho các phiên bản self-hosted mà trình kết nối đám mây của Anthropic bỏ qua. Không đám mây, chạy sau tường lửa.",
    cta_browse: "Xem trình kết nối", cta_install: "Cách cài đặt",
    h2_connectors: "Trình kết nối", h2_install: "Cài đặt cục bộ",
    step1_h: "Biên dịch", step1_p: "Biên dịch các trình kết nối một lần.",
    step2_h: "Đăng ký", step2_p: "Trỏ Claude Desktop tới tệp chạy bằng đoạn lệnh trên mỗi thẻ trình kết nối.",
    step3_h: "Khởi động lại", step3_p: "Thoát hẳn Claude Desktop (khay hệ thống, rồi Exit) rồi mở lại. Yêu cầu Claude dùng công cụ mới.",
    footer_license: "Mã nguồn mở · MIT", footer_src: "Tạo từ <code>registry.json</code>",
    loading: "Đang tải danh sách…",
    all: "Tất cả", connectors_word: "trình kết nối",
    setup: "Cài đặt & cấu hình", docs: "Tài liệu", required: "bắt buộc", optional: "tùy chọn",
  },
};

// Vietnamese connector descriptions, keyed by connector id (chrome-layer i18n;
// registry.json stays English as the source of truth).
const VI_DESC = {
  confluence: "Tìm kiếm và đọc trang Confluence tự lưu trữ (Server / Data Center).",
  jira: "Tìm kiếm và đọc issue Jira bằng JQL trên bản tự lưu trữ.",
  bitbucket: "Xem repository, pull request và commit trên Bitbucket Server.",
  airtable: "Đọc và ghi base, bảng và bản ghi Airtable.",
  mysql: "Truy vấn chỉ-đọc cơ sở dữ liệu MySQL.",
  mariadb: "Truy vấn chỉ-đọc cơ sở dữ liệu MariaDB.",
  clickhouse: "Truy vấn chỉ-đọc cơ sở dữ liệu ClickHouse.",
  oracle: "Truy vấn chỉ-đọc cơ sở dữ liệu Oracle.",
};
const GROUP_VI = { Atlassian: "Atlassian", Data: "Dữ liệu", Productivity: "Năng suất", Other: "Khác" };

let currentLang = "en";
let currentFilter = "All";
let connectors = [];

const t = (k) => (STRINGS[currentLang] && STRINGS[currentLang][k]) || STRINGS.en[k] || k;
const esc = (s) =>
  String(s).replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c])
  );

function placeholder(f) {
  if (f.default) return f.default;
  if (f.kind === "secret") return "<your token>";
  if (f.kind === "select" && f.options && f.options.length) return f.options[0];
  return "...";
}

function snippetFor(c) {
  const fields = c.auth_fields || [];
  const picked = [];
  const add = (f) => { if (f && !picked.includes(f)) picked.push(f); };
  fields.forEach((f) => { if (f.required) add(f); });
  fields.forEach((f) => { if (f.kind === "secret") add(f); });
  if (!picked.some((f) => f.kind === "text" || f.kind === "select")) {
    add(fields.find((f) => f.kind === "text" || f.kind === "select"));
  }
  if (picked.length === 0) fields.slice(0, 2).forEach(add);
  const pairs = picked.map((f) => `${f.env}="${placeholder(f)}"`).join("; ");
  return `./scripts/install-local.ps1 ${c.id} @{ ${pairs} }`;
}

function fieldRows(c) {
  const all = [...(c.auth_fields || []), ...(c.advanced_fields || [])];
  return all
    .map((f) => {
      const req = f.required ? `<span class="req">${t("required")}</span>` : t("optional");
      const kind = f.kind === "secret" ? "secret" : f.kind || "text";
      return `<div class="field-row"><span class="field-env">${esc(f.env)}</span><span class="field-meta">${esc(kind)} · ${req}</span></div>`;
    })
    .join("");
}

function card(c) {
  const snippet = snippetFor(c);
  const desc = (currentLang === "vi" && VI_DESC[c.id]) ? VI_DESC[c.id] : (c.description || "");
  const group = c.group || "Other";
  const groupLabel = currentLang === "vi" ? (GROUP_VI[group] || group) : group;
  const docs = c.docs_url
    ? `<a class="docs-link" href="${esc(c.docs_url)}" target="_blank" rel="noopener">${t("docs")} ↗</a>`
    : "";
  const note = c.notes ? `<p class="note">${esc(c.notes)}</p>` : "";

  // Card front stays friendly: group, name, description. The technical setup
  // (install command + config fields) is tucked into a collapsed section.
  return `
  <article class="card-shell reveal" data-group="${esc(group)}">
    <div class="card">
      <span class="group-pill">${esc(groupLabel)}</span>
      <h3>${esc(c.name)}</h3>
      <p class="card-desc">${esc(desc)}</p>
      <details class="fields">
        <summary><span class="chev">›</span> ${t("setup")}</summary>
        <div class="snippet-block">
          <code class="snippet">${esc(snippet)}</code>
          <button class="copy-btn" type="button" aria-label="Copy install command" data-copy="${esc(snippet)}">⧉</button>
        </div>
        ${fieldRows(c)}
        ${note}
      </details>
      ${docs ? `<div class="card-foot">${docs}</div>` : ""}
    </div>
  </article>`;
}

function render(list, group) {
  currentFilter = group || "All";
  const shown = currentFilter !== "All" ? list.filter((c) => (c.group || "Other") === currentFilter) : list;
  grid.innerHTML = shown.map(card).join("");
  observeReveals();
  wireCopy();
}

function buildFilters() {
  const groups = ["All", ...new Set(connectors.map((c) => c.group || "Other"))];
  filtersEl.innerHTML = groups
    .map((g) => {
      const label = g === "All" ? t("all") : (currentLang === "vi" ? (GROUP_VI[g] || g) : g);
      return `<button class="filter${g === currentFilter ? " active" : ""}" data-group="${esc(g)}">${esc(label)}</button>`;
    })
    .join("");
  filtersEl.querySelectorAll(".filter").forEach((btn) => {
    btn.addEventListener("click", () => {
      filtersEl.querySelectorAll(".filter").forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      render(connectors, btn.dataset.group);
    });
  });
}

function setCount() {
  countEl.textContent = `${connectors.length} ${t("connectors_word")}`;
}

let io;
function observeReveals() {
  if (!io) {
    io = new IntersectionObserver(
      (entries) => entries.forEach((e) => { if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); } }),
      { threshold: 0.15 }
    );
  }
  document.querySelectorAll(".reveal:not(.in)").forEach((el) => io.observe(el));
}

function wireCopy() {
  document.querySelectorAll(".copy-btn").forEach((btn) => {
    btn.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(btn.dataset.copy);
        btn.textContent = "✓";
        btn.classList.add("copied");
        setTimeout(() => { btn.textContent = "⧉"; btn.classList.remove("copied"); }, 1400);
      } catch { /* clipboard blocked; ignore */ }
    });
  });
}

function applyLang(lang) {
  currentLang = lang === "vi" ? "vi" : "en";
  const dict = STRINGS[currentLang];
  document.documentElement.lang = currentLang;
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const k = el.dataset.i18n;
    if (dict[k] != null) el.textContent = dict[k];
  });
  document.querySelectorAll("[data-i18n-html]").forEach((el) => {
    const k = el.dataset.i18nHtml;
    if (dict[k] != null) el.innerHTML = dict[k];
  });
  document.querySelectorAll(".lang-btn").forEach((b) => b.classList.toggle("active", b.dataset.lang === currentLang));
  try { localStorage.setItem("lang", currentLang); } catch { /* ignore */ }
  if (connectors.length) {
    setCount();
    buildFilters();
    render(connectors, currentFilter);
  }
}

function initLang() {
  let stored = "en";
  try { stored = localStorage.getItem("lang") || "en"; } catch { /* ignore */ }
  applyLang(stored);
  document.querySelectorAll(".lang-btn").forEach((b) =>
    b.addEventListener("click", () => applyLang(b.dataset.lang))
  );
}

async function main() {
  initLang();
  observeReveals();
  try {
    const res = await fetch("./registry.json", { cache: "no-store" });
    if (!res.ok) throw new Error(`registry.json ${res.status}`);
    const data = await res.json();
    connectors = data.connectors || [];
    setCount();
    buildFilters();
    render(connectors, "All");
  } catch (e) {
    grid.innerHTML = `<p class="muted">Could not load registry.json (${esc(e.message)}). Run <code>./scripts/registry.ps1 --release</code> and serve via <code>./scripts/serve-site.ps1</code>.</p>`;
  }
}

main();
