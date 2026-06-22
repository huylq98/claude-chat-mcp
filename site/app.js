// Renders the connector catalog from registry.json (the single source of truth
// shared with the future configurator wizard). Bilingual: EN default, VI toggle.
// No framework, no build.

const grid = document.getElementById("grid");
const filtersEl = document.getElementById("filters");
const countEl = document.getElementById("conn-count");

const STRINGS = {
  en: {
    nav_connectors: "Connectors", nav_install: "Install",
    eyebrow: "Free · works with Claude Desktop",
    hero_title: "Connect Claude to the tools your company runs.",
    hero_lede: "Let Claude read your company's Jira, Confluence, and databases and do real work. Everything runs on your own computer.",
    cta_install: "How it installs",
    h2_connectors: "Connectors", h2_install: "How to install",
    step1_h: "Download", step1_p: "Find your tool above and click \"Download for Claude Desktop\" to get a small file.",
    step2_h: "Open in Claude Desktop", step2_p: "In Claude Desktop, open Settings and then Extensions, and drag the downloaded file in (or just double-click it).",
    step3_h: "Fill in and install", step3_p: "Enter the web address and the password or token your IT gave you, click Install, then reopen Claude Desktop. Now just ask Claude.",
    footer_license: "Free to use",
    loading: "Loading…", search_ph: "Search for a tool, e.g. Jira or Confluence", no_results: "No connectors match",

    all: "All", connectors_word: "connectors",
    add_to_claude: "Download for Claude Desktop",
    install_hint: "Downloads a file. Open it in Claude Desktop to install.",
  },
  vi: {
    nav_connectors: "Trình kết nối", nav_install: "Cài đặt",
    eyebrow: "Miễn phí · dùng với Claude Desktop",
    hero_title: "Kết nối Claude tới công cụ nội bộ công ty bạn dùng.",
    hero_lede: "Cho phép Claude đọc Jira, Confluence và cơ sở dữ liệu của công ty bạn để làm việc thật. Mọi thứ chạy trên máy của bạn.",
    cta_install: "Cách cài đặt",
    h2_connectors: "Trình kết nối", h2_install: "Cách cài đặt",
    step1_h: "Tải về", step1_p: "Tìm công cụ ở trên và bấm \"Tải cho Claude Desktop\" để lấy một tệp nhỏ.",
    step2_h: "Mở trong Claude Desktop", step2_p: "Trong Claude Desktop, mở Settings rồi Extensions, và kéo tệp vừa tải vào (hoặc bấm đúp).",
    step3_h: "Điền thông tin và cài", step3_p: "Nhập địa chỉ web và mật khẩu hoặc token mà bộ phận IT cấp cho bạn, bấm Install, rồi mở lại Claude Desktop. Giờ chỉ cần hỏi Claude.",
    footer_license: "Miễn phí sử dụng",
    loading: "Đang tải…", search_ph: "Tìm công cụ, vd: Jira hoặc Confluence", no_results: "Không có trình kết nối phù hợp",

    all: "Tất cả", connectors_word: "trình kết nối",
    add_to_claude: "Tải cho Claude Desktop",
    install_hint: "Tải về một tệp. Mở trong Claude Desktop để cài đặt.",
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
  gitlab: "Tìm và đọc dự án, issue và merge request trên GitLab tự lưu trữ (và tạo issue/bình luận ở chế độ Writer).",
};
const GROUP_VI = { Atlassian: "Atlassian", Data: "Dữ liệu", Productivity: "Năng suất", Dev: "Lập trình", Other: "Khác" };

let currentLang = "en";
let currentFilter = "All";
let query = "";
let connectors = [];

const t = (k) => (STRINGS[currentLang] && STRINGS[currentLang][k]) || STRINGS.en[k] || k;
const esc = (s) =>
  String(s).replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c])
  );

const RELEASE_BASE = "https://github.com/huylq98/claude-chat-mcp/releases/latest/download";

function card(c) {
  const desc = (currentLang === "vi" && VI_DESC[c.id]) ? VI_DESC[c.id] : (c.description || "");
  const group = c.group || "Other";
  const groupLabel = currentLang === "vi" ? (GROUP_VI[group] || group) : group;
  const dl = `${RELEASE_BASE}/${esc(c.id)}.mcpb`;

  // One-click .mcpb: Claude Desktop collects the URL/token at install time, so the
  // card stays simple. The action area is pinned to the bottom so every card's
  // download button lines up regardless of description length.
  return `
  <article class="card-shell" data-group="${esc(group)}">
    <div class="card">
      <span class="group-pill">${esc(groupLabel)}</span>
      <h3>${esc(c.name)}</h3>
      <p class="card-desc">${esc(desc)}</p>
      <div class="card-actions">
        <a class="btn btn-primary card-dl" href="${dl}" download>
          <span>${t("add_to_claude")}</span>
          <span class="cta-icon" aria-hidden="true">↓</span>
        </a>
      </div>
    </div>
  </article>`;
}

function matchesQuery(c) {
  if (!query) return true;
  const hay = [c.name, c.id, c.description, c.group, VI_DESC[c.id] || ""].join(" ").toLowerCase();
  return hay.includes(query);
}

function visibleList() {
  return connectors.filter(
    (c) => (currentFilter === "All" || (c.group || "Other") === currentFilter) && matchesQuery(c)
  );
}

