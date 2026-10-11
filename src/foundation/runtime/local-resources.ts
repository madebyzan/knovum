import { Channel } from "@tauri-apps/api/core";

import type { LocalModelAvailability } from "../../shared/contracts/generated/LocalModelAvailability";
import type { LocalModelId } from "../../shared/contracts/generated/LocalModelId";
import type { LocalResourcesStatus } from "../../shared/contracts/generated/LocalResourcesStatus";
import type { ResourcePreparationProgress } from "../../shared/contracts/generated/ResourcePreparationProgress";
import type { ResourcePreparationSnapshot } from "../../shared/contracts/generated/ResourcePreparationSnapshot";
import type { ResourcePreparationTarget } from "../../shared/contracts/generated/ResourcePreparationTarget";
import { invokeCommand } from "../ipc/invoke-command";

export function getLocalResourcesStatus(): Promise<LocalResourcesStatus> {
  return invokeCommand("plugin:local-resources|get_status");
}

interface ResourcePreparationInput {
  target: ResourcePreparationTarget;
  jobId: string;
  title: string;
  onProgress: (value: ResourcePreparationProgress) => void;
}

export function prepareResources({ target, jobId, title, onProgress }: ResourcePreparationInput) {
  const channel = new Channel<ResourcePreparationProgress>();
  // Tauri Channel exposes one IPC receiver rather than a DOM EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = onProgress;
  return invokeCommand<void>("plugin:local-resources|prepare_resources", {
    target,
    jobId,
    title,
    onProgress: channel,
  });
}

export function cancelResourcePreparation(jobId: string): Promise<void> {
  return invokeCommand("plugin:local-resources|cancel_resource_preparation", { jobId });
}
export function listResourcePreparations(): Promise<ResourcePreparationSnapshot[]> {
  return invokeCommand("plugin:local-resources|list_resource_preparations");
}
export function textEmbeddingReady(status?: LocalResourcesStatus) {
  if (status?.nativeInference !== true) return false;
  const { models } = status;
  return [
    models.textEmbeddingConfig,
    models.textEmbeddingModules,
    models.textEmbeddingPoolingConfig,
    models.textEmbeddingSentenceTransformersConfig,
    models.textEmbeddingTokenizer,
    models.textEmbeddingTokenizerConfig,
    models.textEmbeddingWeights,
  ].every((model) => model.verified);
}

export function getLocalModelCatalog(): Promise<LocalModelAvailability[]> {
  return invokeCommand("plugin:local-resources|get_model_catalog");
}
export function removeLocalModel(model: LocalModelId): Promise<void> {
  return invokeCommand("plugin:local-resources|remove_model", { model });
}

export function selectLocalModel(model: LocalModelId): Promise<void> {
  return invokeCommand("plugin:local-resources|select_local_model", { model });
}
