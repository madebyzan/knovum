import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { testPublicKey } from "./test-fixtures.mjs";

// Exercise the real packaging entrypoint; pnpm/Tauri is replaced by a recording executable.
// This is not a native build or a signature-verification test.
function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum-packaging-test-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  for (const path of ["tools/release", "bin", "src-tauri/resources/bundled-runtime/macos-arm64"]) {
    mkdirSync(join(root, path), { recursive: true });
  }
  const source = fileURLToPath(new URL("../../", import.meta.url));
  for (const path of [
    "tools/package-desktop.mjs",
    "tools/release/config.mjs",
    "tools/release/signing.mjs",
    "src-tauri/tauri.release.conf.json",
  ]) {
    copyFileSync(join(source, path), join(root, path));
  }
  for (const name of ["ffmpeg", "whisper-cli"]) {
    writeFileSync(join(root, "src-tauri/resources/bundled-runtime/macos-arm64", name), "fixture");
  }
  writeFileSync(join(root, "src-tauri/resources/bundled-runtime/media-import-manifest.json"), "{}");
  const recording = join(root, "command.json");
  const stub = `#!/usr/bin/env node\nrequire('node:fs').writeFileSync(process.env.KNOVUM_TEST_RECORDING, JSON.stringify(process.argv.slice(2)));\n`;
  writeFileSync(join(root, "bin/pnpm.cjs"), stub, { mode: 0o700 });
  const environment = {
    SystemRoot: process.env.SystemRoot,
    npm_execpath: join(root, "bin/pnpm.cjs"),
    KNOVUM_TEST_RECORDING: recording,
    KNOVUM_UPDATER_ENDPOINT: "https://example.com/stable/latest.json",
    KNOVUM_UPDATER_PUBKEY: testPublicKey,
    APPLE_SIGNING_IDENTITY: "Developer ID Application: Test (TESTTEAM)",
    APPLE_API_KEY: "test-api-key",
    APPLE_API_ISSUER: "test-issuer",
    APPLE_API_KEY_PATH: "test.p8",
    TAURI_SIGNING_PRIVATE_KEY: "test-private-key-never-embedded",
  };
  const run = (args, env = environment) =>
    spawnSync(
      process.execPath,
      [join(root, "tools/package-desktop.mjs"), "--runtime-target", "macos-arm64", ...args],
      { env, encoding: "utf8" },
    );
  const configuration = () => {
    const args = JSON.parse(readFileSync(recording, "utf8"));
    assert.deepEqual(args.slice(0, 3), ["exec", "tauri", "build"]);
    return JSON.parse(args[args.indexOf("--config") + 1]);
  };
  return { recording, environment, run, configuration };
}

test("release packaging forwards validated public config and preserves bundled resources", (context) => {
  const subject = fixture(context);
  const result = subject.run(["--release", "--bundles", "app,dmg"]);
  assert.equal(result.status, 0, result.stderr);
  const config = subject.configuration();
  assert.equal(config.bundle.createUpdaterArtifacts, true);
  assert.equal(config.plugins.updater.pubkey, testPublicKey);
  assert.deepEqual(config.plugins.updater.endpoints, [subject.environment.KNOVUM_UPDATER_ENDPOINT]);
  assert.ok(config.bundle.resources.includes("resources/bundled-runtime/macos-arm64/ffmpeg"));
  assert.ok(!JSON.stringify(config).includes(subject.environment.TAURI_SIGNING_PRIVATE_KEY));
});

test("QA packaging does not enable updating even when release environment variables exist", (context) => {
  const subject = fixture(context);
  const result = subject.run(["--bundles", "app,dmg"]);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(subject.configuration().plugins, undefined);
  assert.equal(subject.configuration().bundle.createUpdaterArtifacts, undefined);
});

test("missing public key stops packaging before starting Tauri", (context) => {
  const subject = fixture(context);
  const env = { ...subject.environment };
  delete env.KNOVUM_UPDATER_PUBKEY;
  const result = subject.run(["--release"], env);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /KNOVUM_UPDATER_PUBKEY/);
  assert.equal(existsSync(subject.recording), false);
});

test("a caller cannot replace the validated release config with a second config", (context) => {
  const subject = fixture(context);
  const result = subject.run(["--release", "--config", "unsafe.json"]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /does not permit/);
  assert.equal(existsSync(subject.recording), false);
});

test("inherited Tauri configuration cannot bypass the release policy", (context) => {
  const subject = fixture(context);
  const result = subject.run(["--release"], { ...subject.environment, TAURI_CONFIG: "{}" });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /TAURI_CONFIG override/);
  assert.equal(existsSync(subject.recording), false);
});

test("inline target values cannot select a different bundled runtime", (context) => {
  const subject = fixture(context);
  const result = subject.run(["--target=x86_64-pc-windows-msvc"]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /does not match/);
});

test("missing or repeated target arguments fail instead of choosing the host", (context) => {
  const subject = fixture(context);
  assert.match(subject.run(["--target"]).stderr, /Missing packaging option/);
  assert.match(subject.run(["--target=", "--bundles", "app"]).stderr, /Missing packaging option/);
  assert.match(
    subject.run(["--target=aarch64-apple-darwin", "--target", "aarch64-apple-darwin"]).stderr,
    /Duplicate packaging option/,
  );
});
