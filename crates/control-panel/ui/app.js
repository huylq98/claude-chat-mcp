const invoke = window.__TAURI__.core.invoke;

const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => root.querySelectorAll(sel);

/* ── Bilingual strings (chrome only) ────────────────────────────────── */
const STRINGS = {
  en: {
    appName: "Claude Chat MCP",
    title: "Connector control panel",
    lede: "Turn connectors on or off, set credentials, and choose a permission role. Changes are written to Claude Desktop directly.",
    configure: "Configure",
    role: "Permission role",
    viewer: "Viewer (read only)",
    writer: "Writer (read and write)",
    install: "Install",
    update: "Update",
    remove: "Remove",
    on: "On",
    off: "Off",
    restartNote:
      "Now fully quit and reopen Claude Desktop for this change to take effect.",
    installing: "Installing...",
    removing: "Removing...",
    installed: "Saved. Restart Claude Desktop to finish.",
    removed: "Connector removed.",
    advanced: "Advanced (only if your IT team told you to)",
    advancedHint: "Leave these blank unless instructed.",
    confirmUntested: "You have not run a successful Test connection. Install anyway?",
    missingRequired: "Please fill in the required fields.",
    docs: "Documentation",
    testConn: "Test connection",
    testing: "Testing connection...",
    testOk: "Connection OK",
    confirmRemove: "Remove the {name} connector? Claude Desktop will lose access, and the credentials you saved for it will be cleared.",
    roleHint: "Viewer lets Claude read your data only. Writer also lets Claude create, change, and delete it. Choose Viewer unless you need changes made.",
    loadingConnectors: "Loading connectors...",
    emptyState: "No connectors are bundled with this build.",
    loadError: "Could not load connectors:",
    searchPh: "Search connectors",
    groupAll: "All",
    installedNote: "{n} on",
    noMatch: "No connectors match.",
    authMethod: "Sign in with",
    authToken: "Access token",
    authBasic: "Username and password",
    show: "Show", hide: "Hide",
  },
  vi: {
    appName: "Claude Chat MCP",
    title: "Bảng điều khiển trình kết nối",
    lede: "Bật hoặc tắt trình kết nối, nhập thông tin đăng nhập, và chọn vai trò quyền. Thay đổi được ghi trực tiếp vào Claude Desktop.",
    configure: "Cấu hình",
    role: "Vai trò quyền",
    viewer: "Người xem (chỉ đọc)",
    writer: "Người ghi (đọc và ghi)",
    install: "Cài đặt",
    update: "Cập nhật",
    remove: "Gỡ bỏ",
    on: "Bật",
    off: "Tắt",
    restartNote:
      "Bây giờ hãy thoát hẳn và mở lại Claude Desktop để thay đổi có hiệu lực.",
    installing: "Đang cài đặt...",
    removing: "Đang gỡ bỏ...",
    installed: "Đã lưu. Mở lại Claude Desktop để hoàn tất.",
    removed: "Đã gỡ bỏ trình kết nối.",
    advanced: "Nâng cao (chỉ khi bộ phận IT yêu cầu)",
    advancedHint: "Để trống trừ khi được hướng dẫn.",
    confirmUntested: "Bạn chưa chạy Kiểm tra kết nối thành công. Vẫn cài đặt?",
    missingRequired: "Vui lòng điền các trường bắt buộc.",
    docs: "Tài liệu",
    testConn: "Kiểm tra kết nối",
    testing: "Đang kiểm tra kết nối...",
    testOk: "Kết nối OK",
    confirmRemove: "Gỡ trình kết nối {name}? Claude Desktop sẽ mất quyền truy cập, và thông tin đăng nhập đã lưu sẽ bị xóa.",
    roleHint: "Người xem chỉ cho Claude đọc dữ liệu. Người ghi còn cho Claude tạo, sửa và xóa dữ liệu. Hãy chọn Người xem trừ khi bạn cần thay đổi.",
    loadingConnectors: "Đang tải trình kết nối...",
    emptyState: "Bản dựng này không có trình kết nối nào.",
    loadError: "Không tải được trình kết nối:",
    searchPh: "Tìm trình kết nối",
    groupAll: "Tất cả",
    installedNote: "{n} đang bật",
    noMatch: "Không có trình kết nối phù hợp.",
    authMethod: "Đăng nhập bằng",
    authToken: "Token truy cập",
    authBasic: "Tên đăng nhập và mật khẩu",
    show: "Hiện", hide: "Ẩn",
  },
};

