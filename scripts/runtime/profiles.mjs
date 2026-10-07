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
    ...(profile === "voice-generation" ? ["--no-emit-package", "voxcpm"] : []),
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

export function runtimeSources(target, profile, environment) {
  const platform = target.toUpperCase().replaceAll("-", "_");
  const python = `PYTHON_${profile.toUpperCase().replaceAll("-", "_")}`;
  const names = {
    ...(profile === "light-inference" ? { FFMPEG: "FFMPEG" } : {}),
    PYTHON_RUNTIME: python,
    PYTHON_RUNTIME_DESCRIPTOR: `${python}_DESCRIPTOR`,
    PYTHON_RUNTIME_EVIDENCE: `${python}_EVIDENCE`,
  };
  return Object.entries(names).map(([kind, name]) => {
    const prefix = `KNOVUM_${name}_${platform}`;
    const url = environment[`${prefix}_URL`];
    const expectedHash = environment[`${prefix}_SHA256`];
    if (!url?.startsWith("https://") || !/^[a-f0-9]{64}$/u.test(expectedHash ?? ""))
      throw new Error(`Missing controlled ${kind} source for ${target}; configure its HTTPS URL and SHA256.`);
    return { kind, url, expectedHash };
  });
}
