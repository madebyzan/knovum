import type { ModelKind } from "../../shared/contracts/generated/ModelKind";
export type { ModelKind } from "../../shared/contracts/generated/ModelKind";

import type { LocalResourcesStatus } from "../../shared/contracts/generated/LocalResourcesStatus";
export type { LocalResourcesStatus } from "../../shared/contracts/generated/LocalResourcesStatus";
import type { ModelDownloadProgress as ModelInstallProgress } from "../../shared/contracts/generated/ModelDownloadProgress";
export type { ModelDownloadProgress as ModelInstallProgress } from "../../shared/contracts/generated/ModelDownloadProgress";
import type { ModelInstallSnapshot } from "../../shared/contracts/generated/ModelInstallSnapshot";
export type { ModelInstallSnapshot } from "../../shared/contracts/generated/ModelInstallSnapshot";
import { Channel } from "@tauri-apps/api/core";

import { invokeCommand as invoke } from "../ipc/invoke-command";

export function getLocalResourcesStatus(): Promise<LocalResourcesStatus> {
  return invoke("plugin:local-resources|get_status");
}
export function installLocalModels(
  models: ModelKind[],
  jobId: string,
  onProgress: (value: ModelInstallProgress) => void,
) {
  const channel = new Channel<ModelInstallProgress>();
  // Tauri Channel exposes one IPC receiver rather than a DOM EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = onProgress;
  return invoke<void>("plugin:local-resources|install_models", { models, jobId, onProgress: channel });
}
export function cancelModelInstall(jobId: string): Promise<void> {
  return invoke("plugin:local-resources|cancel_model_install", { jobId });
}
export function listModelInstalls(): Promise<ModelInstallSnapshot[]> {
  return invoke("plugin:local-resources|list_model_installs");
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
