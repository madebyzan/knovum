import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { activateDirectory } from "./files.mjs";
import {
  mediaExecutable,
  mediaManifestName,
  validateMediaRuntime,
  writeMediaManifest,
} from "./media-manifest.mjs";
import { MediaRuntimeContentError } from "./media-runtime-content.mjs";
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
      if (!(error instanceof MediaRuntimeContentError)) throw error;
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
  const args = process.argv.slice(2);
  if (args[0] === "--write-manifest") {
    if (args.length !== 4)
      throw new Error("Usage: media.mjs --write-manifest <directory> <target> <producer-json>");
    writeMediaManifest(repository, args[1], args[2], JSON.parse(readFileSync(args[3], "utf8")));
  } else {
    await runProducer(repository, args, "build_media.py");
  }
}
