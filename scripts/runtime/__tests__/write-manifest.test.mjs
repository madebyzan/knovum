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
const supportedHost = ["macos-arm64", "macos-x64", "windows-x64"].includes(target);

function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum-runtime-manifest-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const scriptDirectory = join(root, "scripts", "runtime");
  const python = join(root, "python/runtimes/resources");
  const runtime = join(root, "runtime");
  mkdirSync(scriptDirectory, { recursive: true });
  mkdirSync(python, { recursive: true });
  mkdirSync(join(runtime, "packages"), { recursive: true });
  copyFileSync(
    new URL("../write-manifest.mjs", import.meta.url),
    join(scriptDirectory, "write-manifest.mjs"),
  );
  copyFileSync(new URL("../files.mjs", import.meta.url), join(scriptDirectory, "files.mjs"));
  copyFileSync(new URL("../profiles.mjs", import.meta.url), join(scriptDirectory, "profiles.mjs"));
  writeFileSync(
    join(python, "uv.lock"),
    'source = { git = "https://github.com/OpenBMB/VoxCPM.git?rev=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa#aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" }\n',
  );
  const executable = platform === "windows" ? join(runtime, "python.exe") : join(runtime, "bin", "python3");
  mkdirSync(join(runtime, "bin"), { recursive: true });
  writeFileSync(executable, "interpreter fixture");
  if (platform !== "windows") chmodSync(executable, 0o755);
  writeFileSync(join(runtime, "runtime-requirements.txt"), "pass\n");
  return {
    root,
    runtime,
    run: (requestedTarget = target) =>
      spawnSync(
        process.execPath,
        [
          join(scriptDirectory, "write-manifest.mjs"),
          "--target",
          requestedTarget,
          "--directory",
          runtime,
          "--profile",
          "resources",
        ],
        { encoding: "utf8" },
      ),
  };
}

test(
  "manifest binds files to runtime requirements without application source",
  { skip: !supportedHost },
  (context) => {
    const sample = fixture(context);
    const execution = sample.run();
    assert.equal(execution.status, 0, execution.stderr);
    const manifest = JSON.parse(readFileSync(join(sample.runtime, "runtime-manifest.json"), "utf8"));
    const requirements = readFileSync(join(sample.runtime, "runtime-requirements.txt"));
    assert.equal(manifest.target, target);
    assert.equal(
      manifest.dependencyRequirementsSha256,
      createHash("sha256").update(requirements).digest("hex"),
    );
    assert.equal(manifest.files.length, 2);
    assert.equal(manifest.files[0].sha256.length, 64);
  },
);

test("manifest refuses links inside the runtime", { skip: !supportedHost }, (context) => {
  const sample = fixture(context);
  symlinkSync(join(sample.root, "python", "uv.lock"), join(sample.runtime, "outside-link"));
  const execution = sample.run();
  assert.notEqual(execution.status, 0);
  assert.match(execution.stderr, /contains a link/);
});

test("manifest rejects a target outside the release scope", { skip: !platform }, (context) => {
  const sample = fixture(context);
  const execution = sample.run("windows-arm64");
  assert.notEqual(execution.status, 0);
  assert.match(execution.stderr, /Unsupported runtime target/);
});
