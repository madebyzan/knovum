import { Channel, invoke } from "@tauri-apps/api/core";

export type ModelKind =
  | "transcription"
  | "transcriptionVad"
  | "textEmbeddingConfig"
  | "textEmbeddingTokenizer"
  | "textEmbeddingWeights";

interface ToolStatus {
  available: boolean;
  path: string | null;
}
interface LocalModelStatus {
  installed: boolean;
  verified: boolean;
  path: string;
  size?: number;
  downloadSize: number;
}
export interface LocalResourcesStatus {
  platform: string;
  ffmpeg: ToolStatus;
  ytDlp: ToolStatus;
  deno: ToolStatus;
  whisperCli: ToolStatus;
  models: Record<ModelKind, LocalModelStatus>;
}
export interface ModelInstallProgress {
  downloaded: number;
  total?: number;
  percent?: number;
}
export interface ModelInstallSnapshot {
  jobId: string;
  models: ModelKind[];
  startedAt: number;
}

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
