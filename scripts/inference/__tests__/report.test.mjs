import assert from "node:assert/strict";
import { cpSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { cosine } from "../embedding-evaluation.mjs";
import { digest, hashedFile } from "../evidence.mjs";
import { evaluateFiles } from "../report.mjs";
import { percentile, tokenDistance, tokens } from "../speech-evaluation.mjs";

function fixture(context, kind = "core") {
  const directory = mkdtempSync(join(tmpdir(), "native-acceptance-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  cpSync(fileURLToPath(new URL(`./fixtures/${kind}`, import.meta.url)), directory, { recursive: true });
  return { directory, corpus: join(directory, "corpus.json"), capture: join(directory, "capture.json") };
}
function mutateOutput(sample, capability, mutate) {
  const capture = JSON.parse(readFileSync(sample.capture)),
    reference = capture.capabilities[capability].candidateOutput;
  const output = JSON.parse(readFileSync(join(sample.directory, reference.path)));
  mutate(output);
  const bytes = JSON.stringify(output);
  writeFileSync(join(sample.directory, reference.path), bytes);
  reference.sha256 = digest(bytes);
  writeFileSync(sample.capture, JSON.stringify(capture));
}
test("synthetic complete chain preserves every scoring metric", (context) => {
  const f = fixture(context),
    result = evaluateFiles(f.corpus, f.capture);
  assert.equal(result.status, "passed");
  assert.equal(result.capabilities.embedding.minimumPairedCosine, 1);
  assert.equal(result.capabilities.asr.wer.candidate, 0);
  assert.equal(result.capabilities.alignment.overall.coverage, 1);
  assert.equal(result.capabilities.tts.minimumRatersPerSample, 3);
});
test("silence hallucinations contribute to micro-aggregated WER", (context) => {
  const f = fixture(context);
  mutateOutput(f, "asr", (v) => {
    v.values.transcripts.silence = "hallucinated on silence";
  });
  const result = evaluateFiles(f.corpus, f.capture);
  assert.equal(result.status, "failed");
  assert.equal(result.capabilities.asr.wer.candidate, 1);
});
for (const field of ["workerDigest", "runtimeDigest", "modelSpecId", "configId", "method"])
  test(`rejects forged ${field}`, (context) => {
    const f = fixture(context);
    mutateOutput(f, "embedding", (v) => {
      v.source[field] = field.endsWith("Digest") ? "b".repeat(64) : "forged";
    });
    assert.throws(() => evaluateFiles(f.corpus, f.capture), /Execution/u);
  });
test("rejects duplicate and reversed invocation evidence", (context) => {
  const f = fixture(context);
  mutateOutput(f, "embedding", (v) => {
    v.source.invocations.push(v.source.invocations[0]);
  });
  assert.throws(() => evaluateFiles(f.corpus, f.capture), /duplicate/u);
});
test("missing model evidence remains unavailable", (context) => {
  const f = fixture(context),
    capture = JSON.parse(readFileSync(f.capture));
  capture.capabilities.tts = { status: "unavailable", reason: "not measured" };
  writeFileSync(f.capture, JSON.stringify(capture));
  assert.equal(evaluateFiles(f.corpus, f.capture).status, "unavailable");
});
test("hashed references reject traversal, links and changed bytes", (context) => {
  const f = fixture(context);
  assert.throws(() => hashedFile({ path: "../escape", sha256: "a".repeat(64) }, f.directory));
  symlinkSync(f.corpus, join(f.directory, "linked"));
  assert.throws(
    () => hashedFile({ path: "linked", sha256: digest(readFileSync(f.corpus)) }, f.directory),
    /symbolic/u,
  );
  assert.throws(() => hashedFile({ path: "corpus.json", sha256: "a".repeat(64) }, f.directory), /identity/u);
});
for (const capability of ["repair", "upscale", "selection", "depth"])
  test(`synthetic ${capability} tensor contract`, (context) => {
    const f = fixture(context, capability);
    assert.equal(evaluateFiles(f.corpus, f.capture, "images").status, "passed");
  });
test("image quality failure and reference-only execution cannot qualify", (context) => {
  const f = fixture(context, "repair"),
    capture = JSON.parse(readFileSync(f.capture));
  capture.models[0].artifacts[0].qualityPassed = false;
  writeFileSync(f.capture, JSON.stringify(capture));
  assert.equal(evaluateFiles(f.corpus, f.capture, "images").status, "failed");
  capture.models[0].artifacts[0].qualityPassed = true;
  capture.models[0].source.kind = "official_python_reference";
  writeFileSync(f.capture, JSON.stringify(capture));
  assert.equal(evaluateFiles(f.corpus, f.capture, "images").status, "unavailable");
});
test("metric primitives cover Unicode, insertion, quantiles and invalid vectors", () => {
  assert.deepEqual(tokens("Ｈｅｌｌｏ, IT'S 2026", "en-US"), ["hello", "it's", "2026"]);
  assert.deepEqual(tokens("你好，世界!", "zh-CN"), ["你", "好", "世", "界"]);
  assert.equal(tokenDistance(["one", "two"], ["one", "three", "four"]), 2);
  assert(Math.abs(percentile([0, 10, 20, 30], 0.95) - 28.5) < 1e-12);
  assert.equal(cosine([1, 0], [1, 0]), 1);
  assert.throws(() => cosine([0], [0]));
  assert.throws(() => cosine([NaN], [1]));
});

function mutateCapture(sample, mutate) {
  const value = JSON.parse(readFileSync(sample.capture));
  mutate(value);
  writeFileSync(sample.capture, JSON.stringify(value));
}
for (const [label, mutate, pattern] of [
  [
    "review after unblinding",
    (s) => {
      s.reviews[0].reviewedAt = s.blinding.revealedAfterReviewsAt;
    },
    /unblinding/u,
  ],
  [
    "duplicate reviewer",
    (s) => {
      s.reviews.push(s.reviews[0]);
    },
    /duplicate/u,
  ],
  [
    "insufficient blind reviewers",
    (s) => {
      s.reviews = s.reviews.filter((r) => r.raterId !== "rater-2");
    },
    /three/u,
  ],
])
  test(`TTS rejects ${label}`, (context) => {
    const f = fixture(context);
    mutateCapture(f, (v) => mutate(v.capabilities.tts));
    assert.throws(() => evaluateFiles(f.corpus, f.capture), pattern);
  });
test("TTS scores below the registered threshold fail qualification", (context) => {
  const f = fixture(context);
  mutateCapture(f, (v) => {
    for (const review of v.capabilities.tts.reviews) review.scores.naturalness = 1;
  });
  assert.equal(evaluateFiles(f.corpus, f.capture).capabilities.tts.status, "failed");
});
test("reversed invocation times are rejected", (context) => {
  const f = fixture(context);
  mutateOutput(f, "embedding", (v) => {
    v.source.invocations[0].finishedAt = "2020-01-01T00:00:00Z";
  });
  assert.throws(() => evaluateFiles(f.corpus, f.capture), /finishes/u);
});
test("alignment partial timestamps cannot qualify", (context) => {
  const f = fixture(context);
  mutateOutput(f, "alignment", (v) => {
    Object.values(v.values.samples)[0][0].startMs = null;
  });
  assert.throws(() => evaluateFiles(f.corpus, f.capture), /boundaries/u);
});

test("evidence rejects unsafe attempt epochs and undeclared nested fields", (context) => {
  const f = fixture(context);
  mutateOutput(f, "embedding", (v) => {
    v.source.invocations[0].attemptEpoch = 1e30;
  });
  assert.throws(() => evaluateFiles(f.corpus, f.capture), /Invalid embedding-output/u);
  mutateOutput(f, "embedding", (v) => {
    v.source.invocations[0].attemptEpoch = 0;
    v.source.invocations[0].unverified = true;
  });
  assert.throws(() => evaluateFiles(f.corpus, f.capture), /Invalid embedding-output/u);
});
