import { validateImageCorpus } from "./corpus.mjs";
import {
  EvidenceError,
  fileDigest,
  hashedFile,
  sameIds,
  status,
  unique,
  validateInvocations,
} from "./evidence.mjs";
import { tensor, shape, outputContract } from "./image-tensors.mjs";

export function evaluateImages(corpus, capture, { corpusPath, base, corpusBase }) {
  validateImageCorpus(corpus);
  if (capture.corpusSha256 !== fileDigest(corpusPath))
    throw new EvidenceError("Image capture differs from corpus bytes");
  const inputs = loadInputs(corpus, corpusBase);
  unique(capture.models.map((m) => m.profileId));
  const models = {};
  for (const model of capture.models) {
    const [id, result] = evaluateModel(model, { corpus, capture, inputs, base });
    models[id] = result;
  }
  for (const profile of corpus.profiles)
    models[profile.id] ??= { status: "unavailable", reason: "no native model capture" };
  return {
    schemaVersion: 1,
    status: status(Object.values(models)),
    suite: "images",
    runId: capture.runId,
    target: capture.target,
    corpusSha256: capture.corpusSha256,
    runtimeDigest: capture.runtimeDigest,
    workerDigest: capture.workerDigest,
    device: capture.device,
    models,
  };
}

function loadInputs(corpus, corpusBase) {
  const inputs = new Map();
  for (const sample of corpus.samples) {
    hashedFile(sample.original, corpusBase);
    const rgb = tensor(sample.renderedRgb, corpusBase),
      mask = tensor(sample.selectionMask, corpusBase);
    shape(rgb, [1, 3, sample.height, sample.width], "u1");
    shape(mask, [1, 1, sample.height, sample.width], "u1");
    if (!mask.data.some((v) => v === 255) || mask.data.some((v) => v !== 0 && v !== 255))
      throw new EvidenceError("Image corpus requires a nonempty binary selection");
    inputs.set(sample.id, { sample, rgb, mask });
  }
  return inputs;
}

function evaluateModel(model, { corpus, capture, inputs, base }) {
  const profile = corpus.profiles.find((p) => p.id === model.profileId),
    source = model.source;
  if (!profile) throw new EvidenceError("Unknown image model profile");
  validateInvocations(source);
  validateModelIdentity(source, profile, capture);
  sameIds(
    model.artifacts.map((a) => a.sampleId),
    profile.sampleIds,
  );
  const limits = profile.limits,
    failures = [];
  if (model.loadSeconds > limits.maximumLoadSeconds) failures.push("cold load exceeds its profile");
  if (model.peakRssBytes > limits.maximumPeakRssBytes) failures.push("peak RSS exceeds its profile");
  for (const artifact of model.artifacts) {
    const { sample, rgb, mask } = inputs.get(artifact.sampleId);
    outputContract(profile, { sample, rgb, mask }, tensor(artifact.output, base));
    if (artifact.warmSeconds > limits.maximumWarmSeconds)
      failures.push(`${sample.id}: warm inference exceeds its profile`);
    if (artifact.cancelSeconds > limits.maximumCancelSeconds)
      failures.push(`${sample.id}: cancellation exceeds its profile`);
    if (!artifact.qualityPassed) failures.push(`${sample.id}: visual quality review failed`);
  }
  return [
    profile.id,
    {
      status: failures.length ? "failed" : source.kind === "bundled_worker" ? "passed" : "unavailable",
      failures,
      executionKind: source.kind,
      loadSeconds: model.loadSeconds,
      peakRssBytes: model.peakRssBytes,
      artifacts: model.artifacts,
    },
  ];
}

function validateModelIdentity(source, profile, capture) {
  if (
    source.runtimeDigest !== capture.runtimeDigest ||
    source.modelSpecId !== profile.model.specId ||
    source.configId !== profile.config.id ||
    source.method !== `image.${profile.capability}` ||
    (source.kind === "bundled_worker" && source.workerDigest !== capture.workerDigest)
  )
    throw new EvidenceError("Image execution differs from its frozen identity");
}
