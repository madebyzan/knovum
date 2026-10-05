import { execFileSync } from "node:child_process";
import { lstatSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, extname, join, relative, resolve, sep } from "node:path";

import { validatePythonRuntime } from "./manifest.mjs";
import { verifyRuntimeSignatures } from "./signatures.mjs";

const windowsTarget = "windows-x64";
const sevenZipVersion = "26.03";
const defaultSevenZip = String.raw`C:\Program Files\7-Zip\7z.exe`;
const maxExtractedEntries = 200_000;

export function verifyWindowsRuntimePackage(packagePath, options) {
  if (options.target !== windowsTarget) {
    throw new Error(`Windows runtime package verification does not support ${options.target}`);
  }
  const packageFile = resolve(packagePath);
  requireRegularFile(packageFile, "Windows package");
  const staging = mkdtempSync(join(tmpdir(), "knovum-windows-package-"));
  try {
    extractPackage(
      packageFile,
      staging,
      options.sevenZipPath ?? process.env.KNOVUM_WINDOWS_7ZIP_PATH ?? defaultSevenZip,
    );
    return verifyExtractedWindowsRuntime(staging, options);
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

function verifyExtractedWindowsRuntime(extractionRoot, options) {
  const runtime = locatePackagedPythonRuntime(extractionRoot, options.target);
  const repository = resolve(options.repository);
  validatePythonRuntime(runtime, options.target, join(repository, "python", "uv.lock"));
  if (options.production) verifyRuntimeSignatures(runtime, options.target, repository);
  return runtime;
}

export function locatePackagedPythonRuntime(extractionRoot, target = windowsTarget) {
  if (target !== windowsTarget) throw new Error(`Unsupported packaged runtime target: ${target}`);
  const manifests = findFiles(extractionRoot, (name) => name === "runtime-manifest.json");
  if (manifests.length !== 1) {
    throw new Error(`Windows package must contain exactly one Python runtime; found ${manifests.length}`);
  }
  const expected = ["resources", "bundled-runtime", target, "python-runtime", "runtime-manifest.json"];
  const parts = relative(resolve(extractionRoot), manifests[0]).split(sep);
  if (!hasPathSuffix(parts, expected)) {
    throw new Error("Windows package Python runtime is outside the application resources directory");
  }
  const runtime = dirname(manifests[0]);
  const executableSuffix = [...expected.slice(0, -1), "python.exe"];
  const executables = findFiles(extractionRoot, (name) => name.toLowerCase() === "python.exe").filter(
    (path) => hasPathSuffix(relative(resolve(extractionRoot), path).split(sep), executableSuffix),
  );
  if (executables.length !== 1) {
    throw new Error(
      `Windows package runtime must contain exactly one python.exe; found ${executables.length}`,
    );
  }
  if (dirname(executables[0]) !== runtime)
    throw new Error("Windows package runtime manifest and executable differ");
  return runtime;
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
