import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

import { fileDigest, safeFilename, verifyReleaseFiles } from "./release-files.mjs";
import { github, requireDistributionRepository } from "./release-github.mjs";
import { httpsUrl, publicKey, readVersion, requireValue } from "./release-policy.mjs";
import { createManifest } from "./update-manifest.mjs";

const root = resolve(import.meta.dirname, "..");
const version = readVersion(root);
const sourceCommit = requireValue(process.env.GITHUB_SHA, "GITHUB_SHA");
const endpoint = httpsUrl(process.env.KNOVUM_UPDATER_ENDPOINT, "KNOVUM_UPDATER_ENDPOINT").href;
const pubkey = publicKey(process.env.KNOVUM_UPDATER_PUBKEY);
const repository = await requireDistributionRepository(process.env.KNOVUM_DISTRIBUTION_REPOSITORY);
const tag = `v${version}`;
const existingRelease = await github(`/repos/${repository}/releases/tags/${tag}`, { allowMissing: true });
const existingTag = await github(`/repos/${repository}/git/ref/tags/${tag}`, { allowMissing: true });
if (existingRelease || existingTag) {
  throw new Error("The release version already exists; never replace a published or draft candidate");
}
const input = resolve(process.argv[2]);
const notes = readFileSync(resolve(process.argv[3]), "utf8").trim();
const output = join(root, "release-candidate");
mkdirSync(output, { recursive: false });
const artifacts = [];
const installers = [];
for (const entry of readdirSync(input, { withFileTypes: true })) {
  if (!entry.isDirectory()) throw new Error("Expected one release artifact directory per target");
  const directory = join(input, entry.name);
  const part = JSON.parse(readFileSync(join(directory, "platform.json"), "utf8"));
  if (part.version !== version || part.sourceCommit !== sourceCommit || part.endpoint !== endpoint || part.pubkey !== pubkey) {
    throw new Error("All platform artifacts must share the release commit, version, endpoint and public key");
  }
  const payloads = [part.artifact, ...part.installers];
  for (const payload of payloads) {
    const name = safeFilename(payload.file);
    const source = join(directory, name);
    if (await fileDigest(source) !== payload.sha256) throw new Error(`Artifact hash mismatch: ${name}`);
    copyFileSync(source, join(output, name));
  }
  copyFileSync(join(directory, `${safeFilename(part.artifact.file)}.sig`), join(output, `${part.artifact.file}.sig`));
  artifacts.push(part.artifact);
  installers.push(...part.installers);
}
const candidate = { version, sourceCommit, endpoint, pubkey, notes, pubDate: new Date().toISOString(), artifacts, installers };
createManifest({ ...candidate, baseUrl: `https://github.com/${repository}/releases/download/${tag}/` });
await verifyReleaseFiles(output, artifacts, pubkey);
writeFileSync(join(output, "candidate.json"), `${JSON.stringify(candidate, null, 2)}\n`);
execFileSync("gh", ["release", "create", tag, "--repo", repository, "--draft", "--title", `Knovum ${version}`, "--notes-file", resolve(process.argv[3])], { stdio: "inherit" });
execFileSync("gh", ["release", "upload", tag, "--repo", repository, ...readdirSync(output).map((file) => join(output, file))], { stdio: "inherit" });
console.log(`Draft ${repository} ${tag} is ready for signed installer and upgrade acceptance.`);
