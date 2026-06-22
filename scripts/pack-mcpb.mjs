// Package each connector as an official .mcpb Desktop Extension.
// Maps our connector manifest (registry.json) to the .mcpb manifest.json schema
// (binary server + user_config from auth_fields), bundles the binary/binaries,
// and runs `mcpb pack`. Output: dist/mcpb/<id>.mcpb
//
// Local (Windows-only, from target/release):
//   node scripts/pack-mcpb.mjs
// Universal (CI), --bins points at a dir with win/ mac/ linux/ subdirs of binaries:
//   node scripts/pack-mcpb.mjs --bins binaries
import { readFileSync, writeFileSync, mkdirSync, copyFileSync, rmSync, existsSync, chmodSync } from "node:fs";
import { join } from "node:path";
import { execFileSync } from "node:child_process";

const root = process.cwd();
const argv = process.argv.slice(2);
const binsIdx = argv.indexOf("--bins");
const binsDir = binsIdx >= 0 ? join(root, argv[binsIdx + 1]) : null; // universal mode if set
const profile = argv.includes("--debug") ? "debug" : "release";

const outRoot = join(root, "dist", "mcpb");
const iconPng = existsSync(join(root, "assets", "icon.png"))
  ? join(root, "assets", "icon.png")
  : join(outRoot, "_assets", "icon.png");

const registry = JSON.parse(readFileSync(join(root, "site", "registry.json"), "utf8"));

function userConfigAndEnv(c) {
  const user_config = {};
  const env = {};
  for (const f of [...(c.auth_fields || []), ...(c.advanced_fields || [])]) {
    const key = f.env.toLowerCase();
    const entry = {
      type: f.kind === "bool" ? "boolean" : "string",
      title: f.label || f.env,
      description: f.help || "",
      required: !!f.required,
    };
    if (f.kind === "secret") entry.sensitive = true;
    if (f.default !== undefined && f.default !== null) entry.default = String(f.default);
    user_config[key] = entry;
    env[f.env] = "${user_config." + key + "}";
  }
  return { user_config, env };
}

function baseManifest(c) {
  return {
    manifest_version: "0.3",
    name: c.id,
    display_name: c.name,
    version: "0.1.0",
    description: c.description || c.name,
    author: { name: "huylq98", url: "https://github.com/huylq98/claude-chat-mcp" },
    icon: "icon.png",
    tools: (c.tools || []).map((t) => ({ name: t.name, description: t.description })),
    keywords: ["mcp", "claude", (c.group || "").toLowerCase()].filter(Boolean),
  };
}

// ---- Windows-only bundle (local dev) ----
function packWindows(c, exe) {
  const { user_config, env } = userConfigAndEnv(c);
  const manifest = {
    ...baseManifest(c),
    server: {
      type: "binary",
      entry_point: `server/${c.binary}`,
      mcp_config: { command: `server/${c.binary}`, args: [], env },
    },
    user_config,
    compatibility: { platforms: ["win32"] },
  };
  const dir = join(outRoot, c.id);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(join(dir, "server"), { recursive: true });
  writeFileSync(join(dir, "manifest.json"), JSON.stringify(manifest, null, 2));
  copyFileSync(exe, join(dir, "server", `${c.binary}.exe`));
  if (existsSync(iconPng)) copyFileSync(iconPng, join(dir, "icon.png"));
  return dir;
}

// ---- Universal bundle (CI): one .mcpb with all three OS binaries ----
function packUniversal(c) {
  const { user_config, env } = userConfigAndEnv(c);
  const rel = { win: `server/win/${c.binary}.exe`, mac: `server/mac/${c.binary}`, linux: `server/linux/${c.binary}` };
  const manifest = {
    ...baseManifest(c),
    server: {
      type: "binary",
      entry_point: rel.linux,
      mcp_config: {
        command: rel.linux,
        args: [],
        env,
        platform_overrides: {
          win32: { command: rel.win },
          darwin: { command: rel.mac },
          linux: { command: rel.linux },
        },
      },
    },
    user_config,
    compatibility: { platforms: ["win32", "darwin", "linux"] },
  };
  const dir = join(outRoot, c.id);
  rmSync(dir, { recursive: true, force: true });
  for (const p of ["win", "mac", "linux"]) mkdirSync(join(dir, "server", p), { recursive: true });
  writeFileSync(join(dir, "manifest.json"), JSON.stringify(manifest, null, 2));
  const srcs = {
    win: join(binsDir, "win", `${c.binary}.exe`),
    mac: join(binsDir, "mac", c.binary),
    linux: join(binsDir, "linux", c.binary),
  };
  for (const [p, src] of Object.entries(srcs)) {
    if (!existsSync(src)) throw new Error(`missing ${p} binary for ${c.id}: ${src}`);
    const dest = join(dir, "server", p, p === "win" ? `${c.binary}.exe` : c.binary);
    copyFileSync(src, dest);
    if (p !== "win") chmodSync(dest, 0o755); // preserve exec bit in the zip
  }
  if (existsSync(iconPng)) copyFileSync(iconPng, join(dir, "icon.png"));
  return dir;
}

const built = [];
for (const c of registry.connectors) {
  let dir;
  if (binsDir) {
    dir = packUniversal(c);
  } else {
    const exe = join(root, "target", profile, `${c.binary}.exe`);
    if (!existsSync(exe)) { console.warn(`skip ${c.id}: missing ${exe}`); continue; }
    dir = packWindows(c, exe);
  }
  const out = join(outRoot, `${c.id}.mcpb`);
  execFileSync("npx", ["mcpb", "pack", dir, out], { stdio: "inherit", shell: true });
  built.push(`${c.id}.mcpb`);
}
console.log(`\nPacked ${built.length} ${binsDir ? "universal" : "windows"} bundles: ${built.join(", ")}`);
