import { createHash } from "node:crypto";
import { join } from "node:path";

import { distributionDirectory, readRuntimeDescriptor } from "./distribution.mjs";
import { listRuntimeFiles, sha256File } from "./files.mjs";
import { validateMediaRuntime } from "./media-manifest.mjs";
import { runtimeProfiles } from "./profiles.mjs";

export function validatePackagedResources(resources, repository, target) {
  for (const profile of runtimeProfiles) {
    const path = join(resources, `python-${profile}-descriptor.json`);
    readRuntimeDescriptor(path, target, profile);
    if (
      sha256File(path) !==
      sha256File(join(distributionDirectory(repository, target, profile), "runtime-descriptor.json"))
    )
      throw new Error(`Packaged ${profile} descriptor differs from the trusted build input`);
  }
  validateMediaRuntime(repository, join(resources, "bundled-runtime", target), target);
  const source = join(repository, "python", "src");
  const payload = join(resources, "python-worker");
  const expected = listRuntimeFiles(source).filter((path) => path.endsWith(".py"));
  const actual = listRuntimeFiles(payload);
  if (expected.some((path) => !path.startsWith("knovum_agent_runtime/")))
    throw new Error("Python Worker ownership is restricted to Agent");
  if (
    actual.length !== expected.length ||
    expected.some((path) => sha256File(join(payload, path)) !== sha256File(join(source, path)))
  )
    throw new Error("Packaged Python Worker differs from the application checkout");
  const files = listRuntimeFiles(resources);
  if (
    files.some(
      (path) =>
        path.includes("python-runtime/") ||
        path.endsWith("/runtime-manifest.json") ||
        path === "runtime-manifest.json" ||
        /(?:^|\/)python(?:3(?:\.11)?|\.exe)?$/u.test(path),
    )
  )
    throw new Error("Desktop package must exclude the third-party Python runtime");
  const worker = join(payload, "knovum_agent_runtime");
  return { worker, workerDigest: workerDigest(payload) };
}

export function workerDigest(directory) {
  const hash = createHash("sha256");
  for (const path of listRuntimeFiles(directory).filter((name) => name.endsWith(".py"))) {
    hash.update(`${path}\0${sha256File(join(directory, path))}\n`);
  }
  return hash.digest("hex");
}
