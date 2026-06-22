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
    installed: "Saved. Connector is on.",
    removed: "Connector removed.",
    missingRequired: "Please fill in the required fields.",
    docs: "Documentation",
  },
  vi: {
    appName: "Claude Chat MCP",
    title: "Bang dieu khien connector",
    lede: "Bat hoac tat connector, nhap thong tin dang nhap, va chon vai tro quyen. Thay doi se duoc ghi truc tiep vao Claude Desktop.",
    configure: "Cau hinh",
    role: "Vai tro quyen",
    viewer: "Nguoi xem (chi doc)",
    writer: "Nguoi ghi (doc va ghi)",
    install: "Cai dat",
    update: "Cap nhat",
    remove: "Go bo",
    on: "Bat",
    off: "Tat",
    restartNote:
      "Bay gio hay thoat han va mo lai Claude Desktop de thay doi co hieu luc.",
    installing: "Dang cai dat...",
    removing: "Dang go bo...",
    installed: "Da luu. Connector dang bat.",
    removed: "Da go bo connector.",
    missingRequired: "Vui long dien cac truong bat buoc.",
    docs: "Tai lieu",
  },
};

let lang = "en";
const t = (key) => (STRINGS[lang] && STRINGS[lang][key]) || STRINGS.en[key] || key;

function applyLang() {
  document.documentElement.lang = lang;
  $$("[data-i18n]").forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });
  // Re-render connector cards so dynamic labels follow the language too.
  render();
}

$$(".lang-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    lang = btn.dataset.lang;
    $$(".lang-btn").forEach((b) => b.classList.toggle("active", b === btn));
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

function render() {
  grid.innerHTML = "";
  for (const c of connectors) grid.appendChild(buildCard(c));
}

function buildCard(c) {
  const node = tpl.content.firstElementChild.cloneNode(true);
  const installed = installedById[c.id];
  const isOn = !!installed;

  node.dataset.state = isOn ? "on" : "off";
  $(".card-group", node).textContent = c.group;
  $(".card-name", node).textContent = c.name;
  $(".card-desc", node).textContent = c.description;

  const pill = $(".state-pill", node);
  pill.textContent = isOn ? t("on") : t("off");
  pill.classList.toggle("on", isOn);

  // Build inputs for every auth field plus advanced field.
  const form = $(".fields", node);
  const fields = [...(c.auth_fields || []), ...(c.advanced_fields || [])];
  for (const f of fields) {
    form.appendChild(buildField(f, installed));
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
  expander.addEventListener("click", () => {
    const open = !body.hidden;
    body.hidden = open;
    expander.classList.toggle("open", !open);
  });
  if (isOn) {
    body.hidden = false;
    expander.classList.add("open");
  }

  // Actions.
  installBtn.addEventListener("click", () => installConnector(c, node));
  removeBtn.addEventListener("click", () => removeConnector(c, node));

  return node;
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
  wrap.appendChild(input);

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
  $$(".field-input, .field-check", node).forEach((input) => {
    const env = input.dataset.env;
    const kind = input.dataset.kind;
    let val;
    if (kind === "bool") {
      val = input.checked ? "true" : "false";
    } else {
      val = input.value.trim();
    }
    if (input.dataset.required === "true" && (val === "" || (kind === "bool" && !input.checked))) {
      missing = true;
    }
    if (val !== "") values[env] = val;
  });
  return { values, missing };
}

/* ── Install / Update ───────────────────────────────────────────────── */
async function installConnector(c, node) {
  const status = $(".card-status", node);
  const { values, missing } = collectValues(node);
  if (missing) {
    setStatus(status, "err", t("missingRequired"));
    return;
  }
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
  } catch (e) {
    setStatus(status, "err", String(e));
  } finally {
    installBtn.disabled = false;
  }
}

/* ── Remove ─────────────────────────────────────────────────────────── */
async function removeConnector(c, node) {
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
  } catch (e) {
    setStatus(status, "err", String(e));
  } finally {
    removeBtn.disabled = false;
  }
}

/* ── Boot ───────────────────────────────────────────────────────────── */
async function init() {
  try {
    await loadData();
    applyLang(); // applies chrome strings and calls render()
  } catch (e) {
    setStatus(globalStatus, "err", "Could not load connectors: " + e);
  }
}

init();
