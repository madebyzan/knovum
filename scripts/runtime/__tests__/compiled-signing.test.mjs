import assert from "node:assert/strict";
import childProcess from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { mock, test } from "node:test";

import { signCompiledFiles } from "../compiled-signing.mjs";
import { verifyWindowsApplicationPackage } from "../windows-package-verification.mjs";

test("Windows production signing verifies the approved signer and preserves vendor CRT only explicitly", () => {
  const directory = mkdtempSync(join(tmpdir(), "knovum-signing-test-"));
  const names = [
    "KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT",
    "KNOVUM_WINDOWS_TIMESTAMP_URL",
    "KNOVUM_WINDOWS_SIGNTOOL",
  ];
  const previous = names.map((name) => process.env[name]);
  const identity = "A".repeat(40);
  const calls = [];
  const replacement = mock.method(childProcess, "execFileSync", (command, args) => {
    calls.push([command, args]);
    return command === "powershell.exe" ? identity : "";
  });
  syncBuiltinESMExports();
  try {
    process.env.KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT = identity;
    process.env.KNOVUM_WINDOWS_TIMESTAMP_URL = "https://example.invalid/timestamp";
    process.env.KNOVUM_WINDOWS_SIGNTOOL = "qualified-signtool";
    for (const name of ["worker.exe", "vcruntime140.dll"]) writeFileSync(join(directory, name), "MZxx");
    const ordinary = signCompiledFiles(directory, "windows-x64", "production");
    assert.equal(ordinary.identity, identity);
    assert.equal(
      calls.filter(([command, args]) => command === "qualified-signtool" && args[0] === "sign").length,
      2,
    );
    calls.length = 0;
    signCompiledFiles(directory, "windows-x64", "production", { preserveMicrosoftRuntime: true });
    const signed = calls.filter(([command, args]) => command === "qualified-signtool" && args[0] === "sign");
    assert.equal(signed.length, 1);
    assert.equal(signed[0][1].at(-1), join(directory, "worker.exe"));
  } finally {
    replacement.mock.restore();
    syncBuiltinESMExports();
    names.forEach((name, index) => {
      if (previous[index] === undefined) delete process.env[name];
      else process.env[name] = previous[index];
    });
    rmSync(directory, { recursive: true, force: true });
  }
});

test("invalid production installer signatures block NSIS and MSI extraction", () => {
  const root = mkdtempSync(join(tmpdir(), "knovum-distribution-signature-"));
  const calls = [];
  const replacement = mock.method(childProcess, "execFileSync", (command, _args, options) => {
    calls.push({ command, path: options.env.KNOVUM_SIGNATURE_VERIFY_PATH });
    throw new Error("Invalid timestamped distribution signature");
  });
  syncBuiltinESMExports();
  try {
    for (const extension of ["exe", "msi"]) {
      const path = join(root, `Knovum.${extension}`);
      writeFileSync(path, "installer");
      assert.throws(
        () =>
          verifyWindowsApplicationPackage(path, {
            target: "windows-x64",
            repository: root,
            production: true,
          }),
        /Invalid timestamped distribution signature/u,
      );
      assert.deepEqual(calls.at(-1), { command: "powershell.exe", path });
    }
    assert.equal(calls.length, 2);
  } finally {
    replacement.mock.restore();
    syncBuiltinESMExports();
    rmSync(root, { recursive: true, force: true });
  }
});

test("Agent extensions use the same compiled inventory and reject the wrong Windows signer", (context) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-agent-signing-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const previous = { ...process.env };
  const commands = [];
  const replacement = mock.method(childProcess, "execFileSync", (command, args) => {
    commands.push([command, args]);
    return command === "powershell.exe" ? "B".repeat(40) : "";
  });
  syncBuiltinESMExports();
  try {
    process.env.KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT = "A".repeat(40);
    process.env.KNOVUM_WINDOWS_TIMESTAMP_URL = "https://example.invalid/timestamp";
    process.env.KNOVUM_WINDOWS_SIGNTOOL = "qualified-signtool";
    for (const name of ["python.exe", "extension.pyd", "library.dll"])
      writeFileSync(join(root, name), "MZxx");
    writeFileSync(join(root, "worker.py"), "pass");
    assert.equal(signCompiledFiles(root, "windows-x64", "unsigned").compiledBinaryCount, 3);
    assert.throws(() => signCompiledFiles(root, "windows-x64", "adhoc"), /Windows/u);
    assert.throws(() => signCompiledFiles(root, "windows-x64", "production"), /signer differs/u);
    assert.equal(commands.filter(([command]) => command === "powershell.exe").length, 1);
  } finally {
    replacement.mock.restore();
    syncBuiltinESMExports();
    process.env = previous;
  }
});

test("macOS signing includes fat64 binaries and binds the approved identity and Team ID", (context) => {
  const root = mkdtempSync(join(tmpdir(), "knovum-agent-mac-signing-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const previous = process.env.APPLE_SIGNING_IDENTITY;
  const identity = "Developer ID Application: Example Company (ABCDEFGHIJ)";
  const calls = [];
  const replacement = mock.method(childProcess, "execFileSync", (command, args) => {
    calls.push([command, args]);
  });
  syncBuiltinESMExports();
  try {
    process.env.APPLE_SIGNING_IDENTITY = identity;
    for (const header of ["cffaedfe", "cafebabf", "bfbafeca"])
      writeFileSync(join(root, header), Buffer.from(header, "hex"));
    assert.throws(() => signCompiledFiles(root, "macos-x64", "unsigned"), /macOS/u);
    assert.equal(signCompiledFiles(root, "macos-x64", "production").compiledBinaryCount, 3);
    const verifications = calls.filter(([, args]) => args[0] === "--verify");
    assert.equal(verifications.length, 3);
    for (const [command, args] of verifications) {
      assert.equal(command, "codesign");
      assert.deepEqual(args.slice(0, 4), [
        "--verify",
        "--strict",
        "-R",
        `anchor apple generic and certificate leaf[subject.CN] = "${identity}" and certificate leaf[subject.OU] = "ABCDEFGHIJ"`,
      ]);
    }
  } finally {
    replacement.mock.restore();
    syncBuiltinESMExports();
    if (previous === undefined) delete process.env.APPLE_SIGNING_IDENTITY;
    else process.env.APPLE_SIGNING_IDENTITY = previous;
  }
});
