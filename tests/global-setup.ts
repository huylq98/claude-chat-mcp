import { copyFileSync, existsSync } from "node:fs";
import { join } from "node:path";

// Ensure the served site has the latest registry.json before tests run.
export default function globalSetup() {
  const root = process.cwd();
  const src = join(root, "registry.json");
  const dest = join(root, "site", "registry.json");
  if (existsSync(src)) {
    copyFileSync(src, dest);
  }
}
