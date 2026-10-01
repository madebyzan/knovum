import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { releaseRepository, stableVersion } from "./config.mjs";

export function verifyVersions({ tag, tauriVersion, packageVersion, cargo }) {
  if (!tag.startsWith("v")) throw new Error("Release ref must be a vMAJOR.MINOR.PATCH tag");
  const version = stableVersion(tag.slice(1));
  const application = cargo.packages.find((entry) => entry.name === "knovum");
  if (tauriVersion !== version || packageVersion !== version || application?.version !== version) {
    throw new Error("Tag, Tauri, package.json and Cargo package versions must match");
  }
  return version;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.env.GITHUB_REF_TYPE !== "tag") throw new Error("Run the release workflow on a tag");
    releaseRepository(process.env.KNOVUM_RELEASE_REPOSITORY);
    const cargo = JSON.parse(
      execFileSync(
        "cargo",
        [
          "metadata",
          "--locked",
          "--no-deps",
          "--format-version",
          "1",
          "--manifest-path",
          "src-tauri/Cargo.toml",
        ],
        { encoding: "utf8" },
      ),
    );
    const version = verifyVersions({
      tag: process.env.GITHUB_REF_NAME ?? "",
      tauriVersion: JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8")).version,
      packageVersion: JSON.parse(readFileSync("package.json", "utf8")).version,
      cargo,
    });
    if (!readFileSync(`releases/${version}.md`, "utf8").trim()) throw new Error("Release notes are empty");
    console.log(version);
  } catch (error) {
    console.error(error instanceof Error ? error.message : "Release preflight failed");
    process.exitCode = 1;
  }
}
