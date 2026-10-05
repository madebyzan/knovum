import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

function fixture(context) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "knovum tauri runner ")));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const tools = join(root, "tools", "desktop");
  const cli = join(root, "node_modules", "@tauri-apps", "cli");
  mkdirSync(tools, { recursive: true });
  mkdirSync(cli, { recursive: true });
  copyFileSync(new URL("../tauri.mjs", import.meta.url), join(tools, "tauri.mjs"));
  writeFileSync(join(cli, "package.json"), JSON.stringify({ name: "@tauri-apps/cli" }));
  writeFileSync(
    join(cli, "tauri.js"),
    `require("node:fs").writeFileSync(process.env.KNOVUM_TEST_OUTPUT, JSON.stringify({
      args: process.argv.slice(2), cwd: process.cwd(),
      ffmpeg: process.env.KNOVUM_FFMPEG_PATH
    }));
    process.exitCode = Number(process.env.KNOVUM_TEST_EXIT || 0);`,
  );
  const output = join(root, "cli-output.json");
  const env = { ...process.env, KNOVUM_TEST_OUTPUT: output };
  delete env.KNOVUM_FFMPEG_PATH;
  delete env.KNOVUM_TEST_EXIT;
  return {
    root,
    run: (args, overrides = {}) =>
      spawnSync(process.execPath, [join(tools, "tauri.mjs"), ...args], {
        cwd: tmpdir(),
        env: { ...env, ...overrides },
        encoding: "utf8",
        timeout: 10_000,
      }),
    read: () => JSON.parse(readFileSync(output, "utf8")),
  };
}

test("forwards exact arguments and uses the repository cwd without a pnpm shim", (context) => {
  const sample = fixture(context);
  const args = ["build", "--config", '{"productName":"Knovum & $HOME %PATH%"}', "a b", 'a"b', ""];
  const execution = sample.run(args, { KNOVUM_FFMPEG_PATH: "explicit ffmpeg path" });
  assert.equal(execution.status, 0, execution.stderr);
  assert.deepEqual(sample.read(), {
    args,
    cwd: sample.root,
    ffmpeg: "explicit ffmpeg path",
  });
});

test("non-development commands do not require a provisioned runtime", (context) => {
  const sample = fixture(context);
  const execution = sample.run(["--version"]);
  assert.equal(execution.status, 0, execution.stderr);
  assert.deepEqual(sample.read().args, ["--version"]);
});

test("a failed Tauri command cannot be reported as success", (context) => {
  const sample = fixture(context);
  const execution = sample.run(["build"], { KNOVUM_TEST_EXIT: "23" });
  assert.equal(execution.error, undefined);
  assert.equal(execution.signal, null);
  assert.notEqual(execution.status, 0);
});

const platform = { darwin: "macos", win32: "windows" }[process.platform];
if (platform) {
  test("development binds provisioned host tools before invoking Tauri", (context) => {
    const sample = fixture(context);
    const target = `${platform}-${process.arch}`;
    const directory = join(sample.root, "src-tauri", "resources", "bundled-runtime", target);
    mkdirSync(directory, { recursive: true });
    const ffmpeg = join(directory, process.platform === "win32" ? "ffmpeg.exe" : "ffmpeg");
    writeFileSync(ffmpeg, "fixture");
    const execution = sample.run(["dev"]);
    assert.equal(execution.status, 0, execution.stderr);
    assert.deepEqual(sample.read(), { args: ["dev"], cwd: sample.root, ffmpeg });
  });

  test("development preserves explicit runtime overrides", (context) => {
    const sample = fixture(context);
    const execution = sample.run(["dev"], {
      KNOVUM_FFMPEG_PATH: "custom ffmpeg",
    });
    assert.equal(execution.status, 0, execution.stderr);
    assert.deepEqual(sample.read(), {
      args: ["dev"],
      cwd: sample.root,
      ffmpeg: "custom ffmpeg",
    });
  });

  test("development rejects a missing runtime", (context) => {
    const sample = fixture(context);
    const execution = sample.run(["dev"]);
    assert.notEqual(execution.status, 0);
    assert.match(execution.stderr, /Development runtime is missing/);
  });
} else {
  test("development rejects unsupported hosts", (context) => {
    const sample = fixture(context);
    const execution = sample.run(["dev"]);
    assert.notEqual(execution.status, 0);
    assert.match(execution.stderr, /Unsupported desktop development host/);
  });
}
