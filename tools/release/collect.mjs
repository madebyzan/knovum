import { copyFileSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

import { artifactSha256, safeArtifactFilename } from "./artifacts.mjs";
import { httpsUrl, publicKey, readVersion, releaseTargets, requireValue } from "./policy.mjs";

const root = resolve(import.meta.dirname, "../..");
const triple = process.argv[2];
const target = releaseTargets[triple];
if (!target) throw new Error("A supported Rust release target is required");
const version = readVersion(root);
const bundle = join(root, "src-tauri", "target", triple, "release", "bundle");
const mac = target.startsWith("darwin-");
const source = findOne(join(bundle, mac ? "macos" : "nsis"), mac ? ".app.tar.gz" : "-setup.exe");
const directory = join(root, "release-artifacts", target);
mkdirSync(directory, { recursive: true });
const file = safeArtifactFilename(`Knovum_${version}_${target}${mac ? ".app.tar.gz" : "-setup.exe"}`);
copyFileSync(source, join(directory, file));
copyFileSync(`${source}.sig`, join(directory, `${file}.sig`));
const artifact = {
  target,
  version,
  file,
  signature: readFileSync(`${source}.sig`, "utf8").trim(),
  sha256: await artifactSha256(source),
};
const installers = [];
if (mac) {
  const dmg = findOne(join(bundle, "dmg"), ".dmg");
  const installer = safeArtifactFilename(`Knovum_${version}_${target}.dmg`);
  copyFileSync(dmg, join(directory, installer));
  installers.push({ file: installer, sha256: await artifactSha256(dmg) });
}
writeFileSync(
  join(directory, "platform.json"),
  JSON.stringify(
    {
      version,
      sourceCommit: requireValue(process.env.GITHUB_SHA, "GITHUB_SHA"),
      endpoint: httpsUrl(process.env.KNOVUM_UPDATER_ENDPOINT, "KNOVUM_UPDATER_ENDPOINT").href,
      pubkey: publicKey(process.env.KNOVUM_UPDATER_PUBKEY),
      artifact,
      installers,
    },
    null,
    2,
  ),
);

function findOne(directoryPath, suffix) {
  const files = readdirSync(directoryPath, { withFileTypes: true }).filter(
    (entry) => entry.isFile() && entry.name.endsWith(suffix),
  );
  if (files.length !== 1) throw new Error(`Expected exactly one ${suffix} in ${directoryPath}`);
  return join(directoryPath, files[0].name);
}
