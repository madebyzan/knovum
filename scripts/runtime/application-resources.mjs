import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { distributionDirectory, readRuntimeDescriptor } from "./distribution.mjs";
import { listRuntimeFiles, sha256File } from "./files.mjs";

export function validateApplicationResources(resources, repository, target) {
  const descriptorPath = join(resources, "python-runtime-descriptor.json");
  readRuntimeDescriptor(descriptorPath, target);
  if (
    sha256File(descriptorPath) !==
    sha256File(join(distributionDirectory(repository, target), "runtime-descriptor.json"))
  )
    throw new Error("Packaged runtime descriptor differs from the trusted build input");
  const source = join(repository, "python", "src");
  const payload = join(resources, "python-worker");
  const expected = listRuntimeFiles(source).filter((path) => path.endsWith(".py"));
  const actual = listRuntimeFiles(payload);
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
        path.endsWith("runtime-manifest.json") ||
        /(?:^|\/)python(?:3(?:\.11)?|\.exe)?$/u.test(path),
    )
  )
    throw new Error("Desktop package must exclude the third-party Python runtime");
  const descriptor = JSON.parse(readFileSync(descriptorPath, "utf8"));
  const worker = join(payload, "knovum_inference");
  return { worker, descriptor, workerDigest: workerDigest(worker) };
}

export function workerDigest(directory) {
  const hash = createHash("sha256");
  for (const path of listRuntimeFiles(directory).filter((name) => name.endsWith(".py"))) {
    hash.update(`${path}\0${sha256File(join(directory, path))}\n`);
  }
  return hash.digest("hex");
}
