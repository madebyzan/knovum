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
      agent: { state: "not_published" },
    },
    nativeInference: true,
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
  status.pythonRuntimes.agent = { state: "invalid", message: "Downloader manifest is corrupt" };
  expect(textEmbeddingReady(status)).toBe(true);
});

it("keeps background indexing unprepared when its own runtime or snapshot is unavailable", () => {
  expect(textEmbeddingReady()).toBe(false);
  const status = resources();
  status.nativeInference = false;
  expect(textEmbeddingReady(status)).toBe(false);
  status.nativeInference = true;
  status.models.textEmbeddingConfig.verified = false;
  expect(textEmbeddingReady(status)).toBe(false);
});
