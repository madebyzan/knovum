import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { parseArgs } from "node:util";

import { signCompiledFiles } from "./compiled-signing.mjs";
import { listRuntimeFiles, sha256File } from "./files.mjs";
import { installStaticMedia, requireStaticMedia } from "./media-binary.mjs";
import { validateMediaLicenseReview } from "./media-license-policy.mjs";
import { mediaExecutable, writeMediaManifest } from "./media-manifest.mjs";
import { mediaSmoke } from "./media-smoke.mjs";
import { hostRuntimeTarget } from "./profiles.mjs";

export async function buildCoreMedia(repository, args) {
  const { output, archiveUrl, signingMode } = buildArguments(args);
  const target = hostRuntimeTarget();
  const mode = signingMode ?? (target.startsWith("macos-") ? "adhoc" : "unsigned");
  const source = JSON.parse(readFileSync(join(repository, "scripts/runtime/media-source.json"), "utf8"));
  const review = validateMediaLicenseReview(repository, target, { requireApproved: mode === "production" });
  mkdirSync(dirname(output), { recursive: true });
  const scratch = mkdtempSync(join(dirname(output), "media-build-"));
  try {
    const runtime = join(scratch, "runtime");
    const executable = await installStaticMedia(source, target, scratch, runtime);
    requireStaticMedia(executable, target, repository);
    const relocated = join(scratch, "relocated media 中文");
    renameSync(runtime, relocated);
    const signing = signCompiledFiles(relocated, target, mode);
    mediaSmoke(relocated, target, source.version);
    const producer = {
      schemaVersion: 1,
      target,
      version: source.version,
      source: source.targets[target],
      compiledBinaryFiles: [mediaExecutable(target)],
      appCommit: execFileSync("git", ["rev-parse", "HEAD"], { cwd: repository, encoding: "utf8" }).trim(),
      relocationVerified: true,
      signing,
      licenseReview: review,
    };
    const evidence = join(relocated, "build-evidence");
    mkdirSync(evidence);
    for (const name of [...source.producerSources, "media-source.json", "media-license-review.json"])
      copyFileSync(join(repository, "scripts/runtime", name), join(evidence, name));
    writeMediaManifest(repository, relocated, target, producer);
    const publication = join(scratch, "publication");
    mkdirSync(publication);
    if (!archiveUrl) {
      renameSync(relocated, join(publication, "runtime"));
      renameSync(publication, output);
      try {
        mediaSmoke(join(output, "runtime"), target, source.version);
      } catch (error) {
        rmSync(output, { recursive: true, force: true });
        throw error;
      }
      return;
    }
    const artifact = join(publication, `ffmpeg-${target}.tar.gz`);
    execFileSync("tar", ["-czf", artifact, "-C", relocated, ...listRuntimeFiles(relocated)]);
    const descriptor = {
      schemaVersion: 1,
      runtimeId: "ffmpeg",
      target,
      version: source.version,
      sourceSha256: source.targets[target].sha256,
      licenseReviewSha256: review.sha256,
      archiveUrl,
      size: statSync(artifact).size,
      sha256: sha256File(artifact),
      manifestSha256: sha256File(join(relocated, "media-manifest.json")),
    };
    writeFileSync(
      join(publication, `ffmpeg-${target}-descriptor.json`),
      `${JSON.stringify(descriptor, null, 2)}\n`,
    );
    renameSync(publication, output);
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

function buildArguments(args) {
  const { values } = parseArgs({
    args,
    options: {
      output: { type: "string" },
      signing: { type: "string" },
      "archive-url": { type: "string" },
      development: { type: "boolean" },
    },
  });
  if (
    !values.output ||
    Boolean(values.development) === Boolean(values["archive-url"]) ||
    (values.development && values.signing) ||
    (values["archive-url"] && !values.signing)
  )
    throw new Error("Media build requires --output and either --development or --archive-url with --signing");
  const output = resolve(values.output);
  if (existsSync(output)) throw new Error("Media output already exists; use a fresh destination");
  const archiveUrl = values["archive-url"];
  validatePublicationUrl(archiveUrl);
  return { output, archiveUrl, signingMode: values.signing };
}

function validatePublicationUrl(archiveUrl) {
  if (archiveUrl) {
    const url = new URL(archiveUrl);
    if (url.protocol !== "https:" || url.username || url.password || url.hash)
      throw new Error("Media publication requires HTTPS");
  }
}
