import { execFileSync } from "node:child_process";
import { chmodSync, copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { validateMediaLicenseReview } from "./media-license-policy.mjs";
import { copyTree, seal, verify } from "./resources.mjs";
import { signCompiledFiles, verifyCompiledFiles } from "./signing.mjs";

function uv(repository, args, env = process.env) {
  execFileSync("uv", ["run", "--project", "python", "--group", "packaging", "--locked", ...args], {
    cwd: repository,
    stdio: "inherit",
    env,
  });
}
export function prepareArtifacts(repository, options, cargo) {
  const resources = join(repository, "src-tauri/resources");
  const mode = options.target.startsWith("macos-") ? "adhoc" : "unsigned";
  const inference = join(resources, "inference-runtime");
  copyTree(cargo.inference, inference);
  const workerName =
    options.target === "windows-x64" ? "knovum-inference-worker.exe" : "knovum-inference-worker";
  mkdirSync(join(inference, "bin"), { recursive: true });
  copyFileSync(cargo.worker, join(inference, "bin", workerName));
  chmodSync(join(inference, "bin", workerName), 0o755);
  if (options.target === "windows-x64")
    execFileSync(
      "cmake",
      [
        `-DCMAKE_INSTALL_PREFIX=${inference}`,
        "-P",
        join(inference, "build-evidence/windows-qualification.cmake"),
      ],
      { stdio: "inherit" },
    );
  const librarySigning = signCompiledFiles(inference, options.target, mode, {
    preserveMicrosoftRuntime: true,
  });
  writeFileSync(join(inference, "build-evidence/signing.json"), JSON.stringify(librarySigning));
  const original = JSON.parse(readFileSync(join(cargo.inference, "inference-runtime-manifest.json"), "utf8"));
  const inferenceManifest = seal(inference, "inference-runtime-manifest.json", {
    target: options.target,
    onnxRuntime: original.onnxRuntime,
  });
  validateMediaLicenseReview(repository, options.target, { requireApproved: options.production });
  const media = join(resources, "bundled-runtime", options.target);
  copyTree(cargo.media, media);
  signCompiledFiles(media, options.target, mode);
  const mediaManifest = seal(media, "media-manifest.json", { target: options.target });
  uv(
    repository,
    [
      "pyinstaller",
      "--noconfirm",
      "--distpath",
      "src-tauri/resources/agent-runtime",
      "--workpath",
      "python/.build/pyinstaller",
      "python/packaging/agent.spec",
    ],
    { ...process.env, APPLE_SIGNING_IDENTITY: options.production ? process.env.APPLE_SIGNING_IDENTITY : "" },
  );
  const agent = join(resources, "agent-runtime/knovum-agent");
  if (options.target === "windows-x64")
    signCompiledFiles(agent, options.target, mode, { preserveMicrosoftRuntime: true });
  uv(repository, [
    "python",
    "python/packaging/distribution.py",
    "--root",
    agent,
    "--target",
    options.target,
    ...(options.production ? ["--require-approved"] : []),
  ]);
  uv(repository, [
    "python",
    "python/packaging/smoke.py",
    join(agent, options.target === "windows-x64" ? "knovum-agent.exe" : "knovum-agent"),
  ]);
  const agentManifest = readFileSync(join(agent, "agent-manifest.json"));
  return { inferenceManifest, mediaManifest, agentManifest };
}
export function validatePackagedResources(resources, repository, options) {
  verify(join(resources, "inference-runtime"), "inference-runtime-manifest.json", options.inferenceManifest);
  verify(join(resources, "bundled-runtime", options.target), "media-manifest.json", options.mediaManifest);
  const agent = join(resources, "agent-runtime/knovum-agent");
  // The trusted manifest is a build input outside the package being verified.
  uv(repository, [
    "python",
    "python/packaging/distribution.py",
    "--root",
    agent,
    "--verify-against",
    join(repository, "src-tauri/resources/agent-runtime/knovum-agent/agent-manifest.json"),
  ]);
  if (!readFileSync(join(agent, "agent-manifest.json")).equals(options.agentManifest))
    throw new Error("Packaged Agent manifest differs from the controlled build");
  if (options.production) {
    for (const directory of [
      join(resources, "inference-runtime"),
      join(resources, "bundled-runtime", options.target),
      agent,
    ]) {
      verifyCompiledFiles(directory, options.target, { preserveMicrosoftRuntime: true });
    }
  }
}
