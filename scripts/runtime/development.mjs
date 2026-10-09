import { randomUUID } from "node:crypto";
import { existsSync, mkdirSync, renameSync, rmSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { validatePythonRuntime } from "./manifest.mjs";
import { runProducer } from "./producer.mjs";
import { developmentDirectory, hostRuntimeTarget, runtimeProfiles, runtimeProject } from "./profiles.mjs";

export async function prepareDevelopment(repository, profiles, scratchParent) {
  const target = hostRuntimeTarget();
  for (const profile of profiles) runtimeProject(repository, profile);
  for (const profile of profiles) {
    const output = developmentDirectory(repository, target, profile);
    const validate = (directory) =>
      validatePythonRuntime(
        join(directory, "runtime"),
        target,
        join(runtimeProject(repository, profile), "uv.lock"),
      );
    // Profiles share producer tooling and Conda caches; sequential builds bound disk and memory.
    // oxlint-disable-next-line no-await-in-loop
    await prepareDevelopmentRuntime({
      output,
      validate,
      stagingParent: scratchParent,
      produce: async (staging) => {
        console.log(`Preparing ${profile} development runtime for ${target}`);
        await runProducer(repository, [
          "--profile",
          profile,
          "--output",
          staging,
          "--development",
          ...(scratchParent ? ["--scratch-parent", scratchParent] : []),
        ]);
      },
    });
  }
}

export async function prepareDevelopmentRuntime({
  output,
  validate,
  produce,
  stagingParent,
  stagingId = randomUUID,
}) {
  if (existsSync(output)) {
    validate(output);
    return;
  }

  const parent = dirname(output);
  mkdirSync(parent, { recursive: true });
  const staging = join(stagingParent ?? parent, `.${basename(output)}.${stagingId()}.installing`);
  try {
    await produce(staging);
    validate(staging);
    try {
      renameSync(staging, output);
    } catch (error) {
      if (!existsSync(output)) throw error;
      validate(output);
    }
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
  const { values } = parseArgs({
    options: { profile: { type: "string" }, "scratch-parent": { type: "string" } },
  });
  await prepareDevelopment(
    repository,
    values.profile ? [values.profile] : runtimeProfiles,
    values["scratch-parent"],
  );
}
