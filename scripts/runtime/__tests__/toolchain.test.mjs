import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { sha256File } from "../files.mjs";
import { hostRuntimeTarget } from "../profiles.mjs";
import { ensureUv } from "../toolchain.mjs";

function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum toolchain 中文 "));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const target = "macos-x64";
  const archiveSha = "a".repeat(64);
  const directory = join(root, ".runtime-development", "tools", target, archiveSha);
  mkdirSync(directory, { recursive: true });
  const executable = join(directory, "uv");
  writeFileSync(executable, "official test binary");
  const source = {
    url: "https://example.invalid/uv.tar.gz",
    sha256: archiveSha,
    size: 1,
    entry: "uv/uv",
    executableSha256: sha256File(executable),
  };
  return { root, target, executable, source };
}

test("reuses only the exact pinned executable without network access", async (context) => {
  const sample = fixture(context);
  assert.equal(await ensureUv(sample.root, sample.target, sample.source), sample.executable);
});

test("a modified cached tool is rejected instead of executing or silently replacing it", async (context) => {
  const sample = fixture(context);
  writeFileSync(sample.executable, "modified binary");
  await assert.rejects(ensureUv(sample.root, sample.target, sample.source), /executable digest/);
});

test("tool installation rejects untrusted transport before downloading", async (context) => {
  const sample = fixture(context);
  rmSync(dirname(sample.executable), { recursive: true });
  sample.source.url = "http://example.invalid/uv.tar.gz";
  await assert.rejects(ensureUv(sample.root, sample.target, sample.source), /HTTPS/);
});

test(
  "core media runs with pinned Python and no publication packages",
  { skip: process.platform !== "darwin" },
  (context) => {
    const sample = fixture(context);
    const target = hostRuntimeTarget();
    const directory = join(sample.root, ".runtime-development/tools", target, sample.source.sha256);
    mkdirSync(directory, { recursive: true });
    const executable = join(directory, "uv");
    writeFileSync(
      executable,
      `#!${process.execPath}\nprocess.stdout.write(JSON.stringify(process.argv.slice(2)));\n`,
    );
    chmodSync(executable, 0o755);
    mkdirSync(join(sample.root, "python"));
    writeFileSync(
      join(sample.root, "python/runtime-source.json"),
      JSON.stringify({
        version: "3.11.17",
        uv: { targets: { [target]: { ...sample.source, executableSha256: sha256File(executable) } } },
      }),
    );
    const producer = fileURLToPath(new URL("../producer.mjs", import.meta.url));
    const execution = spawnSync(
      process.execPath,
      [
        "--input-type=module",
        "-e",
        `import { runProducer } from ${JSON.stringify(producer)}; await runProducer(process.argv[1], ["--development"], "build_media.py");`,
        sample.root,
      ],
      { encoding: "utf8" },
    );
    assert.equal(execution.status, 0, execution.stderr);
    const args = JSON.parse(execution.stdout);
    assert.deepEqual(args.slice(0, 9), [
      "run",
      "--no-project",
      "--python",
      "3.11.17",
      "python",
      "-I",
      "-B",
      "-X",
      "utf8",
    ]);
    assert.equal(args.at(-1), "--development");
  },
);
