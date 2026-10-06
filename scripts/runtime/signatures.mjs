import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { validateDependencyLicenseReview } from "./license-policy.mjs";

export function verifyRuntimeSignatures(directory, runtimeTarget, repository, profile) {
  const record = JSON.parse(readFileSync(join(directory, "..", "build-evidence", "producer.json"), "utf8"));
  if (record.signing?.mode !== "production")
    throw new Error("Production packages require a production-signed Python runtime");
  if (runtimeTarget === "macos-arm64" && record.signing.identity !== process.env.APPLE_SIGNING_IDENTITY) {
    throw new Error("Python runtime signing identity differs from the application identity");
  }
  if (
    runtimeTarget === "windows-x64" &&
    record.signing.identity?.toUpperCase() !==
      process.env.KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT?.toUpperCase()
  ) {
    throw new Error("Python runtime signing certificate differs from the approved certificate");
  }
  validateDependencyLicenseReview(directory, runtimeTarget, repository, profile);
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
