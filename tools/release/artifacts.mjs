import { execFileSync } from "node:child_process";
import {
  constants,
  copyFileSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { publicKey } from "./config.mjs";
import { artifactName, releaseTargets, signatureText } from "./manifest.mjs";

function requireFile(path) {
  if (!lstatSync(path).isFile()) throw new Error(`Expected a regular release artifact: ${path}`);
}

export function stageTarget({ bundleDirectory, outputDirectory, version, target }) {
  const name = artifactName(version, target);
  const mac = target.startsWith("darwin-");
  const directory = join(bundleDirectory, mac ? "macos" : "nsis");
  const suffix = mac ? ".app.tar.gz" : "-setup.exe";
  const candidates = readdirSync(directory).filter((entry) => entry.endsWith(suffix));
  if (candidates.length !== 1) throw new Error(`Expected exactly one updater artifact for ${target}`);
  const path = join(directory, candidates[0]);
  const humanInstaller = mac ? singleDmg(bundleDirectory) : undefined;
  requireFile(path);
  requireFile(`${path}.sig`);
  signatureText(readFileSync(`${path}.sig`, "utf8"));
  mkdirSync(outputDirectory, { recursive: true });
  // No overwrite: a release version must never silently acquire different bytes.
  copyFileSync(path, join(outputDirectory, name), constants.COPYFILE_EXCL);
  copyFileSync(`${path}.sig`, join(outputDirectory, `${name}.sig`), constants.COPYFILE_EXCL);
  if (humanInstaller) {
    copyFileSync(
      humanInstaller,
      join(outputDirectory, name.replace(".app.tar.gz", ".dmg")),
      constants.COPYFILE_EXCL,
    );
  }
}

function singleDmg(bundleDirectory) {
  const directory = join(bundleDirectory, "dmg");
  const files = readdirSync(directory).filter((name) => name.endsWith(".dmg"));
  if (files.length !== 1) throw new Error("Expected exactly one signed macOS DMG");
  const path = join(directory, files[0]);
  requireFile(path);
  return path;
}

export function loadHumanInstallers(directory, version) {
  return releaseTargets
    .filter((target) => target.startsWith("darwin-"))
    .map((target) => {
      const path = join(directory, artifactName(version, target).replace(".app.tar.gz", ".dmg"));
      requireFile(path);
      return path;
    });
}

export function loadArtifacts(directory, version) {
  return releaseTargets.map((target) => {
    const name = artifactName(version, target);
    requireFile(join(directory, name));
    requireFile(join(directory, `${name}.sig`));
    return {
      target,
      name,
      signature: readFileSync(join(directory, `${name}.sig`), "utf8").trim(),
    };
  });
}

/** Use the maintained minisign CLI rather than implementing cryptographic verification. */
export function verifyArtifacts(directory, artifacts, encodedKey, execute = execFileSync) {
  const { minisign } = publicKey(encodedKey);
  const temporary = mkdtempSync(join(tmpdir(), "knovum-update-verification-"));
  try {
    for (const artifact of artifacts) {
      const signatureFile = join(temporary, `${artifact.name}.sig`);
      writeFileSync(signatureFile, signatureText(artifact.signature).text, { flag: "wx" });
      execute("minisign", ["-Vm", join(directory, artifact.name), "-P", minisign, "-x", signatureFile], {
        stdio: "pipe",
      });
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}
