import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { sha256File } from "../files.mjs";
import { mediaLockPath } from "../media-manifest.mjs";
import { runtimeProfiles, runtimeTargets } from "../profiles.mjs";
import { publicationSources, writePublication } from "../publication.mjs";

const sources = Object.fromEntries(
  ["PYTHON_RUNTIME", "PYTHON_RUNTIME_DESCRIPTOR", "PYTHON_RUNTIME_EVIDENCE"].map((name) => [
    name,
    { url: `https://release.example/${name}`, expectedHash: "a".repeat(64) },
  ]),
);
const publication = { schemaVersion: 1, entries: [{ target: "windows-x64", profile: "agent", sources }] };
test("provisioning selects exactly the published platform and profile", () => {
  assert.equal(publicationSources(publication, "windows-x64", "agent").length, 3);
  assert.throws(() => publicationSources(publication, "macos-x64", "agent"), /exactly one/);
  assert.throws(
    () =>
      publicationSources(
        { ...publication, entries: [...publication.entries, ...publication.entries] },
        "windows-x64",
        "agent",
      ),
    /exactly one/,
  );
});

test("publication binds every native target and profile to its actual archive", (context) => {
  const directory = mkdtempSync(join(tmpdir(), "knovum-publication-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  const baseUrl = "https://release.example/immutable-version";
  for (const target of Object.values(runtimeTargets)) {
    const mediaPrefix = `ffmpeg-${target}`;
    writeFileSync(join(directory, `${mediaPrefix}.tar.gz`), `media/${target}`);
    writeFileSync(
      join(directory, `${mediaPrefix}-descriptor.json`),
      JSON.stringify({
        schemaVersion: 1,
        runtimeId: "ffmpeg",
        target,
        version: "9.0.1",
        archiveUrl: `${baseUrl}/${mediaPrefix}.tar.gz`,
        size: Buffer.byteLength(`media/${target}`),
        sha256: sha256File(join(directory, `${mediaPrefix}.tar.gz`)),
        manifestSha256: "a".repeat(64),
        nativeLockSha256: sha256File(
          mediaLockPath(fileURLToPath(new URL("../../../", import.meta.url)), target),
        ),
      }),
    );
    for (const profile of runtimeProfiles) {
      const prefix = `python-${profile}-${target}`;
      writeFileSync(join(directory, `${prefix}.tar.gz`), `${profile}/${target}`);
      writeFileSync(join(directory, `${prefix}-evidence.tar.gz`), "build evidence");
      writeFileSync(
        join(directory, `${prefix}-descriptor.json`),
        JSON.stringify({
          profile,
          target,
          archiveUrl: `${baseUrl}/${prefix}.tar.gz`,
          sha256: sha256File(join(directory, `${prefix}.tar.gz`)),
        }),
      );
    }
  }
  writePublication(directory, baseUrl);
  const result = JSON.parse(readFileSync(join(directory, "runtime-publication.json"), "utf8"));
  assert.equal(result.entries.length, 18);
  assert.equal(publicationSources(result, "macos-arm64", "voice-generation").length, 3);
  assert.equal(publicationSources(result, "macos-arm64", "light-inference").length, 5);
  assert.equal(publicationSources(result, "windows-x64", "core-media").length, 2);
  const incomplete = { ...result, entries: result.entries.filter((entry) => entry.profile !== "core-media") };
  assert.throws(() => publicationSources(incomplete, "macos-arm64", "light-inference"), /core-media/u);

  const archive = join(directory, "python-agent-windows-x64.tar.gz");
  writeFileSync(archive, "changed after descriptor creation");
  assert.throws(() => writePublication(directory, baseUrl), /differs from its produced archive/);
  rmSync(archive);
  assert.throws(() => writePublication(directory, baseUrl), /ENOENT/);
});
test("publication cannot supply unhashed or non-HTTPS resources", () => {
  const entry = publication.entries[0];
  for (const invalid of [{ url: "http://release.example/runtime" }, { expectedHash: "invalid" }]) {
    const bad = {
      ...entry,
      sources: { ...sources, PYTHON_RUNTIME: { ...sources.PYTHON_RUNTIME, ...invalid } },
    };
    assert.throws(
      () => publicationSources({ schemaVersion: 1, entries: [bad] }, "windows-x64", "agent"),
      /Invalid controlled/,
    );
  }
});
