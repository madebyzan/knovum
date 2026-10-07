import { validateRuntimeDescriptor } from "../runtime/distribution.mjs";
import { runtimeProfiles } from "../runtime/profiles.mjs";

export function validateRuntimePublications(value, target) {
  if (!value || !/^[a-f0-9]{64}$/u.test(value.workerDigest ?? ""))
    throw new Error("Candidate requires the packaged Worker identity");
  const runtimes = value.pythonRuntimes;
  if (
    !Array.isArray(runtimes) ||
    runtimes.length !== runtimeProfiles.length ||
    new Set(runtimes.map((runtime) => runtime.profile)).size !== runtimeProfiles.length
  )
    throw new Error("Candidate requires each independent Python runtime publication");
  for (const profile of runtimeProfiles) {
    const runtime = runtimes.find((entry) => entry.profile === profile);
    if (!runtime) throw new Error(`Candidate runtime publication is missing: ${profile}`);
    validateRuntimeDescriptor(runtime, target, profile);
  }
}
