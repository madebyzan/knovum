import { spawn } from "node:child_process";
import { resolve as resolvePath, dirname } from "node:path";
import { fileURLToPath } from "node:url";

import { prepareNativeRuntime } from "./native.mjs";

export async function prepareNativeWorker(repository) {
  await prepareNativeRuntime(repository);
  await new Promise((resolve, reject) => {
    const build = spawn(
      "cargo",
      ["build", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "-p", "knovum-inference-worker"],
      { cwd: repository, stdio: "inherit" },
    );
    build.on("error", reject);
    build.on("exit", (code) =>
      code === 0 ? resolve() : reject(new Error(`Native worker build failed (${code})`)),
    );
  });
}

if (process.argv[1] && resolvePath(process.argv[1]) === fileURLToPath(import.meta.url))
  await prepareNativeWorker(resolvePath(dirname(fileURLToPath(import.meta.url)), "../.."));
