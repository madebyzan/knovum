import { createHash } from "node:crypto";
import { chmodSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import { listRuntimeFiles } from "../../files.mjs";
import { validatePythonRuntime } from "../../manifest.mjs";
import { runtimeRecipeIdentity } from "../../profiles.mjs";

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
export const FIXTURE_COMMIT = "a".repeat(40);

function write(base, path, bytes) {
  const file = join(base, path);
  mkdirSync(join(file, ".."), { recursive: true });
  writeFileSync(file, bytes);
}

export function fixture(context, profile = "agent") {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-validation-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  cpSync(fileURLToPath(new URL("../../", import.meta.url)), join(root, "scripts/runtime"), {
    recursive: true,
  });
  write(root, "scripts/runtime/inference/voice_bridge.cpp", "bridge");
  write(root, "scripts/runtime/inference/voice_bridge.cmake", "recipe");
  write(
    root,
    "src-tauri/tauri.conf.json",
    JSON.stringify({ bundle: { macOS: { minimumSystemVersion: "14.0" } } }),
  );
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
      version: "3.11.17",
      uv: { targets: { [target]: source } },
      micromamba: { targets: { [target]: source } },
      producerSources: JSON.parse(
        readFileSync(new URL("../../../../python/runtime-source.json", import.meta.url), "utf8"),
      ).producerSources,
      uvVersion: "0.12.23",
      voiceLibrary: {
        revision: FIXTURE_COMMIT,
        license: "Apache-2.0",
        cmakeVersion: "4.3.4",
        source: { sha256: sha256("source") },
      },
      condaSupplier: {
        name: "conda-forge",
        micromambaVersion: "2.9.0",
        profiles: { [profile]: "base" },
        pythonDistributions: { base: ["python"] },
      },
    }),
  );
  const condaLock = JSON.stringify({ packages: [{ name: "python", ...source }] });
  write(shared, `conda-locks/base/${target}.json`, condaLock);
  const compiledSources = {
    sources: [{ fileName: "compiled.zip", name: "compiled", sha256: sha256("source"), size: 6 }],
  };
  write(shared, "compiled-source-evidence.json", JSON.stringify(compiledSources));
  write(shared, "src/knovum_agent_runtime/__main__.py", "pass\n");
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
    dependencyLockSha256: sha256(lock + condaLock),
    recipeSha256: runtimeRecipeIdentity(root, target, profile),
    appCommit: FIXTURE_COMMIT,
    condaSupplier: {
      supplier: "conda-forge",
      micromambaVersion: "2.9.0",
      lockSha256: sha256(condaLock),
      packages: [{ name: "python", ...source, files: [] }],
    },
    relocationVerified: true,
    signing: { mode: "adhoc", identity: "-", compiledBinaryCount: 1 },
    voxcpm: {
      name: "audio.cpp",
      version: FIXTURE_COMMIT,
      license: "Apache-2.0",
      cmakeVersion: "4.3.4",
      portableCpu: true,
      sourceSha256: sha256("source"),
      bridgeSha256: sha256("bridge"),
      recipeSha256: sha256("recipe"),
    },
  };
  write(evidence, "producer.json", JSON.stringify(producer));
  const dependencyEvidence = {
    cpython: { version: "3.11.17", source },
    runtimeInstallation: {
      supplier: "conda-forge",
      target,
      compiledBinaries: ["bin/python3"],
      notices: [],
    },
  };
  for (const [name, bytes] of [
    ["dependency-evidence.json", JSON.stringify(dependencyEvidence)],
    ["conda-packages.json", JSON.stringify(producer.condaSupplier)],
    ["sbom.cdx.json", "{}"],
    ["requirements.txt", "requirements"],
    ["audio-cpp-source.tar.gz", "source"],
    ["voice_bridge.cpp", "bridge"],
    ["voice_bridge.cmake", "recipe"],
    ["compiled-source-evidence.json", JSON.stringify(compiledSources)],
    ["compiled-sources/compiled.zip", "source"],
  ])
    write(evidence, name, bytes);
  function manifest() {
    const files = listRuntimeFiles(runtime, { excludeManifest: true }).map((path) => {
      const bytes = readFileSync(join(runtime, path));
      return { path, size: bytes.length, sha256: sha256(bytes) };
    });
    write(
      runtime,
      "runtime-manifest.json",
      JSON.stringify({
        schemaVersion: 1,
        profile,
        dependencyLockSha256: sha256(lock + condaLock),
        runtimeId: "cpython",
        version: "3.11.17",
        dependencyRequirementsSha256: sha256("requirements"),
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
