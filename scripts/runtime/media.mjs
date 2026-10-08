import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { activateDirectory } from "./files.mjs";
import { mediaExecutable, mediaManifestName, validateMediaRuntime } from "./media-manifest.mjs";
import { runProducer } from "./producer.mjs";
import { hostRuntimeTarget } from "./profiles.mjs";

export async function prepareCoreMedia(repository) {
  const target = hostRuntimeTarget();
  const destination = join(repository, "src-tauri/resources/bundled-runtime", target);
  if (existsSync(join(destination, mediaManifestName))) {
    try {
      validateMediaRuntime(repository, destination, target);
      return;
    } catch (error) {
      console.log(`Rebuilding owned core media resource: ${error.message}`);
    }
  }
  const root = join(repository, ".runtime-development/core-media");
  mkdirSync(root, { recursive: true });
  const scratch = mkdtempSync(join(root, `${target}-`));
  try {
    console.log(`Preparing core FFmpeg media resource for ${target}`);
    const output = join(scratch, "produced");
    await runProducer(repository, ["--output", output, "--development"], "build_media.py");
    const runtime = join(output, "runtime");
    validateMediaRuntime(repository, runtime, target);
    activateDirectory(runtime, destination);
    execFileSync(join(destination, mediaExecutable(target)), ["-version"], { stdio: "ignore" });
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
  await runProducer(repository, process.argv.slice(2), "build_media.py");
}
