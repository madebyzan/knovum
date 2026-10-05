import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { isDeepStrictEqual } from "node:util";

import { downloadPublicFile, fileDigest, safeFilename, verifyReleaseFiles } from "./files.mjs";
import { github, readStableChannel, requireDistributionRepository, writeStableChannel } from "./github.mjs";
import { httpsUrl, publicKey, requireValue, stableVersion } from "./policy.mjs";
import { assertPromotion, createManifest } from "./update-manifest.mjs";

const version = stableVersion(process.env.RELEASE_VERSION);
const repository = await requireDistributionRepository(process.env.KNOVUM_DISTRIBUTION_REPOSITORY);
const endpoint = httpsUrl(process.env.KNOVUM_UPDATER_ENDPOINT, "KNOVUM_UPDATER_ENDPOINT").href;
const pubkey = publicKey(process.env.KNOVUM_UPDATER_PUBKEY);
// The protected release environment approves this evidence before granting its token.
httpsUrl(
  requireValue(process.env.UPGRADE_EVIDENCE_URL, "UPGRADE_EVIDENCE_URL"),
  "Upgrade acceptance evidence",
);
const tag = `v${version}`;
const release = await github(`/repos/${repository}/releases/tags/${tag}`);
const directory = mkdtempSync(join(tmpdir(), "knovum-promote-"));
try {
  execFileSync("gh", ["release", "download", tag, "--repo", repository, "--dir", directory], {
    stdio: "inherit",
  });
  const candidate = JSON.parse(readFileSync(join(directory, "candidate.json"), "utf8"));
  if (candidate.version !== version || candidate.endpoint !== endpoint || candidate.pubkey !== pubkey) {
    throw new Error("Candidate does not match the approved version, permanent endpoint and signing key");
  }
  if (release.prerelease) throw new Error("A prerelease cannot be promoted to stable");
  await verifyReleaseFiles(directory, candidate.artifacts, pubkey);
  await Promise.all(
    candidate.installers.map(async (installer) => {
      if ((await fileDigest(join(directory, safeFilename(installer.file)))) !== installer.sha256) {
        throw new Error("Installer hash does not match the approved candidate");
      }
    }),
  );
  const baseUrl =
    process.env.ARTIFACT_BASE_URL?.trim() || `https://github.com/${repository}/releases/download/${tag}/`;
  const manifest = createManifest({ ...candidate, baseUrl });
  const previous = await readStableChannel(repository);
  const current = previous ? JSON.parse(Buffer.from(previous.content, "base64").toString("utf8")) : null;
  assertPromotion(current, manifest);
  if (release.draft) {
    await github(`/repos/${repository}/releases/${release.id}`, {
      method: "PATCH",
      body: { draft: false, make_latest: "true" },
    });
  }
  // Verify the anonymous path, including a new COS/CDN host, before exposing it to old clients.
  await Promise.all(
    candidate.artifacts.map(async (artifact) => {
      const file = join(directory, `public-${safeFilename(artifact.file)}`);
      downloadPublicFile(manifest.platforms[artifact.target].url, file);
      if ((await fileDigest(file)) !== artifact.sha256)
        throw new Error("Public artifact differs from the approved candidate");
    }),
  );
  await writeStableChannel(repository, previous, manifest);
  const published = join(directory, "published-channel.json");
  downloadPublicFile(endpoint, published);
  const observed = JSON.parse(readFileSync(published, "utf8"));
  if (!isDeepStrictEqual(observed, manifest)) {
    throw new Error(
      "The permanent endpoint is not serving the promoted manifest; check hosting and cache before retrying",
    );
  }
  console.log(`Promoted ${version}. Permanent discovery verified: ${endpoint}`);
} finally {
  rmSync(directory, { recursive: true, force: true });
}
