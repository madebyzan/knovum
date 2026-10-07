import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const cli = createRequire(import.meta.url).resolve("@tauri-apps/cli/tauri.js");
execFileSync(process.execPath, [cli, ...args], {
  cwd: repository,
  stdio: "inherit",
});
