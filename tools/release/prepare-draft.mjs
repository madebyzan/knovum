import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { loadArtifacts, loadHumanInstallers, verifyArtifacts } from "./artifacts.mjs";
import { releaseRepository, stableVersion } from "./config.mjs";
import { createManifest } from "./manifest.mjs";

export function prepareDraft({ repository, version, directory, notesFile, publicKey }, execute = execFileSync) {
  const repo = releaseRepository(repository);
  const tag = `v${stableVersion(version)}`;
  const notes = readFileSync(notesFile, "utf8");
  const artifacts = loadArtifacts(directory, version);
  const humanInstallers = loadHumanInstallers(directory, version);
  verifyArtifacts(directory, artifacts, publicKey, execute);
  const manifest = createManifest({
    version, notes, publishedAt: new Date().toISOString(), artifacts,
    baseUrl: `https://github.com/${repo}/releases/download/${tag}`,
  });
  const privacy = execute("gh", ["repo", "view", repo, "--json", "isPrivate", "--jq", ".isPrivate"], { encoding: "utf8" }).trim();
  if (privacy !== "false") throw new Error("The distribution repository must be public; never embed GitHub credentials in the app");
  const temporary = mkdtempSync(join(tmpdir(), "knovum-release-"));
  try {
    const manifestFile = join(temporary, "latest.json");
    writeFileSync(manifestFile, JSON.stringify(manifest, null, 2) + "\n", { flag: "wx" });
    // Deliberately draft-only. A conflicting existing release is an error, never overwritten.
    execute("gh", ["release", "create", tag, "--repo", repo, "--draft", "--title", `Knovum ${version}`, "--notes-file", notesFile], { stdio: "pipe" });
    const files = artifacts.flatMap(({ name }) => [join(directory, name), join(directory, `${name}.sig`)]);
    execute("gh", ["release", "upload", tag, "--repo", repo, ...files, ...humanInstallers, manifestFile], { stdio: "pipe" });
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
  return { repository: repo, tag, manifest };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const [version, directory, notesFile, ...extra] = process.argv.slice(2);
    if (!version || !directory || !notesFile || extra.length) {
      throw new Error("Usage: node tools/release/prepare-draft.mjs VERSION ARTIFACT_DIRECTORY NOTES_FILE");
    }
    const result = prepareDraft({
      repository: process.env.KNOVUM_RELEASE_REPOSITORY, version,
      directory: resolve(directory), notesFile: resolve(notesFile), publicKey: process.env.KNOVUM_UPDATER_PUBKEY,
    });
    console.log(`Draft ${result.repository}@${result.tag} prepared. It is not published.`);
  } catch (error) {
    // execFileSync exceptions include child output: do not serialize them or credentials.
    console.error(error instanceof Error ? error.message.split("\n")[0] : "Release preparation failed");
    process.exitCode = 1;
  }
}