let lang = "en";
const t = (key) => (STRINGS[lang] && STRINGS[lang][key]) || STRINGS.en[key] || key;

function applyLang() {
  document.documentElement.lang = lang;
  $$("[data-i18n]").forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });
  if (searchEl) searchEl.placeholder = t("searchPh");
  // Re-render the filter row and connector cards so dynamic labels follow the
  // language too.
  buildFilters();
  render();
}

$$(".lang-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    lang = btn.dataset.lang;
    $$(".lang-btn").forEach((b) => {
      const on = b === btn;
      b.classList.toggle("active", on);
      b.setAttribute("aria-pressed", String(on));
    });
    applyLang();
  });
});

/* ── Generic status helper ──────────────────────────────────────────── */
function setStatus(el, kind, msg) {
  if (!msg) {
    el.className = "status";
    el.textContent = "";
    return;
  }
  el.className = `status visible ${kind}`;
  el.textContent = msg;
}

const globalStatus = $("#global-status");

/* ── Data ───────────────────────────────────────────────────────────── */
let connectors = [];
let installedById = {}; // id -> { command, env }

async function loadData() {
  connectors = await invoke("list_connectors");
  const installed = await invoke("list_installed");
  installedById = {};
  for (const e of installed) installedById[e.id] = { command: e.command, env: e.env };
}

/* ── Render ─────────────────────────────────────────────────────────── */
const grid = $("#connectors");
const tpl = $("#card-tpl");
const searchEl = $("#app-search");
const filtersEl = $("#app-filters");
let currentGroup = "All";
let query = "";

searchEl.addEventListener("input", () => { query = searchEl.value; render(); });

function buildFilters() {
  if (!connectors.length) { filtersEl.innerHTML = ""; return; }
  const groups = ["All", ...new Set(connectors.map((c) => c.group || "Other"))];
  filtersEl.innerHTML = groups
    .map((g) => {
      const label = g === "All" ? t("groupAll") : g;
      const on = g === currentGroup;
      return `<button type="button" class="app-filter${on ? " active" : ""}" aria-pressed="${on}" data-group="${g}">${label}</button>`;
    })
    .join("");
  const onCount = Object.keys(installedById).length;
  if (onCount > 0) {
    const chip = document.createElement("span");
    chip.className = "installed-chip";
    chip.textContent = t("installedNote").replace("{n}", String(onCount));
    filtersEl.appendChild(chip);
  }
  filtersEl.querySelectorAll(".app-filter").forEach((btn) => {
    btn.addEventListener("click", () => {
      currentGroup = btn.dataset.group;
      buildFilters();
      render();
    });
  });
}

function visibleConnectors() {
  const q = query.trim().toLowerCase();
  return connectors.filter((c) => {
    const inGroup = currentGroup === "All" || (c.group || "Other") === currentGroup;
    const hay = [c.name, c.id, c.description, c.group].join(" ").toLowerCase();
    return inGroup && (!q || hay.includes(q));
  });
}

function render() {
  grid.innerHTML = "";
  if (!connectors.length) {
    const p = document.createElement("p");
    p.className = "empty-state";
    p.textContent = t("emptyState");
    grid.appendChild(p);
    return;
  }
  const shown = visibleConnectors();
  if (!shown.length) {
    const p = document.createElement("p");
    p.className = "empty-state";
    p.textContent = t("noMatch");
    grid.appendChild(p);
    return;
  }
  for (const c of shown) grid.appendChild(buildCard(c));
}

