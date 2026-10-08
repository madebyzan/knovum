import { expect, it } from "vitest";

import type { LocalResourcesStatus } from "../../../shared/contracts/generated/LocalResourcesStatus";
import { textEmbeddingReady } from "../local-resources";

function resources(): LocalResourcesStatus {
  const model = { installed: true, verified: true, path: "/model", size: 1, downloadSize: 1 };
  const tool = { available: false, path: null };
  return {
    platform: "macos-x64",
    ffmpeg: tool,
    ytDlp: tool,
    deno: tool,
    pythonRuntimes: {
      resources: { state: "not_published" },
      "light-inference": { state: "not_published" },
      "torch-inference": { state: "ready" },
      "voice-generation": { state: "not_published" },
      agent: { state: "not_published" },
    },
    models: {
      transcription: [],
      imageSegmentation: model,
      voxcpm2: [],
      speechAlignment: [],
      speechSeparation: [],
      textEmbeddingConfig: { ...model },
      textEmbeddingModules: { ...model },
      textEmbeddingPoolingConfig: { ...model },
      textEmbeddingSentenceTransformersConfig: { ...model },
      textEmbeddingTokenizer: { ...model },
      textEmbeddingTokenizerConfig: { ...model },
      textEmbeddingWeights: { ...model },
      diarization: [],
    },
  };
}

it("indexes verified embeddings without unrelated or downloader runtime publications", () => {
  const status = resources();
  expect(textEmbeddingReady(status)).toBe(true);
  status.pythonRuntimes.resources = { state: "invalid", message: "Downloader manifest is corrupt" };
  expect(textEmbeddingReady(status)).toBe(true);
});

it("keeps background indexing unprepared when its own runtime or snapshot is unavailable", () => {
  expect(textEmbeddingReady()).toBe(false);
  const status = resources();
  status.pythonRuntimes["torch-inference"] = { state: "not_installed", downloadSize: 1 };
  expect(textEmbeddingReady(status)).toBe(false);
  status.pythonRuntimes["torch-inference"] = { state: "ready" };
  status.models.textEmbeddingConfig.verified = false;
  expect(textEmbeddingReady(status)).toBe(false);
});
