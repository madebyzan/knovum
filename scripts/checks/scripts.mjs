import { spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export function scriptUnitTests(root) {
  return readdirSync(root, { recursive: true })
    .filter((path) => path.endsWith(".test.mjs") && path.split(/[\\/]/u).includes("__tests__"))
    .map((path) => join(root, path))
    .toSorted();
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const tests = scriptUnitTests(fileURLToPath(new URL("../", import.meta.url)));
  if (!tests.length) throw new Error("No script unit tests found");
  const testExecution = spawnSync(process.execPath, ["--test", ...tests], { stdio: "inherit" });
  if (testExecution.error) throw testExecution.error;
  process.exitCode = testExecution.status ?? 1;
}
