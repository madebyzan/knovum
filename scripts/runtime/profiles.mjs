import { join } from "node:path";

export const runtimeProfiles = ["inference", "agent"];

export function runtimeProject(repository, profile) {
  if (!runtimeProfiles.includes(profile)) throw new Error(`Unsupported Python runtime profile: ${profile}`);
  return profile === "inference"
    ? join(repository, "python")
    : join(repository, "python", "runtimes", "agent");
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
    ...(profile === "inference"
      ? ["--extra", "inference", "--extra", "model-resources", "--no-emit-package", "voxcpm"]
      : []),
  ];
}

export function hostRuntimeTarget() {
  const target = `${{ darwin: "macos", win32: "windows" }[process.platform]}-${process.arch}`;
  if (!["macos-arm64", "windows-x64"].includes(target))
    throw new Error(`Unsupported runtime host: ${process.platform}/${process.arch}`);
  return target;
}

export function runtimeTargetFromTriple(triple) {
  if (!triple) return undefined;
  const target = { "aarch64-apple-darwin": "macos-arm64", "x86_64-pc-windows-msvc": "windows-x64" }[triple];
  if (!target) throw new Error(`Unsupported runtime target: ${triple}`);
  return target;
}

export function runtimeSources(target, profile, environment) {
  const platform = target.toUpperCase().replaceAll("-", "_");
  const python = `PYTHON_${profile.toUpperCase()}`;
  const names = {
    ...(profile === "inference" ? { FFMPEG: "FFMPEG" } : {}),
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
