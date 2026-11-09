import { defaultExecutionTarget } from "../../../foundation/ai/ai-execution";
import type { AiModelSettings } from "../../../foundation/ai/schema/model-settings";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import type { VideoTranslationProvenance } from "../contracts/generated/VideoTranslationProvenance";

export type TranslationModelTarget = Extract<TaskTarget, { type: "model" }>;

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

export function resolveInitialTarget(
  settings: AiModelSettings,
  targets: Array<{ target: TranslationModelTarget }>,
  provenance?: VideoTranslationProvenance | null,
): TranslationModelTarget | undefined {
  const configured =
    persistedTranslationTarget(settings, provenance) ?? defaultExecutionTarget(settings, "translation");
  return configured?.type === "model" &&
    targets.some(({ target }) => sameTranslationTarget(target, configured))
    ? configured
    : undefined;
}

export function parseTranslationTerms(input: string): string[] {
  return input
    .split(/[,，;；\n]+/)
    .map((term) => term.trim())
    .filter(Boolean);
}
