import assert from "node:assert/strict";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { locatePackagedPythonRuntime, verifyWindowsRuntimePackage } from "../runtime-package.mjs";

function extraction(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-package-test-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  return root;
}

function manifest(root, prefix = "Knovum") {
  const runtime = join(root, prefix, "resources", "bundled-runtime", "windows-x64", "python-runtime");
  mkdirSync(runtime, { recursive: true });
  writeFileSync(join(runtime, "runtime-manifest.json"), "{}");
  writeFileSync(join(runtime, "python.exe"), "python");
  return runtime;
}

test("locates the single Windows runtime in the extracted application resources", (context) => {
  const root = extraction(context);
  const runtime = manifest(root);
  assert.equal(locatePackagedPythonRuntime(root), runtime);
});

test("rejects an installer without a Python runtime payload", (context) => {
  const root = extraction(context);
  assert.throws(() => locatePackagedPythonRuntime(root), /exactly one Python runtime; found 0/u);
});

test("rejects duplicate Python runtime payloads", (context) => {
  const root = extraction(context);
  manifest(root, "Knovum");
  manifest(root, "duplicate");
  assert.throws(() => locatePackagedPythonRuntime(root), /exactly one Python runtime; found 2/u);
});

test("rejects a second runtime executable even when its manifest is missing", (context) => {
  const root = extraction(context);
  manifest(root, "Knovum");
  const duplicate = join(root, "duplicate", "resources", "bundled-runtime", "windows-x64", "python-runtime");
  mkdirSync(duplicate, { recursive: true });
  writeFileSync(join(duplicate, "python.exe"), "python");
  assert.throws(() => locatePackagedPythonRuntime(root), /exactly one python.exe; found 2/u);
});

test("rejects a manifest outside the packaged resource path", (context) => {
  const root = extraction(context);
  writeFileSync(join(root, "runtime-manifest.json"), "{}");
  assert.throws(() => locatePackagedPythonRuntime(root), /outside the application resources/u);
});

test("rejects links in the extracted installer tree", (context) => {
  const root = extraction(context);
  const outside = extraction(context);
  manifest(outside);
  symlinkSync(outside, join(root, "linked"), "dir");
  assert.throws(() => locatePackagedPythonRuntime(root), /contains a link/u);
});

test("rejects targets outside the supported Windows release architecture", (context) => {
  const root = extraction(context);
  assert.throws(
    () => locatePackagedPythonRuntime(root, "windows-arm64"),
    /Unsupported packaged runtime target/u,
  );
});

test("cleans extraction staging when NSIS payload validation fails", (context) => {
  const root = extraction(context);
  const packageFile = join(root, "Knovum-setup.exe");
  const sevenZip = join(root, "7z");
  writeFileSync(packageFile, "installer");
  writeFileSync(sevenZip, '#!/bin/sh\n[ "$1" = i ] && echo "7-Zip 26.03 (x64)"\nexit 0\n');
  chmodSync(sevenZip, 0o755);
  const before = temporaryPackageDirectories();
  assert.throws(
    () =>
      verifyWindowsRuntimePackage(packageFile, {
        target: "windows-x64",
        repository: root,
        production: false,
        sevenZipPath: sevenZip,
      }),
    /exactly one Python runtime; found 0/u,
  );
  assert.deepEqual(temporaryPackageDirectories(), before);
});

test("rejects an unreviewed 7-Zip version for NSIS extraction", (context) => {
  const root = extraction(context);
  const packageFile = join(root, "Knovum-setup.exe");
  const sevenZip = join(root, "7z");
  writeFileSync(packageFile, "installer");
  writeFileSync(sevenZip, '#!/bin/sh\necho "7-Zip 25.01 (x64)"\n');
  chmodSync(sevenZip, 0o755);
  assert.throws(
    () =>
      verifyWindowsRuntimePackage(packageFile, {
        target: "windows-x64",
        repository: root,
        production: false,
        sevenZipPath: sevenZip,
      }),
    /requires 7-Zip 26\.03/u,
  );
});

test("extracts MSI through a Windows Installer administrative image", (context) => {
  const root = extraction(context);
  const system = join(root, "Windows");
  const installer = join(system, "System32", "msiexec.exe");
  const packageFile = join(root, "Knovum.msi");
  const log = join(root, "msiexec-args.txt");
  mkdirSync(join(system, "System32"), { recursive: true });
  writeFileSync(packageFile, "installer");
  writeFileSync(
    installer,
    `#!/bin/sh
printf '%s\n' "$@" > "$KNOVUM_MSI_TEST_LOG"
for value in "$@"; do case "$value" in TARGETDIR=*) target="\${value#TARGETDIR=}";; esac; done
runtime="$target/Knovum/resources/bundled-runtime/windows-x64/python-runtime"
mkdir -p "$runtime"
printf '{}' > "$runtime/runtime-manifest.json"
printf 'python' > "$runtime/python.exe"
`,
  );
  chmodSync(installer, 0o755);
  const previousRoot = process.env.SystemRoot;
  const previousLog = process.env.KNOVUM_MSI_TEST_LOG;
  process.env.SystemRoot = system;
  process.env.KNOVUM_MSI_TEST_LOG = log;
  try {
    assert.throws(
      () =>
        verifyWindowsRuntimePackage(packageFile, {
          target: "windows-x64",
          repository: root,
          production: false,
        }),
      /manifest is invalid/u,
    );
  } finally {
    restoreEnvironment("SystemRoot", previousRoot);
    restoreEnvironment("KNOVUM_MSI_TEST_LOG", previousLog);
  }
  const args = readFileSync(log, "utf8").trim().split("\n");
  assert.deepEqual(args.slice(0, 4), ["/a", packageFile, "/qn", "/norestart"]);
  assert.match(args[4], /^TARGETDIR=/u);
});

function temporaryPackageDirectories() {
  return readdirSync(tmpdir())
    .filter((name) => name.startsWith("knovum-windows-package-"))
    .toSorted();
}

function restoreEnvironment(name, value) {
  if (value === undefined) delete process.env[name];
  else process.env[name] = value;
}
