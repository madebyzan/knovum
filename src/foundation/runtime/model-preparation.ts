import type { ModelKind } from "./local-resources";
import { startResourcePreparation, type ResourcePreparationHandle } from "./resource-preparation";

export type ModelCapability = "transcription" | "textEmbedding";

const MODEL_CAPABILITIES: Record<ModelCapability, readonly ModelKind[]> = {
  transcription: ["transcription", "transcriptionVad"],
  textEmbedding: ["textEmbeddingConfig", "textEmbeddingTokenizer", "textEmbeddingWeights"],
};

export function modelKinds(capability: ModelCapability): ModelKind[] {
  return [...MODEL_CAPABILITIES[capability]];
}

export function prepareModelCapability(capability: ModelCapability, title: string): Promise<void> {
  return startModelPreparation(capability, title).completion;
}

export function startModelPreparation(
  capability: ModelCapability,
  title: string,
  onProgress?: Parameters<typeof startResourcePreparation>[2],
): ResourcePreparationHandle {
  return startResourcePreparation({ kind: capability }, title, onProgress);
}
