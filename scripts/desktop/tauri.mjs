import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { run } from "@tauri-apps/cli";

import { buildDesktopDependencies } from "./cargo.mjs";
import { buildOptions, lockedRunnerArguments } from "./options.mjs";
import { packageDesktop, packagingConfiguration } from "./package.mjs";

const repository = fileURLToPath(new URL("../../", import.meta.url));
const [command, ...args] = process.argv.slice(2);
if (!["dev", "build"].includes(command) || args.includes("--help") || args.includes("-h")) {
  await run(process.argv.slice(2), "node scripts/desktop/tauri.mjs");
} else {
  const options = buildOptions(args, command);
  const configuration = command === "build" ? packagingConfiguration(repository, options) : undefined;
  execFileSync("uv", ["sync", "--project", "python", "--locked"], { cwd: repository, stdio: "inherit" });
  const dependencies = buildDesktopDependencies(repository, options);
  if (command === "build") await packageDesktop(repository, options, dependencies, configuration);
  else await run(["dev", ...lockedRunnerArguments(options.args)], "pnpm desktop:dev");
}
