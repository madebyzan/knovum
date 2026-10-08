import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { sha256File } from "./files.mjs";
import { validateMediaDescriptor } from "./media-manifest.mjs";
import { runtimeProfiles, runtimeTargets } from "./profiles.mjs";

export function publicationSources(publication, target, profile) {
  if (publication.schemaVersion !== 1) throw new Error("Unsupported runtime publication schema");
  const kinds =
    profile === "core-media"
      ? ["FFMPEG", "FFMPEG_DESCRIPTOR"]
      : ["PYTHON_RUNTIME", "PYTHON_RUNTIME_DESCRIPTOR", "PYTHON_RUNTIME_EVIDENCE"];
  const sources = selectSources(publication, target, profile, kinds);
  return profile === "light-inference"
    ? [...sources, ...publicationSources(publication, target, "core-media")]
    : sources;
}

function selectSources(publication, target, profile, kinds) {
  const matches = publication.entries.filter((entry) => entry.target === target && entry.profile === profile);
  if (matches.length !== 1) throw new Error(`Publication must contain exactly one ${profile}/${target}`);
  const entry = matches[0];
  return kinds.map((kind) => {
    const source = entry.sources[kind];
    const url = new URL(source.url);
    if (
      url.protocol !== "https:" ||
      url.username ||
      url.password ||
      url.hash ||
      !/^[a-f0-9]{64}$/u.test(source.expectedHash)
    )
      throw new Error(`Invalid controlled publication source: ${kind}`);
    return { kind, url: source.url, expectedHash: source.expectedHash };
  });
}

export function writePublication(directory, baseUrl) {
  const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
  const coreMedia = Object.values(runtimeTargets).map((target) => {
    const prefix = `ffmpeg-${target}`;
    const sources = Object.fromEntries(
      Object.entries({ FFMPEG: `${prefix}.tar.gz`, FFMPEG_DESCRIPTOR: `${prefix}-descriptor.json` }).map(
        ([kind, name]) => [
          kind,
          { url: `${baseUrl}/${name}`, expectedHash: sha256File(join(directory, name)) },
        ],
      ),
    );
    const descriptor = JSON.parse(readFileSync(join(directory, `${prefix}-descriptor.json`), "utf8"));
    validateMediaDescriptor(repository, descriptor, target);
    if (descriptor.archiveUrl !== sources.FFMPEG.url || descriptor.sha256 !== sources.FFMPEG.expectedHash)
      throw new Error(`Publication descriptor differs from its produced archive: ${prefix}`);
    return { profile: "core-media", target, sources };
  });
  const entries = [
    ...coreMedia,
    ...Object.values(runtimeTargets).flatMap((target) =>
      runtimeProfiles.map((profile) => {
        const prefix = `python-${profile}-${target}`;
        const files = {
          PYTHON_RUNTIME: `${prefix}.tar.gz`,
          PYTHON_RUNTIME_DESCRIPTOR: `${prefix}-descriptor.json`,
          PYTHON_RUNTIME_EVIDENCE: `${prefix}-evidence.tar.gz`,
        };
        const descriptor = JSON.parse(readFileSync(join(directory, files.PYTHON_RUNTIME_DESCRIPTOR), "utf8"));
        const sources = Object.fromEntries(
          Object.entries(files).map(([kind, name]) => [
            kind,
            { url: `${baseUrl}/${name}`, expectedHash: sha256File(join(directory, name)) },
          ]),
        );
        if (
          descriptor.profile !== profile ||
          descriptor.target !== target ||
          descriptor.archiveUrl !== sources.PYTHON_RUNTIME.url ||
          descriptor.sha256 !== sources.PYTHON_RUNTIME.expectedHash
        )
          throw new Error(`Publication descriptor differs from its produced archive: ${prefix}`);
        return { profile, target, sources };
      }),
    ),
  ];
  const publication = { schemaVersion: 1, entries };
  for (const entry of entries) publicationSources(publication, entry.target, entry.profile);
  writeFileSync(join(directory, "runtime-publication.json"), `${JSON.stringify(publication, null, 2)}\n`);
}
