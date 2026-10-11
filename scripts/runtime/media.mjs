import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { signCompiledFiles } from "./compiled-signing.mjs";
import { activateDirectory } from "./files.mjs";
import { installStaticMedia, requireStaticMedia } from "./media-binary.mjs";
import { validateMediaLicenseReview } from "./media-license-policy.mjs";
import {
  mediaExecutable,
  mediaManifestName,
  validateMediaRuntime,
  writeMediaManifest,
} from "./media-manifest.mjs";
import { MediaRuntimeContentError } from "./media-runtime-content.mjs";
import { mediaSmoke } from "./media-smoke.mjs";
import { hostRuntimeTarget } from "./profiles.mjs";

export async function prepareCoreMedia(repository, { production = false } = {}) {
  const target = hostRuntimeTarget();
  const destination = join(repository, "src-tauri/resources/bundled-runtime", target);
  const review = validateMediaLicenseReview(repository, target, { requireApproved: production });
  if (reusableMedia(repository, destination, target, production)) return;
  const root = join(repository, ".runtime-development/core-media");
  mkdirSync(root, { recursive: true });
  const scratch = mkdtempSync(join(root, `${target}-`));
  try {
    console.log(`Preparing core FFmpeg media resource for ${target}`);
    const source = JSON.parse(readFileSync(join(repository, "scripts/runtime/media-source.json"), "utf8"));
    const runtime = join(scratch, "runtime");
    const executable = await installStaticMedia(source, target, scratch, runtime);
    requireStaticMedia(executable, target, repository);
    const relocated = join(scratch, "media 中文");
    renameSync(runtime, relocated);
    const mode = production ? "production" : target.startsWith("macos-") ? "adhoc" : "unsigned";
    const signing = signCompiledFiles(relocated, target, mode);
    mediaSmoke(relocated, target, source.version);
    const evidence = join(relocated, "build-evidence");
    mkdirSync(evidence);
    for (const name of [...source.producerSources, "media-source.json", "media-license-review.json"])
      copyFileSync(join(repository, "scripts/runtime", name), join(evidence, name));
    writeMediaManifest(repository, relocated, target, {
      schemaVersion: 1,
      target,
      version: source.version,
      source: source.targets[target],
      compiledBinaryFiles: [mediaExecutable(target)],
      appCommit: execFileSync("git", ["rev-parse", "HEAD"], { cwd: repository, encoding: "utf8" }).trim(),
      relocationVerified: true,
      signing,
      licenseReview: review,
    });
    activateDirectory(relocated, destination);
    execFileSync(join(destination, mediaExecutable(target)), ["-version"], { stdio: "ignore" });
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

function reusableMedia(repository, destination, target, production) {
  if (!existsSync(join(destination, mediaManifestName))) return false;
  try {
    const manifest = validateMediaRuntime(repository, destination, target);
    return !production || manifest.producer.signing.mode === "production";
  } catch (error) {
    if (!(error instanceof MediaRuntimeContentError)) throw error;
    console.log(`Rebuilding owned core media resource: ${error.message}`);
    return false;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.length !== 2) throw new Error("Core media is prepared directly; no publication options");
  await prepareCoreMedia(resolve(dirname(fileURLToPath(import.meta.url)), "../.."));
}