function renderGrid() {
  const shown = visibleList();
  if (shown.length === 0) {
    grid.innerHTML = `<p class="muted no-results">${t("no_results")}</p>`;
    return;
  }
  grid.innerHTML = shown.map(card).join("");
}

function buildFilters() {
  const groups = ["All", ...new Set(connectors.map((c) => c.group || "Other"))];
  filtersEl.innerHTML = groups
    .map((g) => {
      const label = g === "All" ? t("all") : (currentLang === "vi" ? (GROUP_VI[g] || g) : g);
      const on = g === currentFilter;
      return `<button class="filter${on ? " active" : ""}" aria-pressed="${on}" data-group="${esc(g)}">${esc(label)}</button>`;
    })
    .join("");
  filtersEl.querySelectorAll(".filter").forEach((btn) => {
    btn.addEventListener("click", () => {
      filtersEl.querySelectorAll(".filter").forEach((b) => { b.classList.remove("active"); b.setAttribute("aria-pressed", "false"); });
      btn.classList.add("active");
      btn.setAttribute("aria-pressed", "true");
      currentFilter = btn.dataset.group;
      renderGrid();
    });
  });
}

function setCount() {
  countEl.textContent = `${connectors.length} ${t("connectors_word")}`;
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
  document.querySelectorAll("[data-i18n-ph]").forEach((el) => {
    const k = el.dataset.i18nPh;
    if (dict[k] != null) el.setAttribute("placeholder", dict[k]);
  });
  document.querySelectorAll(".lang-btn").forEach((b) => {
    const on = b.dataset.lang === currentLang;
    b.classList.toggle("active", on);
    b.setAttribute("aria-pressed", String(on));
  });
  try { localStorage.setItem("lang", currentLang); } catch { /* ignore */ }
  if (connectors.length) {
    setCount();
    buildFilters();
    renderGrid();
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

function initSearch() {
  const input = document.getElementById("search");
  const box = document.getElementById("suggest");
  if (!input) return;
  let active = -1;

  const suggestionsFor = (q) =>
    !q ? [] : connectors.filter((c) => [c.name, c.id, c.group].join(" ").toLowerCase().includes(q)).slice(0, 6);
  const itemEls = () => (box ? [...box.querySelectorAll(".suggest-item")] : []);

  function showSuggest(q) {
    if (!box) return;
    const items = suggestionsFor(q);
    if (!items.length) { hideSuggest(); return; }
    active = -1;
    box.innerHTML = items
      .map((c, i) => {
        const group = currentLang === "vi" ? (GROUP_VI[c.group] || c.group) : (c.group || "");
        return `<li class="suggest-item" id="sg-${i}" role="option" data-name="${esc(c.name)}"><span>${esc(c.name)}</span><span class="suggest-group">${esc(group)}</span></li>`;
      })
      .join("");
    box.hidden = false;
    input.setAttribute("aria-expanded", "true");
    input.removeAttribute("aria-activedescendant");
  }
  function hideSuggest() {
    if (!box) return;
    box.hidden = true;
    active = -1;
    input.setAttribute("aria-expanded", "false");
    input.removeAttribute("aria-activedescendant");
  }
  function setActive(i) {
    const els = itemEls();
    if (!els.length) return;
    active = (i + els.length) % els.length;
    els.forEach((el, n) => el.classList.toggle("is-active", n === active));
    input.setAttribute("aria-activedescendant", els[active].id);
  }
  function choose(li) {
    if (!li) return;
    input.value = li.dataset.name;
    query = li.dataset.name.trim().toLowerCase();
    hideSuggest();
    renderGrid();
    input.focus();
  }

  input.addEventListener("input", () => {
    query = input.value.trim().toLowerCase();
    renderGrid();
    showSuggest(query);
  });
  input.addEventListener("focus", () => { if (query) showSuggest(query); });
  input.addEventListener("keydown", (e) => {
    if (box.hidden) { if (e.key === "Escape") hideSuggest(); return; }
    if (e.key === "ArrowDown") { e.preventDefault(); setActive(active + 1); }
    else if (e.key === "ArrowUp") { e.preventDefault(); setActive(active - 1); }
    else if (e.key === "Enter" && active > -1) { e.preventDefault(); choose(itemEls()[active]); }
    else if (e.key === "Escape") { hideSuggest(); }
  });
  input.addEventListener("blur", () => setTimeout(hideSuggest, 120));

  if (box) {
    // mousedown (not click) so it fires before the input's blur hides the list
    box.addEventListener("mousedown", (e) => {
      const li = e.target.closest(".suggest-item");
      if (!li) return;
      e.preventDefault();
      choose(li);
    });
  }
}

async function main() {
  initLang();
  initSearch();
  try {
    const res = await fetch("./registry.json", { cache: "no-store" });
    if (!res.ok) throw new Error(`registry.json ${res.status}`);
    const data = await res.json();
    connectors = data.connectors || [];
    setCount();
    buildFilters();
    renderGrid();
  } catch (e) {
    grid.innerHTML = `<p class="muted">Could not load registry.json (${esc(e.message)}). Run <code>./scripts/registry.ps1 --release</code> and serve via <code>./scripts/serve-site.ps1</code>.</p>`;
  }
}

main();
