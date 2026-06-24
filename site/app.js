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
    cta_install: "Get the app",
    h2_connectors: "What Claude can connect to", h2_install: "Or install one at a time",
    conn_sub: "Every connector below comes built into the app. Install once, then turn the ones you need on or off.",
    cta_download: "Download the app", cta_browse: "Browse connectors",
    cta_platforms: "Free and open source. Windows, macOS, Linux.",
    video_cap: "Add a connector's address, pick a permission, install. Under a minute.",
    fb_open: "Feedback",
    fb_title: "Send feedback",
    fb_sub: "Found a bug or have an idea? Tell us.",
    fb_ph: "What's working, what isn't, what you'd like.",
    fb_email: "Your email (optional)",
    fb_cancel: "Cancel",
    fb_send: "Send",
    fb_sending: "Sending…",
    fb_ok: "Thanks. Your feedback was sent.",
    fb_err: "Could not send. Please try again later.",
    tut_1: "Browse the connectors you need.",
    tut_2: "Enter your server address, token, and permission.",
    tut_3: "Turn it on, then just ask Claude.",
    card_dl_manual: "Download .mcpb",
    h2_app: "Get the app",
    app_h: "One app to manage every connector",
    app_p: "Install the app, then turn connectors on or off, enter your link and password, pick a permission, and test the connection. No files to drag.",
    app_note: "Free. Works on Windows, macOS, and Linux.",
    app_dl: "Download the app",
    app_dl_for: "Download for",
    os_windows: "Windows", os_macos: "macOS", os_linux: "Linux (.deb)",
    os_linux_appimage: "Linux (AppImage)",
    app_other: "Other formats and older versions",
    step1_h: "Download", step1_p: "Prefer to do it by hand? Use the small Download .mcpb link on any connector to get a single file.",
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
    cta_install: "Tải ứng dụng",
    h2_connectors: "Những gì Claude có thể kết nối", h2_install: "Hoặc cài từng cái một",
    conn_sub: "Mọi trình kết nối bên dưới đều có sẵn trong ứng dụng. Cài một lần, rồi bật tắt cái bạn cần.",
    cta_download: "Tải ứng dụng", cta_browse: "Xem trình kết nối",
    cta_platforms: "Miễn phí và mã nguồn mở. Windows, macOS, Linux.",
    video_cap: "Nhập địa chỉ, chọn quyền, cài đặt. Dưới một phút.",
    fb_open: "Góp ý",
    fb_title: "Gửi góp ý",
    fb_sub: "Gặp lỗi hay có ý tưởng? Hãy cho chúng tôi biết.",
    fb_ph: "Điều gì chạy tốt, điều gì chưa, bạn muốn gì.",
    fb_email: "Email của bạn (không bắt buộc)",
    fb_cancel: "Hủy",
    fb_send: "Gửi",
    fb_sending: "Đang gửi…",
    fb_ok: "Cảm ơn. Góp ý của bạn đã được gửi.",
    fb_err: "Không gửi được. Vui lòng thử lại sau.",
    tut_1: "Chọn trình kết nối bạn cần.",
    tut_2: "Nhập địa chỉ máy chủ, token và quyền.",
    tut_3: "Bật lên, rồi hỏi Claude.",
    card_dl_manual: "Tải .mcpb",
    h2_app: "Tải ứng dụng",
    app_h: "Một ứng dụng quản lý mọi trình kết nối",
    app_p: "Cài ứng dụng, rồi bật tắt trình kết nối, nhập địa chỉ và mật khẩu, chọn quyền, và kiểm tra kết nối. Không cần kéo thả tệp.",
    app_note: "Miễn phí. Chạy trên Windows, macOS và Linux.",
    app_dl: "Tải ứng dụng",
    app_dl_for: "Tải cho",
    os_windows: "Windows", os_macos: "macOS", os_linux: "Linux (.deb)",
    os_linux_appimage: "Linux (AppImage)",
    app_other: "Định dạng khác và phiên bản cũ",
    step1_h: "Tải về", step1_p: "Muốn làm thủ công? Dùng liên kết Tải .mcpb nhỏ trên mỗi trình kết nối để lấy một tệp.",
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
  sentry: "Browse your company's Sentry projects and error reports.",
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
  sentry: "Xem các dự án và báo cáo lỗi Sentry của công ty bạn.",
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

