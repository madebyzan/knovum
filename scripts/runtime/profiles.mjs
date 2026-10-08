import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";

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

export function nativeLockPath(repository, target, profile) {
  const source = JSON.parse(readFileSync(join(repository, "python/runtime-source.json"), "utf8"));
  const group = source.nativeSupplier.profiles[profile];
  return join(repository, "python/native-locks", group, `${target}.json`);
}

export function dependencyIdentity(repository, target, profile) {
  return createHash("sha256")
    .update(readFileSync(join(runtimeProject(repository, profile), "uv.lock")))
    .update(readFileSync(nativeLockPath(repository, target, profile)))
    .digest("hex");
}

export function developmentDirectory(repository, target, profile) {
  const identity = createHash("sha256")
    .update(readFileSync(join(repository, "python/runtime-source.json")))
    .update(producerBytes(repository, profile))
    .update(readFileSync(join(runtimeProject(repository, profile), "uv.lock")))
    .update(readFileSync(nativeLockPath(repository, target, profile)))
    .digest("hex");
  return join(repository, ".runtime-development", profile, target, identity);
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
    ).nativeSupplier.pythonDistributions[profile === "torch-inference" ? "tensor" : "base"].flatMap(
      (name) => ["--no-emit-package", name],
    ),
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
