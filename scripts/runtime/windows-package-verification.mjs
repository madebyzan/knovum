import { execFileSync } from "node:child_process";
import { lstatSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, extname, join, relative, resolve, sep } from "node:path";

import { validateInferencePackage } from "./inference/package.mjs";
import { validatePackagedResources } from "./packaged-resources.mjs";

const windowsTarget = "windows-x64";
const sevenZipVersion = "26.03";
const defaultSevenZip = String.raw`C:\Program Files\7-Zip\7z.exe`;
const maxExtractedEntries = 200_000;

export function verifyWindowsApplicationPackage(packagePath, options) {
  if (options.target !== windowsTarget) {
    throw new Error(`Windows runtime package verification does not support ${options.target}`);
  }
  const packageFile = resolve(packagePath);
  requireRegularFile(packageFile, "Windows package");
  if (options.production) verifyDistributionSignature(packageFile);
  const staging = mkdtempSync(join(tmpdir(), "knovum-windows-package-"));
  try {
    extractPackage(packageFile, staging, options.sevenZipPath ?? defaultSevenZip);
    return verifyExtractedWindowsApplication(staging, options);
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

function verifyExtractedWindowsApplication(extractionRoot, options) {
  const resources = locateApplicationResources(extractionRoot, options.target);
  validatePackagedResources(resources, resolve(options.repository), options.target);
  if (options.production) verifyDistributionSignature(join(dirname(resources), "knovum.exe"));
  validateInferencePackage(
    join(resources, "inference-runtime"),
    options.target,
    options.inferenceManifest,
    options.production,
  );
  return resources;
}

export function locateApplicationResources(extractionRoot, target = windowsTarget) {
  if (target !== windowsTarget) throw new Error(`Unsupported packaged runtime target: ${target}`);
  const manifests = findFiles(extractionRoot, (name) => name === "python-agent-descriptor.json");
  if (manifests.length !== 1)
    throw new Error(`Windows package must contain exactly one runtime descriptor; found ${manifests.length}`);
  const parts = relative(resolve(extractionRoot), manifests[0]).split(sep);
  if (!hasPathSuffix(parts, ["resources", "python-agent-descriptor.json"]))
    throw new Error("Windows package descriptor is outside the application resources directory");
  const forbidden = findFiles(
    extractionRoot,
    (name) => name === "runtime-manifest.json" || name.toLowerCase() === "python.exe",
  );
  if (forbidden.length) throw new Error("Windows package must exclude the third-party Python runtime");
  return dirname(manifests[0]);
}

function hasPathSuffix(parts, suffix) {
  return (
    parts.length >= suffix.length && suffix.every((part, index) => part === parts.at(index - suffix.length))
  );
}

function extractPackage(packageFile, staging, sevenZip) {
  const extension = extname(packageFile).toLowerCase();
  if (extension === ".msi") return extractMsi(packageFile, staging);
  if (extension !== ".exe") throw new Error(`Unsupported Windows package type: ${extension || "none"}`);
  verifySevenZip(sevenZip);
  extractArchive(sevenZip, packageFile, staging);
}

function extractMsi(packageFile, staging) {
  const systemRoot = process.env.SystemRoot;
  if (!systemRoot) throw new Error("SystemRoot is required to locate Windows Installer");
  const installer = join(systemRoot, "System32", "msiexec.exe");
  requireRegularFile(installer, "Windows Installer executable");
  execFileSync(installer, ["/a", packageFile, "/qn", "/norestart", `TARGETDIR=${resolve(staging)}`], {
    stdio: "ignore",
    windowsHide: true,
  });
}

function verifySevenZip(executable) {
  requireRegularFile(executable, "7-Zip executable");
  const output = execFileSync(executable, ["i"], {
    encoding: "utf8",
    maxBuffer: 64 * 1024,
    windowsHide: true,
  });
  if (!new RegExp(`\\b7-Zip ${sevenZipVersion.replace(".", "\\.")}\\b`, "u").test(output)) {
    throw new Error(`Windows package verification requires 7-Zip ${sevenZipVersion}`);
  }
}

function extractArchive(executable, archive, destination) {
  execFileSync(executable, ["x", "-y", `-o${destination}`, archive], {
    stdio: "ignore",
    windowsHide: true,
  });
}

function findFiles(root, matches) {
  const matchingFiles = [];
  let entries = 0;
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      entries += 1;
      if (entries > maxExtractedEntries)
        throw new Error("Windows package extraction exceeds the entry limit");
      const path = join(directory, entry.name);
      if (entry.isSymbolicLink())
        throw new Error(`Windows package extraction contains a link: ${entry.name}`);
      if (entry.isDirectory()) visit(path);
      else if (entry.isFile() && matches(basename(path))) matchingFiles.push(path);
    }
  };
  visit(resolve(root));
  return matchingFiles;
}

function requireRegularFile(path, label) {
  if (!lstatSync(path).isFile()) throw new Error(`${label} is not a regular file`);
}

function verifyDistributionSignature(path) {
  requireRegularFile(path, "Signed distribution file");
  const script =
    "$ErrorActionPreference='Stop'; $s=Get-AuthenticodeSignature -LiteralPath $env:KNOVUM_SIGNATURE_VERIFY_PATH; if($s.Status -ne 'Valid' -or $null -eq $s.TimeStamperCertificate){throw 'A valid timestamped distribution signature is required'}";
  execFileSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", script], {
    env: { ...process.env, KNOVUM_SIGNATURE_VERIFY_PATH: path },
    windowsHide: true,
  });
}
