import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { validateDependencyLicenseReview } from "./license-policy.mjs";

export function verifyRuntimeSignatures(directory, runtimeTarget, repository, profile) {
  const producer = JSON.parse(readFileSync(join(directory, "..", "build-evidence", "producer.json"), "utf8"));
  validateDependencyLicenseReview(directory, runtimeTarget, repository, profile);
  verifyCompiledResourceSignatures(directory, runtimeTarget, repository, producer);
}

export function verifyCompiledResourceSignatures(directory, runtimeTarget, repository, producer) {
  if (producer.signing?.mode !== "production")
    throw new Error("Production packages require signed compiled resources");
  if (
    runtimeTarget.startsWith("macos-") &&
    producer.signing.identity !== process.env.APPLE_SIGNING_IDENTITY
  ) {
    throw new Error("Python runtime signing identity differs from the application identity");
  }
  if (
    runtimeTarget === "windows-x64" &&
    producer.signing.identity?.toUpperCase() !==
      process.env.KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT?.toUpperCase()
  ) {
    throw new Error("Python runtime signing certificate differs from the approved certificate");
  }
  execFileSync(
    "uv",
    [
      "run",
      "--project",
      join(repository, "python"),
      "--only-group",
      "publication",
      "--frozen",
      "python",
      "-I",
      "-B",
      "-X",
      "utf8",
      join(repository, "scripts", "runtime", "verify_signatures.py"),
      "--directory",
      directory,
      "--target",
      runtimeTarget,
    ],
    { stdio: "inherit" },
  );
}
