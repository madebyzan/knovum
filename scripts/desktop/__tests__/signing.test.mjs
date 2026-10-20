import assert from "node:assert/strict";
import childProcess from "node:child_process";
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { mock, test } from "node:test";

import { signCompiledFiles, verifyCompiledFiles } from "../signing.mjs";

test("Windows runtime packaging uses unsigned binaries without requiring Authenticode", () => {
  const directory = mkdtempSync(join(tmpdir(), "knovum-signing-test-"));
  try {
    for (const name of ["worker.exe", "vcruntime140.dll"]) writeFileSync(join(directory, name), "MZxx");
    const signing = signCompiledFiles(directory, "windows-x64");
    assert.equal(signing.identity, "unsigned");
    assert.equal(signing.compiledBinaryCount, 2);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("macOS signing uses ad-hoc codesign on compiled binaries", () => {
  const root = mkdtempSync(join(tmpdir(), "knovum-mac-signing-"));
  const calls = [];
  const replacement = mock.method(childProcess, "execFileSync", (command, args) => {
    calls.push([command, args]);
  });
  syncBuiltinESMExports();
  try {
    for (const header of ["cffaedfe", "cafebabf", "bfbafeca"])
      writeFileSync(join(root, header), Buffer.from(header, "hex"));
    const signing = signCompiledFiles(root, "macos-x64");
    assert.equal(signing.identity, "-");
    assert.equal(signing.compiledBinaryCount, 3);
    const signCalls = calls.filter(([, args]) => args[0] === "--force" && args[1] === "--sign");
    assert.equal(signCalls.length, 3);
  } finally {
    replacement.mock.restore();
    syncBuiltinESMExports();
    rmSync(root, { recursive: true, force: true });
  }
});

test("pre-signed Agent binaries are verified with codesign", () => {
  const root = mkdtempSync(join(tmpdir(), "knovum-agent-signed-"));
  const calls = [];
  const replacement = mock.method(childProcess, "execFileSync", (command, args) => {
    calls.push([command, args]);
  });
  syncBuiltinESMExports();
  try {
    writeFileSync(join(root, "knovum-agent"), Buffer.from("cffaedfe", "hex"));
    if (process.platform !== "win32") symlinkSync("knovum-agent", join(root, "alias"));
    verifyCompiledFiles(root, "macos-x64");
    assert.equal(calls.length, 1);
    assert.equal(calls[0][0], "codesign");
    assert.equal(calls[0][1][0], "--verify");
  } finally {
    replacement.mock.restore();
    syncBuiltinESMExports();
    rmSync(root, { recursive: true, force: true });
  }
});
