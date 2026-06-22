// Renders the connector catalog from registry.json (the single source of truth
// shared with the future configurator wizard). Bilingual: EN default, VI toggle.
// No framework, no build.

const grid = document.getElementById("grid");
const filtersEl = document.getElementById("filters");
const countEl = document.getElementById("conn-count");
const srStatus = document.getElementById("sr-status");
function srAnnounce(msg) { if (srStatus) srStatus.textContent = msg; }

const STRINGS = {
  en: {
    nav_connectors: "Connectors", nav_install: "Install",
    eyebrow: "Free · works with Claude Desktop",
    hero_title: "Connect Claude to the tools your company runs.",
    hero_lede: "Let Claude read your company's Jira, Confluence, and databases and do real work. Everything runs on your own computer.",
    cta_install: "How it installs",
    h2_connectors: "Connectors", h2_install: "How to install",
    step1_h: "Download", step1_p: "Find your tool above and click \"Download for Claude Desktop\" to get a small file.",
    step2_h: "Open in Claude Desktop", step2_p: "Open Claude Desktop, go to Settings then Extensions, and drag the file in. (Double-clicking the file also works.)",
    step3_h: "Fill in and install", step3_p: "Enter the web address and password your IT gave you, click Install, then reopen Claude Desktop. Now just ask Claude.",
    footer_license: "Free to use",
    loading: "Loading…", search_ph: "Search for a tool, e.g. Jira or Confluence", no_results: "No connectors match", clear_search: "Show all", suggestions: "suggestions",

    all: "All", connectors_word: "connectors",
    add_to_claude: "Download for Claude Desktop",
  },
  vi: {
    nav_connectors: "Trình kết nối", nav_install: "Cài đặt",
    eyebrow: "Miễn phí · dùng với Claude Desktop",
    hero_title: "Kết nối Claude tới công cụ nội bộ công ty bạn dùng.",
    hero_lede: "Cho phép Claude đọc Jira, Confluence và cơ sở dữ liệu của công ty bạn để làm việc thật. Mọi thứ chạy trên máy của bạn.",
    cta_install: "Cách cài đặt",
    h2_connectors: "Trình kết nối", h2_install: "Cách cài đặt",
    step1_h: "Tải về", step1_p: "Tìm công cụ ở trên và bấm \"Tải cho Claude Desktop\" để lấy một tệp nhỏ.",
    step2_h: "Mở trong Claude Desktop", step2_p: "Mở Claude Desktop, vào Settings rồi Extensions, và kéo tệp vào. (Bấm đúp vào tệp cũng được.)",
    step3_h: "Điền thông tin và cài", step3_p: "Nhập địa chỉ web và mật khẩu mà bộ phận IT cấp cho bạn, bấm Install, rồi mở lại Claude Desktop. Giờ chỉ cần hỏi Claude.",
    footer_license: "Miễn phí sử dụng",
    loading: "Đang tải…", search_ph: "Tìm công cụ, vd: Jira hoặc Confluence", no_results: "Không có trình kết nối phù hợp", clear_search: "Hiện tất cả", suggestions: "gợi ý",

    all: "Tất cả", connectors_word: "trình kết nối",
    add_to_claude: "Tải cho Claude Desktop",
  },
};

// Plain-language connector descriptions for the cards (the decision surface).
// registry.json stays the dev source of truth; these are the friendly overrides.
const EN_DESC = {
  confluence: "Search and read your company's Confluence pages, and add pages or comments.",
  jira: "Search and read your team's Jira tickets, and create tickets or comments.",
  bitbucket: "Browse your company's code projects and reviews, and comment on pull requests.",
  airtable: "Read and update your Airtable bases and records.",
  mysql: "Ask questions of your company's MySQL database. Read-only.",
  mariadb: "Ask questions of your company's MariaDB database. Read-only.",
  clickhouse: "Ask questions of your company's ClickHouse database. Read-only.",
  oracle: "Ask questions of your company's Oracle database. Read-only.",
  postgres: "Ask questions of your company's PostgreSQL database. Read-only.",
  gitlab: "Search your company's GitLab projects and tickets, and add comments.",
  github: "Search your company's GitHub repositories and issues, and add comments.",
  jenkins: "See your company's build and deploy jobs, and start them.",
  redmine: "Search and read your team's Redmine projects and issues, and add notes.",
  grafana: "Browse your company's Grafana dashboards, data sources, and alerts.",
  elasticsearch: "Search your company's Elasticsearch or OpenSearch indices and logs.",
  mattermost: "Read your company's Mattermost channels and messages, and post updates.",
  mongodb: "Query your company's MongoDB databases and collections.",
};
const VI_DESC = {
  confluence: "Tìm và đọc các trang Confluence của công ty bạn, và thêm trang hoặc bình luận.",
  jira: "Tìm và đọc các ticket Jira của nhóm bạn, và tạo ticket hoặc bình luận.",
  bitbucket: "Xem các dự án mã nguồn và lượt review của công ty bạn, và bình luận trên pull request.",
  airtable: "Đọc và cập nhật base và bản ghi Airtable của bạn.",
  mysql: "Hỏi đáp dữ liệu MySQL của công ty bạn. Chỉ đọc.",
  mariadb: "Hỏi đáp dữ liệu MariaDB của công ty bạn. Chỉ đọc.",
  clickhouse: "Hỏi đáp dữ liệu ClickHouse của công ty bạn. Chỉ đọc.",
  oracle: "Hỏi đáp dữ liệu Oracle của công ty bạn. Chỉ đọc.",
  postgres: "Hỏi đáp dữ liệu PostgreSQL của công ty bạn. Chỉ đọc.",
  gitlab: "Tìm dự án và ticket GitLab của công ty bạn, và thêm bình luận.",
  github: "Tìm repository và issue GitHub của công ty bạn, và thêm bình luận.",
  jenkins: "Xem các tác vụ build và triển khai của công ty bạn, và chạy chúng.",
  redmine: "Tìm và đọc dự án và issue Redmine của nhóm bạn, và thêm ghi chú.",
  grafana: "Xem bảng điều khiển, nguồn dữ liệu và cảnh báo Grafana của công ty bạn.",
  elasticsearch: "Tìm kiếm trong các chỉ mục và log Elasticsearch hoặc OpenSearch của công ty bạn.",
  mattermost: "Đọc các kênh và tin nhắn Mattermost của công ty bạn, và đăng cập nhật.",
  mongodb: "Truy vấn cơ sở dữ liệu và collection MongoDB của công ty bạn.",
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
  const desc = currentLang === "vi"
    ? (VI_DESC[c.id] || c.description || "")
    : (EN_DESC[c.id] || c.description || "");
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
    grid.innerHTML = `
      <div class="no-results">
        <div class="no-results-mark" aria-hidden="true">⌕</div>
        <p class="no-results-title">${t("no_results")}</p>
        <button type="button" class="btn btn-ghost" id="empty-reset">${t("clear_search")}</button>
      </div>`;
    srAnnounce(t("no_results"));
    const reset = document.getElementById("empty-reset");
    if (reset) reset.addEventListener("click", () => {
      query = "";
      currentFilter = "All";
      const input = document.getElementById("search");
      if (input) input.value = "";
      buildFilters();
      renderGrid();
    });
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
    srAnnounce(`${items.length} ${t("suggestions")}`);
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
