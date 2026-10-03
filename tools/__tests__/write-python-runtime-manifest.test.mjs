import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

const platform = { darwin: "macos", win32: "windows" }[process.platform];
const target = `${platform}-${process.arch}`;
const supportedHost = ["macos-arm64", "windows-x64"].includes(target);

function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-manifest-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const tools = join(root, "tools");
  const python = join(root, "python");
  const runtime = join(root, "runtime");
  mkdirSync(tools);
  mkdirSync(python);
  mkdirSync(join(runtime, "packages", "knovum_inference"), { recursive: true });
  copyFileSync(
    new URL("../write-python-runtime-manifest.mjs", import.meta.url),
    join(tools, "write-python-runtime-manifest.mjs"),
  );
  writeFileSync(join(python, "uv.lock"), "locked dependency fixture\n");
  const executable = platform === "windows" ? join(runtime, "python.exe") : join(runtime, "bin", "python3");
  mkdirSync(join(runtime, "bin"), { recursive: true });
  writeFileSync(executable, "interpreter fixture");
  if (platform !== "windows") chmodSync(executable, 0o755);
  writeFileSync(join(runtime, "packages", "knovum_inference", "__main__.py"), "pass\n");
  return {
    root,
    runtime,
    run: (requestedTarget = target) =>
      spawnSync(
        process.execPath,
        [
          join(tools, "write-python-runtime-manifest.mjs"),
          "--target",
          requestedTarget,
          "--directory",
          runtime,
        ],
        { encoding: "utf8" },
      ),
  };
}

test("manifest binds every file to the exact dependency lock", { skip: !supportedHost }, (context) => {
  const sample = fixture(context);
  const result = sample.run();
  assert.equal(result.status, 0, result.stderr);
  const manifest = JSON.parse(readFileSync(join(sample.runtime, "runtime-manifest.json"), "utf8"));
  const lock = readFileSync(join(sample.root, "python", "uv.lock"));
  assert.equal(manifest.target, target);
  assert.equal(manifest.dependencyLockSha256, createHash("sha256").update(lock).digest("hex"));
  assert.equal(manifest.files.length, 2);
  assert.equal(manifest.files[0].sha256.length, 64);
});

test("manifest refuses links inside the runtime", { skip: !supportedHost }, (context) => {
  const sample = fixture(context);
  symlinkSync(join(sample.root, "python", "uv.lock"), join(sample.runtime, "outside-link"));
  const result = sample.run();
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /contains a link/);
});

test("manifest rejects a target outside the release scope", { skip: !platform }, (context) => {
  const sample = fixture(context);
  const result = sample.run("macos-x64");
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Unsupported runtime target/);
});
