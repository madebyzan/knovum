import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { run } from "@tauri-apps/cli";

import { prepareCoreMedia } from "../runtime/media.mjs";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const separator = args.indexOf("--");
const cliArgs = separator < 0 ? args : args.slice(0, separator);
const command = cliArgs.find((argument) => !argument.startsWith("-"));
const informational = cliArgs.some((argument) => ["--help", "-h", "--version", "-V"].includes(argument));
try {
  // Resource production precedes Tauri's bounded wait for the frontend server.
  if (command === "dev" && !informational) {
    await prepareCoreMedia(repository);
  }
  await run(args, "pnpm tauri");
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
