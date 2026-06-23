// Generate crates/control-panel/resources/binaries.json: for each connector and
// OS, the sha256 + size of the release binary the app will download on demand.
//
// Usage:
//   node scripts/gen-binaries-json.mjs \
//     --win <dir> --mac-arm <dir> --mac-intel <dir> --linux <dir> \
//     --version <v> --out <path>
//
// Each <dir> holds that platform's connector binaries: `<id>.exe` for Windows,
// bare `<id>` for the others. `mac-arm` is Apple Silicon, `mac-intel` is x86_64.

import { readFileSync, writeFileSync, statSync } from "node:fs";
import { createHash } from "node:crypto";
import { join } from "node:path";

const IDS = [
  "confluence", "jira", "bitbucket", "airtable", "mysql", "mariadb",
  "clickhouse", "oracle", "gitlab", "postgres", "github", "jenkins",
  "redmine", "grafana", "elasticsearch", "mattermost", "mongodb", "sentry",
];

function arg(name, def) {
  const i = process.argv.indexOf(`--${name}`);
  if (i >= 0 && i + 1 < process.argv.length) return process.argv[i + 1];
  if (def !== undefined) return def;
  throw new Error(`missing required --${name}`);
}

const dirs = {
  win: arg("win"),
  "mac-arm": arg("mac-arm"),
  "mac-intel": arg("mac-intel"),
  linux: arg("linux"),
};
const version = arg("version");
const out = arg("out", "crates/control-panel/resources/binaries.json");

function meta(dir, id, plat) {
  const file = join(dir, plat === "win" ? `${id}.exe` : id);
  const buf = readFileSync(file);
  return {
    sha256: createHash("sha256").update(buf).digest("hex"),
    size: statSync(file).size,
  };
}

const binaries = {};
for (const id of IDS) {
  binaries[id] = {
    win: meta(dirs.win, id, "win"),
    "mac-arm": meta(dirs["mac-arm"], id, "mac-arm"),
    "mac-intel": meta(dirs["mac-intel"], id, "mac-intel"),
    linux: meta(dirs.linux, id, "linux"),
  };
}

writeFileSync(out, JSON.stringify({ version, binaries }, null, 2) + "\n");
console.log(`wrote ${out} for ${IDS.length} connectors (version ${version})`);