function buildCard(c) {
  const node = tpl.content.firstElementChild.cloneNode(true);
  const installed = installedById[c.id];
  const isOn = !!installed;

  // Translate the card chrome (Configure, role label/hint, restart note, Remove)
  // for the current language; specific dynamic labels are overridden below.
  $$("[data-i18n]", node).forEach((el) => { el.textContent = t(el.dataset.i18n); });

  node.dataset.state = isOn ? "on" : "off";
  $(".card-group", node).textContent = c.group;
  $(".card-name", node).textContent = c.name;
  $(".card-desc", node).textContent = c.description;

  const pill = $(".state-pill", node);
  pill.textContent = isOn ? t("on") : t("off");
  pill.classList.toggle("on", isOn);

  // Required credentials go in the main form. Advanced/network fields (proxy,
  // CA bundle, SSL) are tucked into a collapsed section so non-technical users
  // are not confused by IT plumbing next to their token.
  const form = $(".fields", node);
  for (const f of (c.auth_fields || [])) form.appendChild(buildField(f, installed));
  setupAuthToggle(form, installed, node);
  const adv = c.advanced_fields || [];
  if (adv.length) {
    const details = document.createElement("details");
    details.className = "advanced";
    const summary = document.createElement("summary");
    summary.textContent = t("advanced");
    details.appendChild(summary);
    const hint = document.createElement("p");
    hint.className = "advanced-hint";
    hint.textContent = t("advancedHint");
    details.appendChild(hint);
    const advGrid = document.createElement("div");
    advGrid.className = "fields";
    for (const f of adv) advGrid.appendChild(buildField(f, installed));
    details.appendChild(advGrid);
    form.after(details);
  }

  // Permission role: relabel options for current language; prefill from env.
  const roleSelect = $(".role-select", node);
  const opts = roleSelect.options;
  opts[0].textContent = t("viewer");
  opts[1].textContent = t("writer");
  if (installed) {
    const modeKey = c.id.toUpperCase() + "_MODE";
    const mode = installed.env[modeKey];
    if (mode === "writer" || mode === "viewer") roleSelect.value = mode;
  }

  // Notes + docs link.
  const notesEl = $(".card-notes", node);
  const noteParts = [];
  if (c.notes) noteParts.push(c.notes);
  notesEl.textContent = noteParts.join(" ");
  notesEl.hidden = noteParts.length === 0;
  if (c.docs_url) {
    const a = document.createElement("a");
    a.href = c.docs_url;
    a.target = "_blank";
    a.rel = "noopener";
    a.className = "docs-link";
    a.textContent = t("docs");
    notesEl.appendChild(document.createTextNode(" "));
    notesEl.appendChild(a);
    notesEl.hidden = false;
  }

  // Install / Update button label.
  const installBtn = $(".btn-install", node);
  $("span", installBtn).textContent = isOn ? t("update") : t("install");
  const removeBtn = $(".btn-remove", node);
  removeBtn.hidden = !isOn;

  // Expander toggle.
  const expander = $(".expander", node);
  const body = $(".card-body", node);
  expander.setAttribute("aria-expanded", "false");
  expander.addEventListener("click", () => {
    const willOpen = body.hidden;
    if (willOpen) {
      // Accordion: close any other open card so the dashboard never becomes an
      // endless column of expanded forms.
      grid.querySelectorAll(".card.is-open").forEach((other) => {
        if (other === node) return;
        other.classList.remove("is-open");
        const ob = $(".card-body", other);
        if (ob) ob.hidden = true;
        const oe = $(".expander", other);
        if (oe) { oe.classList.remove("open"); oe.setAttribute("aria-expanded", "false"); }
      });
    }
    body.hidden = !willOpen;
    expander.classList.toggle("open", willOpen);
    expander.setAttribute("aria-expanded", String(willOpen));
    node.classList.toggle("is-open", willOpen);
  });

  // Editing any field invalidates a prior successful test.
  node.addEventListener("input", () => { node.dataset.tested = ""; });

  // Actions.
  $(".btn-test", node).addEventListener("click", () => testConnection(c, node));
  installBtn.addEventListener("click", () => installConnector(c, node));
  removeBtn.addEventListener("click", () => removeConnector(c, node));

  return node;
}

