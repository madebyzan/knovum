import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

const tag = `runtime-${"a".repeat(40)}-1-1`;
const provisionArgs = [
  "--tag",
  tag,
  "--repository",
  "owner/distribution",
  "--source-repository",
  "owner/source",
  "--target",
  "windows-x64",
];

function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum-agent-provision-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const directory = join(root, "scripts/runtime");
  mkdirSync(directory, { recursive: true });
  copyFileSync(new URL("../provision.mjs", import.meta.url), join(directory, "provision.mjs"));
  for (const name of ["archive-policy", "distribution", "files", "profiles"])
    writeFileSync(
      join(directory, `${name}.mjs`),
      `export * from ${JSON.stringify(new URL(`../${name}.mjs`, import.meta.url).href)};`,
    );
  writeFileSync(
    join(root, "transport.mjs"),
    `
    import childProcess from 'node:child_process';
    import { syncBuiltinESMExports } from 'node:module';
    import { appendFileSync, writeFileSync } from 'node:fs';
    import { join } from 'node:path';
    childProcess.execFileSync = (command, args) => {
      appendFileSync('events.jsonl', JSON.stringify({command,args})+'\\n');
      if (command !== 'gh') throw new Error('Extraction must not begin');
      if (args[0] === 'attestation') {
        if (process.env.REJECT_ATTESTATION === 'true') throw new Error('Invalid provenance');
        return '';
      }
      const root = args[args.indexOf('--dir')+1];
      const prefix = 'python-agent-windows-x64';
      writeFileSync(join(root,prefix+'.tar.gz'), 'archive');
      writeFileSync(join(root,prefix+'-evidence.tar.gz'), 'evidence');
      const descriptor = {schemaVersion:1, target:'windows-x64', profile:'agent', version:'3.11.17',
        dependencyRequirementsSha256:'a'.repeat(64), dependencyLockSha256:'b'.repeat(64),
        archiveUrl:'https://github.com/owner/distribution/releases/download/${tag}/'+prefix+'.tar.gz',
        size:7, sha256:'c'.repeat(64), manifestSha256:'d'.repeat(64)};
      if (process.env.WRONG_TARGET === 'true') descriptor.target = 'macos-x64';
      writeFileSync(join(root,prefix+'-descriptor.json'), JSON.stringify(descriptor));
      return '';
    };
    syncBuiltinESMExports();
  `,
  );
  return root;
}

function launch(root, options = {}, flags = provisionArgs) {
  return spawnSync(
    process.execPath,
    ["--import", "./transport.mjs", "scripts/runtime/provision.mjs", ...flags],
    {
      cwd: root,
      encoding: "utf8",
      timeout: 5000,
      env: { ...process.env, ...options },
    },
  );
}
const events = (root) => readFileSync(join(root, "events.jsonl"), "utf8").trim().split("\n").map(JSON.parse);

test("an untrusted attestation blocks extraction and activation", (context) => {
  const root = fixture(context);
  const execution = launch(root, { REJECT_ATTESTATION: "true" });
  assert.equal(execution.status, 1);
  assert.match(execution.stderr, /Invalid provenance/u);
  assert.deepEqual(
    events(root).map(({ args }) => args[0]),
    ["release", "attestation"],
  );
});

test("all assets are provenance-verified and tampered archive hashes block extraction", (context) => {
  const root = fixture(context);
  const execution = launch(root);
  assert.equal(execution.status, 1);
  assert.match(execution.stderr, /descriptor differs/u);
  const verified = events(root).filter(({ args }) => args[0] === "attestation");
  assert.equal(verified.length, 3);
  for (const { args } of verified) {
    assert.equal(
      args[args.indexOf("--signer-workflow") + 1],
      "owner/source/.github/workflows/python-runtime.yml",
    );
    assert.equal(args[args.indexOf("--source-digest") + 1], "a".repeat(40));
    assert.equal(args[args.indexOf("--source-ref") + 1], "refs/heads/main");
    assert.ok(args.includes("--deny-self-hosted-runners"));
  }
});

test("an attested descriptor for another target cannot be installed", (context) => {
  const root = fixture(context);
  const execution = launch(root, { WRONG_TARGET: "true" });
  assert.equal(execution.status, 1);
  assert.match(execution.stderr, /Invalid Python runtime descriptor/u);
  assert.equal(
    events(root).some(({ command }) => command === "tar"),
    false,
  );
});
