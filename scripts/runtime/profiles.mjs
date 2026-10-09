import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

export const runtimeProfiles = [
  "resources",
  "light-inference",
  "torch-inference",
  "voice-generation",
  "agent",
];
export const runtimeTargets = {
  "aarch64-apple-darwin": "macos-arm64",
  "x86_64-apple-darwin": "macos-x64",
  "x86_64-pc-windows-msvc": "windows-x64",
};

export function runtimeProject(repository, profile) {
  if (!runtimeProfiles.includes(profile)) throw new Error(`Unsupported Python runtime profile: ${profile}`);
  return join(repository, "python", "runtimes", profile);
}

export function condaLockPath(repository, target, profile) {
  const source = JSON.parse(readFileSync(join(repository, "python/runtime-source.json"), "utf8"));
  const group = source.condaSupplier.profiles[profile];
  return join(repository, "python/conda-locks", group, `${target}.json`);
}

export function dependencyIdentity(repository, target, profile) {
  return createHash("sha256")
    .update(readFileSync(join(runtimeProject(repository, profile), "uv.lock")))
    .update(readFileSync(condaLockPath(repository, target, profile)))
    .digest("hex");
}

export function developmentDirectory(repository, target, profile) {
  return join(
    repository,
    ".runtime-development",
    profile,
    target,
    runtimeRecipeIdentity(repository, target, profile),
  );
}

export function runtimeRecipeIdentity(repository, target, profile) {
  return createHash("sha256")
    .update(runtimeRecipeBytes(repository, target, profile))
    .update(producerBytes(repository, profile))
    .update(readFileSync(join(runtimeProject(repository, profile), "uv.lock")))
    .update(readFileSync(condaLockPath(repository, target, profile)))
    .digest("hex");
}

export function runtimeRecipeBytes(repository, target, profile) {
  runtimeProject(repository, profile);
  const configuration = JSON.parse(readFileSync(join(repository, "python/runtime-source.json"), "utf8"));
  const condaGroup = configuration.condaSupplier.profiles[profile];
  const recipe = {
    profile,
    target,
    version: configuration.version,
    uvVersion: configuration.uvVersion,
    uv: configuration.uv.targets[target],
    condaSupplier: {
      name: configuration.condaSupplier.name,
      micromambaVersion: configuration.condaSupplier.micromambaVersion,
      group: condaGroup,
      pythonDistributions: configuration.condaSupplier.pythonDistributions[condaGroup],
    },
    micromamba: configuration.micromamba.targets[target],
    ...(profile === "light-inference"
      ? {
          mossNano: configuration.mossNano,
          compiledSourceEvidence: JSON.parse(
            readFileSync(join(repository, "python/compiled-source-evidence.json"), "utf8"),
          ),
        }
      : {}),
    ...(profile === "voice-generation"
      ? {
          voiceLibrary: Object.fromEntries(
            Object.entries(configuration.voiceLibrary).filter(
              ([key]) => key !== "vulkanSdk" || target === "windows-x64",
            ),
          ),
          ...(target.startsWith("macos-")
            ? {
                macosMinimumSystemVersion: JSON.parse(
                  readFileSync(join(repository, "src-tauri/tauri.conf.json"), "utf8"),
                ).bundle.macOS.minimumSystemVersion,
              }
            : {}),
        }
      : {}),
  };
  return recipeBytes(recipe);
}

export function recipeBytes(recipe) {
  return Buffer.from(
    JSON.stringify(recipe, (_key, value) =>
      value !== null && typeof value === "object" && !Array.isArray(value)
        ? Object.fromEntries(
            Object.entries(value).toSorted(([left], [right]) => left.localeCompare(right, "en")),
          )
        : value,
    ),
  );
}

export function producerBytes(repository, profile) {
  if (profile !== "media") runtimeProject(repository, profile);
  const { producerSources } = JSON.parse(
    readFileSync(join(repository, "python/runtime-source.json"), "utf8"),
  );
  const names = [
    ...producerSources.common,
    ...(profile === "media" ? producerSources.media : producerSources.python),
    ...(producerSources.profiles[profile] ?? []),
  ].toSorted();
  return Buffer.concat(
    names.flatMap((name) => [
      Buffer.from(`${name}\0`),
      readFileSync(join(repository, "scripts/runtime", name)),
      Buffer.from("\0"),
    ]),
  );
}

export function requirementsArguments(repository, profile) {
  return [
    "export",
    "--project",
    runtimeProject(repository, profile),
    "--frozen",
    "--no-emit-project",
    "--no-default-groups",
    "--no-header",
    "--no-annotate",
    ...JSON.parse(
      readFileSync(join(repository, "python/runtime-source.json"), "utf8"),
    ).condaSupplier.pythonDistributions[profile === "torch-inference" ? "tensor" : "base"].flatMap((name) => [
      "--no-emit-package",
      name,
    ]),
  ];
}

export function hostRuntimeTarget() {
  const target = `${{ darwin: "macos", win32: "windows" }[process.platform]}-${process.arch}`;
  if (!Object.values(runtimeTargets).includes(target))
    throw new Error(`Unsupported runtime host: ${process.platform}/${process.arch}`);
  return target;
}

export function runtimeTargetFromTriple(triple) {
  if (!triple) return undefined;
  const target = runtimeTargets[triple];
  if (!target) throw new Error(`Unsupported runtime target: ${triple}`);
  return target;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const { values } = parseArgs({
    options: {
      repository: { type: "string" },
      target: { type: "string" },
      profile: { type: "string" },
      identity: { type: "boolean" },
    },
  });
  if (!values.repository || !values.target || !values.profile)
    throw new Error("Runtime recipe requires --repository, --target, and --profile");
  process.stdout.write(
    values.identity
      ? runtimeRecipeIdentity(values.repository, values.target, values.profile)
      : runtimeRecipeBytes(values.repository, values.target, values.profile),
  );
}
