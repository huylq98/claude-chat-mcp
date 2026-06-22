// Package each connector as an official .mcpb Desktop Extension.
// Maps our connector manifest (registry.json) to the .mcpb manifest.json schema
// (binary server + user_config from auth_fields), bundles the release binary,
// and runs `mcpb pack`. Output: dist/mcpb/<id>.mcpb
//
// Usage: node scripts/pack-mcpb.mjs            (release binaries)
//        node scripts/pack-mcpb.mjs --debug
import { readFileSync, writeFileSync, mkdirSync, copyFileSync, rmSync, existsSync } from "node:fs";
import { join } from "node:path";
import { execFileSync } from "node:child_process";

const root = process.cwd();
const profile = process.argv.includes("--debug") ? "debug" : "release";
const binDir = join(root, "target", profile);
const outRoot = join(root, "dist", "mcpb");
const iconPng = join(outRoot, "_assets", "icon.png");

const registry = JSON.parse(readFileSync(join(root, "site", "registry.json"), "utf8"));

function toManifest(c) {
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
  return {
    manifest_version: "0.3",
    name: c.id,
    display_name: c.name,
    version: "0.1.0",
    description: c.description || c.name,
    author: { name: "huylq98", url: "https://github.com/huylq98/claude-chat-mcp" },
    icon: "icon.png",
    server: {
      type: "binary",
      entry_point: `server/${c.binary}`,
      mcp_config: { command: `server/${c.binary}`, args: [], env },
    },
    user_config,
    tools: (c.tools || []).map((t) => ({ name: t.name, description: t.description })),
    // Only Windows binaries are bundled here; macOS/Linux bundles need their own
    // CI-built binaries added under server/ with platform_overrides.
    compatibility: { platforms: ["win32"] },
    keywords: ["mcp", "claude", (c.group || "").toLowerCase()].filter(Boolean),
  };
}

const built = [];
for (const c of registry.connectors) {
  const exe = join(binDir, `${c.binary}.exe`);
  if (!existsSync(exe)) {
    console.warn(`skip ${c.id}: missing ${exe} (build --release first)`);
    continue;
  }
  const dir = join(outRoot, c.id);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(join(dir, "server"), { recursive: true });
  writeFileSync(join(dir, "manifest.json"), JSON.stringify(toManifest(c), null, 2));
  copyFileSync(exe, join(dir, "server", `${c.binary}.exe`));
  if (existsSync(iconPng)) copyFileSync(iconPng, join(dir, "icon.png"));
  const out = join(outRoot, `${c.id}.mcpb`);
  execFileSync("npx", ["mcpb", "pack", dir, out], { stdio: "inherit", shell: true });
  built.push(`${c.id}.mcpb`);
}
console.log(`\nPacked ${built.length} bundles into dist/mcpb/: ${built.join(", ")}`);
