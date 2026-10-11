import { execFileSync } from "node:child_process";
import { openSync, readSync, closeSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

import { listRuntimeFiles } from "./files.mjs";
import { isMicrosoftRuntime, verifyMicrosoftRuntime } from "./windows-runtime-signature.mjs";

const mach = new Set([
  "cefaedfe",
  "cffaedfe",
  "feedface",
  "feedfacf",
  "cafebabe",
  "bebafeca",
  "cafebabf",
  "bfbafeca",
]);
export function compiledFiles(directory, target) {
  return listRuntimeFiles(directory)
    .filter((name) => {
      const fd = openSync(join(directory, name), "r");
      const header = Buffer.alloc(4);
      try {
        readSync(fd, header, 0, 4, 0);
      } finally {
        closeSync(fd);
      }
      return target.startsWith("macos-")
        ? mach.has(header.toString("hex"))
        : header.subarray(0, 2).toString() === "MZ";
    })
    .map((name) => join(directory, name));
}

function macRequirement() {
  const identity = process.env.APPLE_SIGNING_IDENTITY;
  const match = /^Developer ID Application: .+ \(([A-Z0-9]{10})\)$/u.exec(identity ?? "");
  if (!match) throw new Error("Production signing requires a Developer ID Application identity and Team ID");
  return `anchor apple generic and certificate leaf[subject.CN] = ${JSON.stringify(identity)} and certificate leaf[subject.OU] = ${JSON.stringify(match[1])}`;
}

function windowsIdentity() {
  const identity = process.env.KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT?.toUpperCase();
  if (!/^[A-F0-9]{40}$/u.test(identity ?? ""))
    throw new Error("Production signing requires an approved certificate thumbprint");
  return identity;
}

export function verifyCompiledSignature(
  path,
  target,
  production = true,
  { preserveMicrosoftRuntime = false } = {},
) {
  if (target.startsWith("macos-")) {
    execFileSync("codesign", ["--verify", "--strict", ...(production ? ["-R", macRequirement()] : []), path]);
    return;
  }
  if (preserveMicrosoftRuntime && isMicrosoftRuntime(path)) return verifyMicrosoftRuntime(path);
  execFileSync(process.env.KNOVUM_WINDOWS_SIGNTOOL, ["verify", "/pa", "/all", path]);
  const script =
    "$ErrorActionPreference='Stop'; $s=Get-AuthenticodeSignature -LiteralPath $env:KNOVUM_SIGNATURE_VERIFY_PATH; if($s.Status -ne 'Valid' -or $null -eq $s.SignerCertificate){throw 'Invalid system signature'}; $s.SignerCertificate.Thumbprint";
  const signer = execFileSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", script], {
    encoding: "utf8",
    env: { ...process.env, KNOVUM_SIGNATURE_VERIFY_PATH: path },
  })
    .trim()
    .toUpperCase();
  if (signer !== windowsIdentity())
    throw new Error("Compiled resource signer differs from the approved certificate");
}

export function signCompiledFiles(directory, target, mode, { preserveMicrosoftRuntime = false } = {}) {
  if (!["production", "adhoc", "unsigned"].includes(mode))
    throw new Error("Invalid compiled resource signing mode");
  const files = compiledFiles(directory, target);
  let identity;
  if (target.startsWith("macos-")) {
    if (mode === "unsigned") throw new Error("macOS requires ad-hoc or production signing");
    identity = mode === "adhoc" ? "-" : process.env.APPLE_SIGNING_IDENTITY;
    if (mode === "production") macRequirement();
    for (const path of files) {
      execFileSync("codesign", [
        "--force",
        "--sign",
        identity,
        ...(mode === "production" ? ["--options", "runtime", "--timestamp"] : []),
        path,
      ]);
      verifyCompiledSignature(path, target, mode === "production");
    }
  } else {
    identity = signWindows(files, target, mode, preserveMicrosoftRuntime);
  }
  return { mode, identity, compiledBinaryCount: files.length };
}

function signWindows(files, target, mode, preserveMicrosoftRuntime) {
  if (mode === "adhoc") throw new Error("Windows does not support ad-hoc signing");
  let identity = "unsigned";
  if (mode === "production") {
    identity = windowsIdentity();
    const timestamp = new URL(process.env.KNOVUM_WINDOWS_TIMESTAMP_URL);
    if (timestamp.protocol !== "https:" || timestamp.username || timestamp.password)
      throw new Error("Timestamp service requires HTTPS");
    for (const path of files) {
      if (preserveMicrosoftRuntime && isMicrosoftRuntime(path)) {
        verifyMicrosoftRuntime(path);
        continue;
      }
      execFileSync(process.env.KNOVUM_WINDOWS_SIGNTOOL, [
        "sign",
        "/sha1",
        identity,
        "/fd",
        "SHA256",
        "/tr",
        timestamp.href,
        "/td",
        "SHA256",
        path,
      ]);
      verifyCompiledSignature(path, target);
    }
  }

  return identity;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      directory: { type: "string" },
      target: { type: "string" },
      mode: { type: "string" },
      list: { type: "boolean" },
    },
  });
  if (!values.directory || !["macos-arm64", "macos-x64", "windows-x64"].includes(values.target))
    throw new Error("Compiled resource inspection requires a directory and supported target");
  const result = values.list
    ? compiledFiles(values.directory, values.target).map((file) =>
        relative(values.directory, file).replaceAll("\\", "/"),
      )
    : signCompiledFiles(values.directory, values.target, values.mode);
  if (!values.list && result.compiledBinaryCount === 0)
    throw new Error("Runtime contains no compiled executable files");
  process.stdout.write(JSON.stringify(result));
}
