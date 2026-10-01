import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { verifyArtifacts } from "./artifacts.mjs";
import { httpsUrl } from "./config.mjs";
import { artifactName, relocateManifest } from "./manifest.mjs";

/** Verify deployed bytes before replacing the stable discovery manifest. */
export function verifyHosted(manifest, key, execute = execFileSync) {
  // Reuse the schema/target/name checks; this does not alter the supplied manifest.
  relocateManifest(manifest, "https://validation.invalid/artifacts");
  const temporary = mkdtempSync(join(tmpdir(), "knovum-hosted-update-"));
  try {
    const artifacts = Object.entries(manifest.platforms).map(([target, artifact]) => {
      const name = artifactName(manifest.version, target);
      execute(
        "curl",
        [
          "--fail",
          "--location",
          "--silent",
          "--show-error",
          "--proto",
          "=https",
          "--proto-redir",
          "=https",
          "--connect-timeout",
          "10",
          "--max-time",
          "1800",
          "--output",
          join(temporary, name),
          httpsUrl(artifact.url, "artifact URL"),
        ],
        { stdio: "pipe" },
      );
      return { name, signature: artifact.signature };
    });
    verifyArtifacts(temporary, artifacts, key, execute);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const [path, ...extra] = process.argv.slice(2);
    if (!path || extra.length) throw new Error("Usage: node tools/release/verify-hosted.mjs MANIFEST_FILE");
    verifyHosted(JSON.parse(readFileSync(path, "utf8")), process.env.KNOVUM_UPDATER_PUBKEY);
    console.log("All hosted updater artifacts passed signature verification.");
  } catch {
    console.error("Hosted artifact verification failed; do not promote the manifest.");
    process.exitCode = 1;
  }
}