// Feedback form posts to Web3Forms (free, no backend; routes to your email).
// Get a key at https://web3forms.com and paste it here to enable sending.
const FEEDBACK_ENDPOINT = "https://api.web3forms.com/submit";
const FEEDBACK_ACCESS_KEY = "ac3d1e73-dc17-4da6-9371-7a1bbb9bd8d9";

// Control-panel installers. Installer releases use a cp-v* tag (not "latest",
// which belongs to the .mcpb release), and Tauri bakes the version into the
// filename, so bump CP_TAG + CP_VERSION together when cutting a new installer.
// Keep CP_TAG/CP_VERSION in sync with the live installer release.
const CP_TAG = "cp-v0.14.0";
const CP_VERSION = "0.14.0";
const CP_BASE = `https://github.com/huylq98/claude-chat-mcp/releases/download/${CP_TAG}`;
const CP_RELEASES = "https://github.com/huylq98/claude-chat-mcp/releases";
const CP_INSTALLERS = {
  windows: `Claude.Chat.MCP_${CP_VERSION}_x64-setup.exe`,
  macos: `Claude.Chat.MCP_${CP_VERSION}_universal.dmg`,
  // Default Linux download is the small .deb; the AppImage is offered as an
  // extra link below for non-Debian distros.
  linux: `Claude.Chat.MCP_${CP_VERSION}_amd64.deb`,
};
const CP_LINUX_APPIMAGE = `Claude.Chat.MCP_${CP_VERSION}_amd64.AppImage`;
function detectOS() {
  const p = ((navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || navigator.userAgent || "").toLowerCase();
  if (p.includes("win")) return "windows";
  if (p.includes("mac") || p.includes("iphone") || p.includes("ipad")) return "macos";
  if (p.includes("linux") || p.includes("x11") || p.includes("android")) return "linux";
  return null;
}
function renderApp() {
  const primary = document.getElementById("app-dl-primary");
  const all = document.getElementById("app-dl-all");
  if (!primary || !all) return;
  const order = ["windows", "macos", "linux"];
  const primaryOs = detectOS() || "windows";
  primary.href = `${CP_BASE}/${CP_INSTALLERS[primaryOs]}`;
  primary.querySelector("span").textContent = `${t("app_dl_for")} ${t("os_" + primaryOs)}`;
  // The hero download points at the same OS-detected installer (the catchy CTA).
  const heroDl = document.getElementById("hero-dl");
  if (heroDl) heroDl.href = `${CP_BASE}/${CP_INSTALLERS[primaryOs]}`;
  all.innerHTML =
    order.filter((o) => o !== primaryOs)
      .map((o) => `<a class="app-dl-link" href="${CP_BASE}/${CP_INSTALLERS[o]}" aria-label="${esc(t("app_dl_for") + " " + t("os_" + o))}" rel="noopener">${esc(t("os_" + o))}</a>`)
      .join("") +
    `<a class="app-dl-link" href="${CP_BASE}/${CP_LINUX_APPIMAGE}" aria-label="${esc(t("app_dl_for") + " " + t("os_linux_appimage"))}" rel="noopener">${esc(t("os_linux_appimage"))}</a>` +
    `<a class="app-dl-link app-dl-more" href="${CP_RELEASES}" target="_blank" rel="noopener">${esc(t("app_other"))}</a>`;
}

function card(c) {
  const desc = currentLang === "vi"
    ? (VI_DESC[c.id] || c.description || "")
    : (EN_DESC[c.id] || c.description || "");
  const group = c.group || "Other";
  const groupLabel = currentLang === "vi" ? (GROUP_VI[group] || group) : group;
  const dl = `${RELEASE_BASE}/${esc(c.id)}.mcpb`;

  // Cards are a showcase of what Claude can connect to; the app is the install
  // path. The per-connector .mcpb stays as a quiet secondary link for people who
  // prefer to drag a single file in. The action area is pinned to the bottom so
  // every link lines up regardless of description length.
  return `
  <article class="card-shell" data-group="${esc(group)}">
    <div class="card">
      <span class="group-pill">${esc(groupLabel)}</span>
      <h3>${esc(c.name)}</h3>
      <p class="card-desc">${esc(desc)}</p>
      <div class="card-actions">
        <a class="card-dl" href="${dl}" download aria-label="${esc(t("card_dl_manual") + ": " + c.name)}">
          <span class="cta-icon" aria-hidden="true">↓</span>
          <span>${t("card_dl_manual")}</span>
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
  renderApp();
  if (window.__refreshTut) window.__refreshTut();
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

function initFeedback() {
  const open = document.getElementById("fb-open");
  const dlg = document.getElementById("fb-dialog");
  const form = document.getElementById("fb-form");
  const statusEl = document.getElementById("fb-status");
  const cancel = document.getElementById("fb-cancel");
  if (!open || !dlg || !form) return;
  open.addEventListener("click", () => { statusEl.textContent = ""; dlg.showModal(); });
  cancel.addEventListener("click", () => dlg.close());
  dlg.addEventListener("click", (e) => { if (e.target === dlg) dlg.close(); });
  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    const msg = document.getElementById("fb-msg").value.trim();
    if (!msg) return;
    if (FEEDBACK_ACCESS_KEY.startsWith("REPLACE")) { statusEl.textContent = t("fb_err"); return; }
    statusEl.textContent = t("fb_sending");
    try {
      const res = await fetch(FEEDBACK_ENDPOINT, {
        method: "POST",
        headers: { "Content-Type": "application/json", Accept: "application/json" },
        body: JSON.stringify({
          access_key: FEEDBACK_ACCESS_KEY,
          subject: "Claude Chat MCP site feedback",
          from_name: "Claude Chat MCP site",
          message: msg,
          email: document.getElementById("fb-email").value.trim() || "(not provided)",
        }),
      });
      // Web3Forms returns HTTP 200 with {success:false} on failure, so check the body.
      const data = await res.json().catch(() => ({}));
      if (!res.ok || !data.success) throw new Error(data.message || "HTTP " + res.status);
      statusEl.textContent = t("fb_ok");
      form.reset();
      setTimeout(() => dlg.close(), 1300);
    } catch (err) {
      statusEl.textContent = t("fb_err");
    }
  });
}

function initTut() {
  const track = document.getElementById("tut-track");
  const prev = document.getElementById("tut-prev");
  const next = document.getElementById("tut-next");
  const dotsWrap = document.getElementById("tut-dots");
  const cap = document.getElementById("tut-cap");
  if (!track || !track.children.length) return;
  const n = track.children.length;
  let idx = 0;
  let timer = null;

  dotsWrap.innerHTML = Array.from({ length: n }, (_, i) =>
    `<button class="tut-dot" type="button" role="tab" data-i="${i}" aria-label="Step ${i + 1}"></button>`
  ).join("");
  const dots = [...dotsWrap.querySelectorAll(".tut-dot")];

  function show(i) {
    idx = (i + n) % n;
    track.style.transform = `translateX(-${idx * 100}%)`;
    dots.forEach((d, k) => {
      d.classList.toggle("active", k === idx);
      d.setAttribute("aria-selected", String(k === idx));
    });
    if (cap) cap.textContent = t(`tut_${idx + 1}`);
  }
  function stopAuto() { if (timer) { clearInterval(timer); timer = null; } }
  function go(i) { show(i); stopAuto(); }

  prev.addEventListener("click", () => go(idx - 1));
  next.addEventListener("click", () => go(idx + 1));
  dots.forEach((d) => d.addEventListener("click", () => go(+d.dataset.i)));

  timer = setInterval(() => show(idx + 1), 5000);
  window.__refreshTut = () => show(idx); // re-translate caption on language switch
  show(0);
}

async function main() {
  initLang();
  initSearch();
  initFeedback();
  initTut();
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