// When a connector accepts EITHER a token OR username+password, show a method
// toggle and reveal only the chosen set, so users aren't faced with both at once.
// Detected generically by env suffix (_TOKEN / _USERNAME / _PASSWORD).
function setupAuthToggle(form, installed, node) {
  const tokenField = form.querySelector('.field[data-env$="_TOKEN"]');
  const userField = form.querySelector('.field[data-env$="_USERNAME"]');
  const passField = form.querySelector('.field[data-env$="_PASSWORD"]');
  if (!tokenField || !userField || !passField) return;

  // Default to token; switch to basic only if an install already used a username.
  let method = "token";
  if (installed && installed.env[userField.dataset.env]) method = "basic";

  const seg = document.createElement("div");
  seg.className = "auth-toggle";
  seg.innerHTML =
    `<span class="auth-label">${t("authMethod")}</span>` +
    `<div class="seg" role="group">` +
    `<button type="button" class="seg-btn" data-method="token">${t("authToken")}</button>` +
    `<button type="button" class="seg-btn" data-method="basic">${t("authBasic")}</button>` +
    `</div>`;
  tokenField.before(seg);

  function apply(m, fromUser) {
    method = m;
    tokenField.hidden = m !== "token";
    userField.hidden = m !== "basic";
    passField.hidden = m !== "basic";
    seg.querySelectorAll(".seg-btn").forEach((b) => {
      const on = b.dataset.method === m;
      b.classList.toggle("active", on);
      b.setAttribute("aria-pressed", String(on));
    });
    if (fromUser) node.dataset.tested = ""; // changing method invalidates a prior test
  }
  seg.querySelectorAll(".seg-btn").forEach((b) =>
    b.addEventListener("click", () => apply(b.dataset.method, true))
  );
  apply(method, false);
}

function buildField(f, installed) {
  const wrap = document.createElement("label");
  wrap.className = "field";
  wrap.dataset.env = f.env;

  const labelText = f.label + (f.required ? " *" : "");
  const labelEl = document.createElement("span");
  labelEl.className = "field-label";
  labelEl.textContent = labelText;
  wrap.appendChild(labelEl);

  const prefill =
    installed && installed.env[f.env] !== undefined ? String(installed.env[f.env]) : "";

  let input;
  if (f.kind === "bool") {
    input = document.createElement("input");
    input.type = "checkbox";
    input.className = "field-check";
    const checkedDefault = (f.default || "").toLowerCase() === "true";
    input.checked = prefill ? prefill.toLowerCase() === "true" : checkedDefault;
    wrap.classList.add("field-bool");
  } else if (f.kind === "select") {
    input = document.createElement("select");
    input.className = "field-input";
    for (const opt of f.options || []) {
      const o = document.createElement("option");
      o.value = opt;
      o.textContent = opt;
      input.appendChild(o);
    }
    input.value = prefill || f.default || (f.options && f.options[0]) || "";
  } else {
    input = document.createElement("input");
    input.type = f.kind === "secret" ? "password" : "text";
    input.className = "field-input";
    input.spellcheck = false;
    input.autocapitalize = "off";
    input.value = prefill || f.default || "";
    if (f.kind === "secret") input.autocomplete = "off";
  }
  input.dataset.env = f.env;
  input.dataset.kind = f.kind;
  if (f.required) input.dataset.required = "true";

  if (f.kind === "secret") {
    // Wrap the secret input with a Show/Hide toggle so users can verify a paste.
    const row = document.createElement("div");
    row.className = "secret-row";
    row.appendChild(input);
    const toggle = document.createElement("button");
    toggle.type = "button";
    toggle.className = "reveal-btn";
    toggle.textContent = t("show");
    toggle.setAttribute("aria-pressed", "false");
    toggle.addEventListener("click", () => {
      const showing = input.type === "text";
      input.type = showing ? "password" : "text";
      toggle.textContent = showing ? t("show") : t("hide");
      toggle.setAttribute("aria-pressed", String(!showing));
    });
    row.appendChild(toggle);
    wrap.appendChild(row);
  } else {
    wrap.appendChild(input);
  }

  if (f.help) {
    const hint = document.createElement("span");
    hint.className = "field-hint";
    hint.textContent = f.help;
    wrap.appendChild(hint);
  }
  return wrap;
}

