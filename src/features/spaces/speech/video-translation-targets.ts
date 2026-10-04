import type { AiModelSettings } from "../../../foundation/ai/schema/model-settings";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { VideoTranslationProvenance } from "../contracts/generated/VideoTranslationProvenance";

export type TranslationModelTarget = Extract<TaskTargetRef, { type: "model" }>;

export function sameTranslationTarget(left: TranslationModelTarget, right: TranslationModelTarget) {
  return left.connectionId === right.connectionId && left.modelId === right.modelId;
}

export function isLocalTranslationTarget(settings: AiModelSettings, candidate: TranslationModelTarget) {
  const connection = settings.connections.find(({ id }) => id === candidate.connectionId);
  if (connection?.provider !== "ollama") return false;
  try {
    const host = new URL(connection.baseUrl).hostname;
    return host === "localhost" || host === "127.0.0.1" || host === "[::1]" || host === "::1";
  } catch {
    return false;
  }
}

export function persistedTranslationTarget(
  settings: AiModelSettings,
  provenance: VideoTranslationProvenance | null | undefined,
): TranslationModelTarget | undefined {
  if (!provenance) return;
  for (const connection of settings.connections) {
    if (connection.provider !== provenance.provider) continue;
    const matchedModel = connection.models.find(({ model: modelName }) => modelName === provenance.model);
    if (!matchedModel) continue;
    const candidate: TranslationModelTarget = {
      type: "model",
      connectionId: connection.id,
      modelId: matchedModel.id,
    };
    if (!provenance.localOnly || isLocalTranslationTarget(settings, candidate)) return candidate;
  }
}
