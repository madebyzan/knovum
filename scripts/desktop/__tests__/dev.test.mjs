import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
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
  const root = realpathSync(mkdtempSync(join(tmpdir(), "knovum-desktop-startup-")));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  for (const directory of ["scripts/desktop", "scripts/runtime", "node_modules/@tauri-apps/cli"]) {
    mkdirSync(join(root, directory), { recursive: true });
  }
  copyFileSync(new URL("../dev.mjs", import.meta.url), join(root, "scripts/desktop/dev.mjs"));
  writeFileSync(
    join(root, "node_modules/@tauri-apps/cli/package.json"),
    JSON.stringify({
      type: "module",
      main: "main.mjs",
    }),
  );
  writeFileSync(
    join(root, "node_modules/@tauri-apps/cli/main.mjs"),
    `
    import { appendFileSync } from 'node:fs';
    export async function run(args) {
      appendFileSync('events.jsonl', JSON.stringify({ stage: 'tauri', args }) + '\\n');
    }
  `,
  );
  writeFileSync(
    join(root, "scripts/runtime/media.mjs"),
    `
    import { appendFileSync, existsSync } from 'node:fs';
    import { join } from 'node:path';
    export async function prepareCoreMedia(repository) {
      await new Promise(resolve => setTimeout(resolve, 30));
      if (existsSync(join(repository, 'broken-resource'))) throw new Error('Core media verification failed');
      appendFileSync(join(repository, 'events.jsonl'), JSON.stringify({ stage: 'prepared' }) + '\\n');
    }
  `,
  );
  writeFileSync(
    join(root, "scripts/runtime/native.mjs"),
    `
    import { appendFileSync, existsSync } from 'node:fs';
    import { join } from 'node:path';
    export async function prepareNativeRuntime(repository) {
      if (existsSync(join(repository,'broken-native'))) throw new Error('Native worker verification failed');
      appendFileSync(join(repository,'events.jsonl'),JSON.stringify({stage:'native'})+'\\n');
    }
  `,
  );
  writeFileSync(
    join(root, "cargo-fixture.mjs"),
    `
    import childProcess from 'node:child_process';
    import { syncBuiltinESMExports } from 'node:module';
    import { appendFileSync, existsSync } from 'node:fs';
    childProcess.execFileSync = (command, args, options) => {
      if (existsSync('broken-build')) throw new Error('Native worker compilation failed');
      appendFileSync('events.jsonl', JSON.stringify({ stage: command, args, cwd: options.cwd }) + '\\n');
    };
    syncBuiltinESMExports();
  `,
  );
  return root;
}

const launch = (root, args) =>
  spawnSync(process.execPath, ["--import", "./cargo-fixture.mjs", "scripts/desktop/dev.mjs", ...args], {
    cwd: root,
    encoding: "utf8",
    timeout: 5000,
  });
const events = (root) => readFileSync(join(root, "events.jsonl"), "utf8").trim().split("\n").map(JSON.parse);

for (const args of [
  ["dev", "--no-watch", "--", "--help"],
  ["--verbose", "dev"],
]) {
  test(`${args.join(" ")}: resources finish before Tauri starts its frontend wait`, (context) => {
    const root = fixture(context);
    const execution = launch(root, args);
    assert.equal(execution.status, 0, execution.stderr);
    assert.deepEqual(events(root), [
      { stage: "prepared" },
      { stage: "native" },
      {
        stage: "cargo",
        args: [
          "build",
          "--locked",
          "--manifest-path",
          "src-tauri/Cargo.toml",
          "-p",
          "knovum-inference-worker",
        ],
        cwd: root,
      },
      { stage: "tauri", args },
    ]);
  });
}

test("failed resource preparation stops startup with the actual error", (context) => {
  const root = fixture(context);
  writeFileSync(join(root, "broken-resource"), "");
  const execution = launch(root, ["dev"]);
  assert.equal(execution.status, 1);
  assert.match(execution.stderr, /Core media verification failed/u);
  assert.equal(existsSync(join(root, "events.jsonl")), false);
});

for (const args of [["dev", "--help"], ["dev", "-h"], ["dev", "--version"], ["info"]]) {
  test(`${args.join(" ")} runs without preparing resources`, (context) => {
    const root = fixture(context);
    writeFileSync(join(root, "broken-resource"), "");
    const execution = launch(root, args);
    assert.equal(execution.status, 0, execution.stderr);
    assert.deepEqual(events(root), [{ stage: "tauri", args }]);
  });
}

test("desktop development uses preflight and the Tauri hook only starts Vite", () => {
  const configuration = JSON.parse(
    readFileSync(new URL("../../../src-tauri/tauri.conf.json", import.meta.url)),
  );
  const { scripts } = JSON.parse(readFileSync(new URL("../../../package.json", import.meta.url)));
  assert.equal(configuration.build.beforeDevCommand, "pnpm dev");
  assert.equal(Object.hasOwn(scripts, "tauri"), false);
  assert.equal(scripts["desktop:dev"], "node scripts/desktop/dev.mjs dev");
});

test("invalid native worker prevents launching the desktop", (context) => {
  const root = fixture(context);
  writeFileSync(join(root, "broken-native"), "");
  const execution = launch(root, ["dev"]);
  assert.equal(execution.status, 1);
  assert.match(execution.stderr, /Native worker verification failed/u);
  assert.deepEqual(events(root), [{ stage: "prepared" }]);
});

test("failed worker compilation prevents launching the desktop", (context) => {
  const root = fixture(context);
  writeFileSync(join(root, "broken-build"), "");
  const execution = launch(root, ["dev"]);
  assert.equal(execution.status, 1);
  assert.match(execution.stderr, /Native worker compilation failed/u);
  assert.deepEqual(events(root), [{ stage: "prepared" }, { stage: "native" }]);
});
