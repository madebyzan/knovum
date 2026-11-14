import type { LocalModelAvailability } from "../../shared/contracts/generated/LocalModelAvailability";
import type { LocalModelCapability } from "../../shared/contracts/generated/LocalModelCapability";
import type { LocalModelId } from "../../shared/contracts/generated/LocalModelId";
import { formatBytes } from "../format/bytes";
import { queryClient } from "../query/query-client";
import { localModelsQueryOptions } from "../runtime/use-local-models-query";
import { preferredChoiceId, type ModelChoiceOption } from "./model-choice";

/**
 * The models that serve a capability. Only a verified download can run now; the Settings default
 * is the one the chooser preselects.
 */
export function localModelChoiceOptions(
  models: readonly LocalModelAvailability[],
  capability: LocalModelCapability,
): ModelChoiceOption[] {
  return models
    .filter((model) => model.capability === capability)
    .map((model) => ({
      id: model.id,
      name: model.name,
      description: formatBytes(model.totalBytes),
      runnable: model.verified,
      preferred: model.selected,
    }));
}

/** The Settings preference for a capability among these models, or the first when none is preferred. */
export function preferredLocalModelId(
  models: readonly LocalModelAvailability[],
  capability: LocalModelCapability,
): LocalModelId | undefined {
  return preferredChoiceId(localModelChoiceOptions(models, capability)) as LocalModelId | undefined;
}

/** The catalog's preferred model for a capability, read fresh so a new request freezes today's choice. */
export async function preferredLocalModel(
  capability: LocalModelCapability,
): Promise<LocalModelId | undefined> {
  const models = await queryClient.fetchQuery({ ...localModelsQueryOptions, staleTime: 0, retry: false });
  return preferredLocalModelId(models, capability);
}
