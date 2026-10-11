import { distance } from "fastest-levenshtein";

import { EvidenceError, execution, hashedFile, sameIds } from "./evidence.mjs";

export function tokens(text, language) {
  const normalized = text.normalize("NFKC").toLowerCase();
  return language === "zh-CN"
    ? [...normalized].filter((c) => /[\p{L}\p{N}]/u.test(c))
    : (normalized.match(/[\p{L}\p{N}]+(?:'[\p{L}\p{N}]+)?/gu) ?? []);
}
export function tokenDistance(left, right) {
  const ids = new Map();
  const encode = (values) =>
    values
      .map((value) => {
        if (!ids.has(value)) ids.set(value, ids.size + 1);
        if (ids.size > 0xd7ff) throw new EvidenceError("Transcript exceeds the scoring vocabulary limit");
        return String.fromCharCode(ids.get(value));
      })
      .join("");
  return distance(encode(left), encode(right));
}
export function evaluateTranscription(corpus, section, base, provenance) {
  const profile = corpus.baselineProfile,
    params = provenance.configs.asr.parameters;
  if (params.modelFamily !== profile.modelFamily || params.modelSize !== profile.modelSize)
    throw new EvidenceError("ASR differs from the registered model family and size");
  const baseline = execution(section.baselineOutput, base, "asr", {
    provenance,
    kind: "product_baseline",
    profile,
  });
  const candidate = execution(section.candidateOutput, base, "asr", { provenance });
  const ids = corpus.samples.map((item) => item.id);
  sameIds(Object.keys(baseline.transcripts), ids);
  sameIds(Object.keys(candidate.transcripts), ids);
  const metrics = {};
  for (const [language, metric] of [
    ["zh-CN", "cer"],
    ["en-US", "wer"],
  ]) {
    let units = 0,
      before = 0,
      after = 0;
    for (const sample of corpus.samples.filter((item) => item.language === language)) {
      const expected = tokens(sample.referenceText, language);
      units += expected.length;
      before += tokenDistance(expected, tokens(baseline.transcripts[sample.id], language));
      after += tokenDistance(expected, tokens(candidate.transcripts[sample.id], language));
    }
    if (!units) throw new EvidenceError("ASR corpus has no scored units");
    metrics[metric] = {
      baseline: before / units,
      candidate: after / units,
      regression: (after - before) / units,
      maximumRegression: 0.01,
    };
  }
  return {
    status: Object.values(metrics).every((item) => item.regression <= 0.01) ? "passed" : "failed",
    ...metrics,
  };
}

export function percentile(values, p) {
  if (!values.length) return null;
  const sorted = values.toSorted((a, b) => a - b),
    position = (sorted.length - 1) * p,
    lower = Math.floor(position);
  return sorted[lower] + (sorted[Math.ceil(position)] - sorted[lower]) * (position - lower);
}
const alignmentMetrics = (aligned, total, errors) => ({
  coverage: aligned / total,
  medianBoundaryErrorMs: percentile(errors, 0.5),
  p95BoundaryErrorMs: percentile(errors, 0.95),
});
export function evaluateAlignment(corpus, section, base, provenance) {
  const values = execution(section.candidateOutput, base, "alignment", { provenance });
  sameIds(
    Object.keys(values.samples),
    corpus.samples.map((s) => s.id),
  );
  const byLanguage = {},
    allErrors = [];
  let allAligned = 0,
    allTotal = 0;
  for (const language of ["zh-CN", "en-US"]) {
    let aligned = 0,
      total = 0;
    const errors = [];
    for (const sample of corpus.samples.filter((s) => s.language === language)) {
      const actual = values.samples[sample.id];
      sameIds(
        actual.map((u) => u.id),
        sample.units.map((u) => u.id),
      );
      total += sample.units.length;
      for (const expected of sample.units) {
        const unit = actual.find((u) => u.id === expected.id);
        if (
          (unit.startMs === null) !== (unit.endMs === null) ||
          (unit.startMs !== null && unit.endMs < unit.startMs)
        )
          throw new EvidenceError("Invalid alignment boundaries");
        if (unit.startMs !== null) {
          aligned++;
          errors.push(Math.abs(unit.startMs - expected.startMs), Math.abs(unit.endMs - expected.endMs));
        }
      }
    }
    if (!total) throw new EvidenceError("Alignment requires annotations for each language");
    byLanguage[language] = alignmentMetrics(aligned, total, errors);
    allErrors.push(...errors);
    allAligned += aligned;
    allTotal += total;
  }
  const overall = alignmentMetrics(allAligned, allTotal, allErrors),
    thresholds = corpus.thresholds;
  const passed = [overall, ...Object.values(byLanguage)].every(
    (m) =>
      m.medianBoundaryErrorMs !== null &&
      m.p95BoundaryErrorMs !== null &&
      m.coverage >= thresholds.minimumCoverage &&
      m.medianBoundaryErrorMs <= thresholds.maximumMedianBoundaryErrorMs &&
      m.p95BoundaryErrorMs <= thresholds.maximumP95BoundaryErrorMs,
  );
  return { status: passed ? "passed" : "failed", thresholds, overall, byLanguage };
}

export function evaluateSynthesis(corpus, section, base, provenance) {
  const values = execution(section.candidateOutput, base, "tts", { provenance });
  const ids = corpus.samples.map((s) => s.id);
  sameIds(
    values.artifacts.map((s) => s.sampleId),
    ids,
  );
  for (const item of values.artifacts) hashedFile(item.audio, base);
  hashedFile(section.blinding.assignment, base);
  const raters = new Map(ids.map((id) => [id, new Set()])),
    scores = {};
  for (const review of section.reviews) {
    if (Date.parse(review.reviewedAt) >= Date.parse(section.blinding.revealedAfterReviewsAt))
      throw new EvidenceError("TTS review was recorded after unblinding");
    const seen = raters.get(review.sampleId);
    if (!seen || seen.has(review.raterId)) throw new EvidenceError("Unknown or duplicate TTS review");
    seen.add(review.raterId);
    for (const [criterion, score] of Object.entries(review.scores)) (scores[criterion] ??= []).push(score);
  }
  const minimumRatersPerSample = Math.min(...[...raters.values()].map((r) => r.size));
  if (minimumRatersPerSample < 3)
    throw new EvidenceError("Each TTS sample requires three independent blind reviews");
  const meanScores = Object.fromEntries(
    Object.entries(scores).map(([key, ratings]) => [
      key,
      ratings.reduce((a, b) => a + b, 0) / ratings.length,
    ]),
  );
  const thresholds = corpus.thresholds;
  return {
    status: Object.entries(thresholds).every(([key, value]) => meanScores[key] >= value)
      ? "passed"
      : "failed",
    sampleCount: ids.length,
    raterCount: new Set(section.reviews.map((review) => review.raterId)).size,
    minimumRatersPerSample,
    meanScores,
    thresholds,
    blinding: section.blinding,
  };
}
