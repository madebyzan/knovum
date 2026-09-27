import type { ModelKind } from "../../shared/contracts/generated/ModelKind";
export type { ModelKind } from "../../shared/contracts/generated/ModelKind";

import type { LocalResourcesStatus } from "../../shared/contracts/generated/LocalResourcesStatus";
export type { LocalResourcesStatus } from "../../shared/contracts/generated/LocalResourcesStatus";
import type { ResourcePreparationProgress } from "../../shared/contracts/generated/ResourcePreparationProgress";
export type { ResourcePreparationProgress } from "../../shared/contracts/generated/ResourcePreparationProgress";
import type { ResourcePreparationSnapshot } from "../../shared/contracts/generated/ResourcePreparationSnapshot";
export type { ResourcePreparationSnapshot } from "../../shared/contracts/generated/ResourcePreparationSnapshot";
import type { ResourcePreparationTarget } from "../../shared/contracts/generated/ResourcePreparationTarget";
export type { ResourcePreparationTarget } from "../../shared/contracts/generated/ResourcePreparationTarget";
import { Channel } from "@tauri-apps/api/core";

import { invokeCommand as invoke } from "../ipc/invoke-command";

export function getLocalResourcesStatus(): Promise<LocalResourcesStatus> {
  return invoke("plugin:local-resources|get_status");
}
export function prepareResources(
  target: ResourcePreparationTarget,
  jobId: string,
  title: string,
  onProgress: (value: ResourcePreparationProgress) => void,
) {
  const channel = new Channel<ResourcePreparationProgress>();
  // Tauri Channel exposes one IPC receiver rather than a DOM EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = onProgress;
  return invoke<void>("plugin:local-resources|prepare_resources", {
    target,
    jobId,
    title,
    onProgress: channel,
  });
}
export function cancelResourcePreparation(jobId: string): Promise<void> {
  return invoke("plugin:local-resources|cancel_resource_preparation", { jobId });
}
export function listResourcePreparations(): Promise<ResourcePreparationSnapshot[]> {
  return invoke("plugin:local-resources|list_resource_preparations");
}
export function removeLocalModel(model: ModelKind): Promise<void> {
  return invoke("plugin:local-resources|remove_model", { model });
}
export function transcriptionReady(status?: LocalResourcesStatus) {
  return Boolean(
    status?.ffmpeg.available &&
    status.whisperCli.available &&
    status.models.transcription.verified &&
    status.models.transcriptionVad.verified,
  );
}
export function textEmbeddingReady(status?: LocalResourcesStatus) {
  return Boolean(
    status?.models.textEmbeddingConfig.verified &&
    status.models.textEmbeddingTokenizer.verified &&
    status.models.textEmbeddingWeights.verified,
  );
}
