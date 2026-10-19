import type { AiModelSettings } from "../../../foundation/ai/schema/model-settings";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { VideoTranslationProvenance } from "../contracts/generated/VideoTranslationProvenance";

export type TranslationModelTarget = Extract<TaskTargetRef, { type: "model" }>;

export function sameTranslationTarget(left: TranslationModelTarget, right: TranslationModelTarget) {
  return left.connectionId === right.connectionId && left.modelId === right.modelId;
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
    return { type: "model", connectionId: connection.id, modelId: matchedModel.id };
  }
}
