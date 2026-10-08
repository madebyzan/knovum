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
import test from "node:test";

function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum-desktop-startup-"));
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
  return root;
}

const launch = (root, args) =>
  spawnSync(process.execPath, ["scripts/desktop/dev.mjs", ...args], {
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
    const result = launch(root, args);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(events(root), [{ stage: "prepared" }, { stage: "tauri", args }]);
  });
}

test("failed resource preparation stops startup with the actual error", (context) => {
  const root = fixture(context);
  writeFileSync(join(root, "broken-resource"), "");
  const result = launch(root, ["dev"]);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Core media verification failed/u);
  assert.equal(existsSync(join(root, "events.jsonl")), false);
});

for (const args of [["dev", "--help"], ["dev", "-h"], ["dev", "--version"], ["info"]]) {
  test(`${args.join(" ")} runs without preparing resources`, (context) => {
    const root = fixture(context);
    writeFileSync(join(root, "broken-resource"), "");
    const result = launch(root, args);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(events(root), [{ stage: "tauri", args }]);
  });
}

test("both development commands use preflight and the Tauri hook only starts Vite", () => {
  const configuration = JSON.parse(
    readFileSync(new URL("../../../src-tauri/tauri.conf.json", import.meta.url)),
  );
  const { scripts } = JSON.parse(readFileSync(new URL("../../../package.json", import.meta.url)));
  assert.equal(configuration.build.beforeDevCommand, "pnpm dev");
  assert.equal(scripts.tauri, "node scripts/desktop/dev.mjs");
  assert.equal(scripts["desktop:dev"], "node scripts/desktop/dev.mjs dev");
});
