#!/usr/bin/env node

import { execFileSync, spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const [binary, ...args] = process.argv.slice(2);
const identity = process.env.KNOVUM_DEV_SIGNING_IDENTITY;
if (!binary || !identity || identity === "-") {
  throw new Error("A compiled binary and a stable macOS development signing identity are required");
}
const repository = join(dirname(fileURLToPath(import.meta.url)), "..");
const { identifier } = JSON.parse(readFileSync(join(repository, "src-tauri", "tauri.conf.json"), "utf8"));

execFileSync(
  "codesign",
  ["--force", "--sign", identity, "--identifier", identifier, "--timestamp=none", binary],
  { stdio: "inherit" },
);
execFileSync("codesign", ["--verify", "--strict", binary], { stdio: "inherit" });

const app = spawn(binary, args, { stdio: "inherit" });
for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => app.kill(signal));
}
app.on("error", (error) => {
  console.error(error);
  process.exitCode = 1;
});
app.on("exit", (code, signal) => {
  process.exitCode = code ?? (signal ? 1 : 0);
});
