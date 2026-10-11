import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { appendFileSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { listRuntimeFiles } from "../../files.mjs";

async function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), "knovum-preparation-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const runtime = join(root, "scripts/runtime");
  cpSync(new URL("../../", import.meta.url), runtime, {
    recursive: true,
    filter: (path) => !path.includes("__tests__"),
  });
  mkdirSync(join(root, "src-tauri"));
  writeFileSync(
    join(root, "src-tauri/tauri.conf.json"),
    JSON.stringify({ bundle: { macOS: { minimumSystemVersion: "14.0" } } }),
  );
  const bytes = Buffer.from("fixture ONNX library");
  const file = {
    path: "onnxruntime.dll",
    size: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
  writeFileSync(
    join(runtime, "inference/onnx-source.json"),
    JSON.stringify({ files: { "windows-x64": [file] } }),
  );
  const actualOnnx = new URL("../onnx.mjs", import.meta.url).href;
  writeFileSync(
    join(runtime, "inference/onnx.mjs"),
    `
    import { mkdirSync, writeFileSync } from 'node:fs';
    import { join } from 'node:path';
    export { verifyOnnxRuntime } from ${JSON.stringify(actualOnnx)};
    export async function prepareOnnxRuntime(root, target) {
      const directory = join(root, 'fixture-onnx');
      mkdirSync(directory, { recursive: true });
      writeFileSync(join(directory, 'onnxruntime.dll'), ${JSON.stringify(bytes.toString())});
      const path = join(directory, 'inference-runtime-manifest.json');
      writeFileSync(path, JSON.stringify({ schemaVersion: 1, target, onnxRuntime: 'onnxruntime.dll', files: [${JSON.stringify(file)}] }));
      return path;
    }
  `,
  );
  for (const [name, fn, output] of [
    [
      "audio",
      "buildAudioLibraries",
      ["compiled-libraries/audiocpp.dll", "compiled-libraries/knovum_voice.dll"],
    ],
    ["speech", "buildSpeechLibraries", ["compiled-libraries/sherpa-onnx-c-api.dll"]],
    ["whisper", "buildWhisperLibraries", ["compiled-libraries/whisper/whisper.dll"]],
  ]) {
    writeFileSync(
      join(runtime, `inference/${name}.mjs`),
      `
      import { appendFileSync, existsSync, mkdirSync, writeFileSync } from 'node:fs';
      import { dirname, join } from 'node:path';
      export async function ${fn}(...args) {
        const root = ${JSON.stringify(root)};
        const destination = args[${name === "audio" ? 2 : 1}];
        appendFileSync(join(root, '${name}-calls'), 'build\\n');
        if ('${name}' === 'whisper' && existsSync(join(root, 'fail-whisper'))) throw new Error('Whisper download failed');
        for (const file of ${JSON.stringify(output)}) {
          mkdirSync(dirname(join(destination, file)), { recursive: true });
          writeFileSync(join(destination, file), '${name} compiled library');
        }
      }
    `,
    );
  }
  const { prepareInferenceRuntime } = await import(pathToFileURL(join(runtime, "inference/preparation.mjs")));
  return { root, prepare: () => prepareInferenceRuntime(root, "windows-x64") };
}

const calls = (root, name) =>
  readFileSync(join(root, `${name}-calls`), "utf8")
    .trim()
    .split("\n").length;

test("a later supplier failure preserves earlier compiled suppliers for the next startup", async (context) => {
  const { root, prepare } = await fixture(context);
  writeFileSync(join(root, "fail-whisper"), "");
  await assert.rejects(prepare(), /Whisper download failed/u);
  rmSync(join(root, "fail-whisper"));
  const manifest = await prepare();
  assert.equal(JSON.parse(readFileSync(manifest, "utf8")).files.length, 5);
  assert.equal(calls(root, "audio"), 1);
  assert.equal(calls(root, "speech"), 1);
  assert.equal(calls(root, "whisper"), 2);
  await prepare();
  assert.equal(calls(root, "whisper"), 2);
});

test("changing Whisper preparation does not invalidate audio and Sherpa builds", async (context) => {
  const { root, prepare } = await fixture(context);
  await prepare();
  appendFileSync(join(root, "scripts/runtime/inference/whisper.mjs"), "\n// changed Whisper build recipe\n");
  await prepare();
  assert.equal(calls(root, "audio"), 1);
  assert.equal(calls(root, "speech"), 1);
  assert.equal(calls(root, "whisper"), 2);
});

test("a cached supplier with changed bytes fails verification before reuse", async (context) => {
  const { root, prepare } = await fixture(context);
  writeFileSync(join(root, "fail-whisper"), "");
  await assert.rejects(prepare(), /Whisper download failed/u);
  const files = listRuntimeFiles(join(root, ".runtime-development"));
  const audio = files.find((path) => path.endsWith("compiled-libraries/audiocpp.dll"));
  assert(audio, "completed audio supplier must survive the later failure");
  writeFileSync(join(root, ".runtime-development", audio), "tampered library");
  rmSync(join(root, "fail-whisper"));
  await assert.rejects(prepare(), /integrity/u);
});
