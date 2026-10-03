import { Channel } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { safeExportFileName } from "../assets/export";
import type {
  CreatePresetVoiceInput,
  CreateSpeechProjectInput,
  SpeechProjectSummary,
  SpeechProjectDetail,
  SaveSpeechProjectRevisionInput,
  VoiceProfileSummary,
  SpeechGenerationProgress,
  SpeechAudioOutput,
  SpeechProjectRenderOutput,
} from "./contracts";

export function listVoiceProfiles(spaceId: string) {
  return invokeCommand<VoiceProfileSummary[]>("plugin:spaces|list_voice_profiles", { spaceId });
}

function progressChannel(progress: (value: SpeechGenerationProgress) => void) {
  const channel = new Channel<SpeechGenerationProgress>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channels use onmessage.
  channel.onmessage = progress;
  return channel;
}

export function previewPresetVoice(
  request: { jobId: string; spaceId: string; voiceRevisionId: string; text: string },
  progress: (value: SpeechGenerationProgress) => void,
) {
  return invokeCommand<SpeechAudioOutput>("plugin:spaces|preview_preset_voice", {
    request,
    onProgress: progressChannel(progress),
  });
}

export function generateSpeechProject(
  request: { jobId: string; projectId: string; expectedRevision: number },
  progress: (value: SpeechGenerationProgress) => void,
) {
  return invokeCommand<SpeechProjectRenderOutput>("plugin:spaces|generate_speech_project", {
    request,
    onProgress: progressChannel(progress),
  });
}

export function cancelSpeechGeneration(jobId: string) {
  return invokeCommand<boolean>("plugin:spaces|cancel_speech_generation", { jobId });
}

export function discardSpeechPreview(path: string) {
  return invokeCommand<void>("plugin:spaces|discard_speech_preview", { path });
}

export async function exportSpeechRender(sourcePath: string, projectTitle: string) {
  const base = safeExportFileName(projectTitle) || "speech";
  const destinationPath = await save({
    defaultPath: `${base}.wav`,
    filters: [{ name: "Wave audio", extensions: ["wav"] }],
  });
  if (!destinationPath) return false;
  await invokeCommand("plugin:spaces|export_asset", { sourcePath, destinationPath });
  return true;
}

export function getSpeechProject(projectId: string) {
  return invokeCommand<SpeechProjectDetail>("plugin:spaces|get_speech_project", { projectId });
}

export function saveSpeechProjectRevision(request: SaveSpeechProjectRevisionInput) {
  return invokeCommand<SpeechProjectDetail>("plugin:spaces|save_speech_project_revision", { request });
}

export function createPresetVoice(request: CreatePresetVoiceInput) {
  return invokeCommand<VoiceProfileSummary>("plugin:spaces|create_preset_voice", { request });
}

export function listSpeechProjects(spaceId: string) {
  return invokeCommand<SpeechProjectSummary[]>("plugin:spaces|list_speech_projects", { spaceId });
}

export function createSpeechProject(request: CreateSpeechProjectInput) {
  return invokeCommand<SpeechProjectSummary>("plugin:spaces|create_speech_project", { request });
}
