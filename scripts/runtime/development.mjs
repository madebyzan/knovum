import { existsSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
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
    const validate = () =>
      validatePythonRuntime(
        join(output, "runtime"),
        target,
        join(runtimeProject(repository, profile), "uv.lock"),
      );
    if (existsSync(output)) {
      validate();
      continue;
    }
    console.log(`Preparing ${profile} development runtime for ${target}`);
    // Profiles share producer tooling and Conda caches; sequential builds bound disk and memory.
    // oxlint-disable-next-line no-await-in-loop
    await runProducer(repository, [
      "--profile",
      profile,
      "--output",
      output,
      "--development",
      ...(scratchParent ? ["--scratch-parent", scratchParent] : []),
    ]);
    try {
      validate();
    } catch (error) {
      rmSync(output, { recursive: true, force: true });
      throw error;
    }
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
