import { createHash } from "node:crypto";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { listRuntimeFiles } from "../files.mjs";
import { validatePythonRuntime } from "../manifest.mjs";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const commit = "a".repeat(40);

function write(base, path, bytes) {
  const file = join(base, path);
  mkdirSync(join(file, ".."), { recursive: true });
  writeFileSync(file, bytes);
}

export function fixture(context, profile = "voice-generation") {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-validation-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const shared = join(root, "python");
  const python = join(shared, "runtimes", profile);
  const runtime = join(root, "runtime");
  const lock = "frozen runtime dependencies\n";
  write(python, "uv.lock", lock);
  const target = "macos-arm64";
  const source = { url: "https://example.test/cpython.tar.gz", sha256: "b".repeat(64), size: 42 };
  write(
    shared,
    "runtime-source.json",
    JSON.stringify({
      uvVersion: "0.12.23",
      voiceNative: {
        revision: commit,
        license: "Apache-2.0",
        cmakeVersion: "4.3.4",
        source: { sha256: hash("source") },
      },
      nativeSupplier: { name: "conda-forge", micromambaVersion: "2.9.0", profiles: { [profile]: "base" } },
    }),
  );
  const nativeLock = JSON.stringify({ packages: [{ name: "python", ...source }] });
  write(shared, `native-locks/base/${target}.json`, nativeLock);
  const native = { sources: [{ fileName: "native.zip", name: "native", sha256: hash("native"), size: 6 }] };
  write(shared, "compiled-source-evidence.json", JSON.stringify(native));
  write(python, "src/knovum_inference/__main__.py", "pass\n");
  write(runtime, "bin/python3", "interpreter");
  chmodSync(join(runtime, "bin/python3"), 0o755);
  write(runtime, "packages/dependency.py", "pass\n");
  write(runtime, "runtime-requirements.txt", "requirements");
  const evidence = join(root, "build-evidence");
  const producer = {
    schemaVersion: 1,
    profile,
    target,
    version: "3.11.17",
    uvVersion: "0.12.23",
    dependencyLockSha256: hash(lock + nativeLock),
    appCommit: commit,
    nativeSupplier: {
      supplier: "conda-forge",
      micromambaVersion: "2.9.0",
      lockSha256: hash(nativeLock),
      packages: [{ name: "python", ...source, files: [] }],
    },
    relocationVerified: true,
    signing: { mode: "adhoc", identity: "-", compiledBinaryCount: 1 },
    voxcpm: {
      name: "audio.cpp",
      version: commit,
      license: "Apache-2.0",
      cmakeVersion: "4.3.4",
      portableCpu: true,
      sourceSha256: hash("source"),
      bridgeSha256: hash("bridge"),
      recipeSha256: hash("recipe"),
    },
  };
  write(evidence, "producer.json", JSON.stringify(producer));
  for (const [name, bytes] of [
    ["dependency-evidence.json", "{}"],
    ["native-packages.json", JSON.stringify(producer.nativeSupplier)],
    ["sbom.cdx.json", "{}"],
    ["requirements.txt", "requirements"],
    ["audio-cpp-source.tar.gz", "source"],
    ["voice_bridge.cpp", "bridge"],
    ["voice_bridge.cmake", "recipe"],
    ["compiled-source-evidence.json", JSON.stringify(native)],
    ["compiled-sources/native.zip", "native"],
  ])
    write(evidence, name, bytes);
  write(root, "scripts/runtime/voice_bridge.cpp", "bridge");
  write(root, "scripts/runtime/voice_bridge.cmake", "recipe");
  function manifest() {
    const files = listRuntimeFiles(runtime, { excludeManifest: true }).map((path) => {
      const bytes = readFileSync(join(runtime, path));
      return { path, size: bytes.length, sha256: hash(bytes) };
    });
    write(
      runtime,
      "runtime-manifest.json",
      JSON.stringify({
        schemaVersion: 1,
        profile,
        dependencyLockSha256: hash(lock + nativeLock),
        runtimeId: "cpython",
        version: "3.11.17",
        dependencyRequirementsSha256: hash("requirements"),
        target,
        executable: "bin/python3",
        packageRoot: "packages",
        files,
      }),
    );
  }
  manifest();
  return {
    python,
    evidence,
    runtime,
    producer,
    manifest,
    write,
    validate: () => validatePythonRuntime(runtime, target, join(python, "uv.lock")),
  };
}