/* ── Collect form values ────────────────────────────────────────────── */
function collectValues(node) {
  const values = {};
  let missing = false;
  let firstErr = null;
  $$(".field", node).forEach((w) => w.classList.remove("field-err"));
  $$(".field-input, .field-check", node).forEach((input) => {
    // Skip fields hidden by the auth-method toggle so the unused set (e.g. a
    // token when using username+password) is never submitted.
    const fieldWrap = input.closest(".field");
    if (fieldWrap && fieldWrap.hidden) return;
    input.removeAttribute("aria-invalid");
    const env = input.dataset.env;
    const kind = input.dataset.kind;
    const val = kind === "bool" ? (input.checked ? "true" : "false") : input.value.trim();
    if (input.dataset.required === "true" && kind !== "bool" && val === "") {
      missing = true;
      input.setAttribute("aria-invalid", "true");
      const w = input.closest(".field");
      if (w) w.classList.add("field-err");
      if (!firstErr) firstErr = input;
    }
    if (val !== "") values[env] = val;
  });
  if (firstErr) firstErr.focus(); // move focus to the first problem, not color alone
  return { values, missing };
}

// Normalize a Tauri rejection (string or object) into a readable message.
function errMsg(e) {
  return typeof e === "string" ? e : e && e.message ? e.message : String(e);
}

/* ── Test connection ────────────────────────────────────────────────── */
async function testConnection(c, node) {
  const status = $(".card-status", node);
  const { values, missing } = collectValues(node);
  if (missing) {
    setStatus(status, "err", t("missingRequired"));
    return;
  }
  const mode = $(".role-select", node).value;
  const btn = $(".btn-test", node);
  btn.disabled = true;
  setStatus(status, "info", t("testing"));
  try {
    const msg = await invoke("test_connection", { id: c.id, values, mode });
    node.dataset.tested = "ok";
    setStatus(status, "ok", msg || t("testOk"));
  } catch (e) {
    setStatus(status, "err", errMsg(e));
  } finally {
    btn.disabled = false;
  }
}

/* ── Install / Update ───────────────────────────────────────────────── */
async function installConnector(c, node) {
  const status = $(".card-status", node);
  const { values, missing } = collectValues(node);
  if (missing) {
    setStatus(status, "err", t("missingRequired"));
    return;
  }
  if (node.dataset.tested !== "ok" && !window.confirm(t("confirmUntested"))) return;
  const mode = $(".role-select", node).value;
  const installBtn = $(".btn-install", node);
  installBtn.disabled = true;
  setStatus(status, "info", t("installing"));

  try {
    await invoke("install_connector", { id: c.id, values, mode });
    installedById[c.id] = { command: "", env: { ...values, [c.id.toUpperCase() + "_MODE"]: mode } };
    node.dataset.state = "on";
    const pill = $(".state-pill", node);
    pill.textContent = t("on");
    pill.classList.add("on");
    $("span", installBtn).textContent = t("update");
    $(".btn-remove", node).hidden = false;
    setStatus(status, "ok", t("installed"));
    $(".restart-note", node).hidden = false;
    buildFilters();
  } catch (e) {
    setStatus(status, "err", errMsg(e));
  } finally {
    installBtn.disabled = false;
  }
}

/* ── Remove ─────────────────────────────────────────────────────────── */
async function removeConnector(c, node) {
  if (!window.confirm(t("confirmRemove").replace("{name}", c.name))) return;
  const status = $(".card-status", node);
  const removeBtn = $(".btn-remove", node);
  removeBtn.disabled = true;
  setStatus(status, "info", t("removing"));

  try {
    await invoke("uninstall_connector", { id: c.id });
    delete installedById[c.id];
    node.dataset.state = "off";
    const pill = $(".state-pill", node);
    pill.textContent = t("off");
    pill.classList.remove("on");
    $("span", $(".btn-install", node)).textContent = t("install");
    removeBtn.hidden = true;
    setStatus(status, "ok", t("removed"));
    $(".restart-note", node).hidden = false;
    buildFilters();
  } catch (e) {
    setStatus(status, "err", errMsg(e));
  } finally {
    removeBtn.disabled = false;
  }
}

/* ── Boot ───────────────────────────────────────────────────────────── */
async function init() {
  const loading = document.createElement("p");
  loading.className = "empty-state";
  loading.textContent = t("loadingConnectors");
  grid.appendChild(loading);
  try {
    await loadData();
    applyLang(); // applies chrome strings and calls render()
  } catch (e) {
    grid.innerHTML = "";
    setStatus(globalStatus, "err", t("loadError") + " " + errMsg(e));
  }
}

init();
