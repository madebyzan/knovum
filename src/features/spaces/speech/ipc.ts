import { Channel } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { safeExportFileName } from "../assets/export";
import type { ArchiveVoiceProfileRequest } from "../contracts/generated/ArchiveVoiceProfileRequest";
import type { CopyVoiceProfileRequest } from "../contracts/generated/CopyVoiceProfileRequest";
import type { CreatePresetVoiceRequest } from "../contracts/generated/CreatePresetVoiceRequest";
import type { CreateSpeechProjectRequest } from "../contracts/generated/CreateSpeechProjectRequest";
import type { DeleteVoiceProfileRequest } from "../contracts/generated/DeleteVoiceProfileRequest";
import type { GenerateSpeechProjectRequest } from "../contracts/generated/GenerateSpeechProjectRequest";
import type { PresetVoice } from "../contracts/generated/PresetVoice";
import type { PreviewVoiceRequest } from "../contracts/generated/PreviewVoiceRequest";
import type { RenameVoiceProfileRequest } from "../contracts/generated/RenameVoiceProfileRequest";
import type { ReviewSpeechSourceSeparationRequest } from "../contracts/generated/ReviewSpeechSourceSeparationRequest";
import type { ReviseVoiceProfileRequest } from "../contracts/generated/ReviseVoiceProfileRequest";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SaveSpeechProjectRevisionRequest } from "../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechAudioOutput } from "../contracts/generated/SpeechAudioOutput";
import type { SpeechGenerationProgress } from "../contracts/generated/SpeechGenerationProgress";
import type { SpeechPreviewAudioChunk } from "../contracts/generated/SpeechPreviewAudioChunk";
import type { SpeechProjectOverview } from "../contracts/generated/SpeechProjectOverview";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import type { StartSpeechSourceSeparationRequest } from "../contracts/generated/StartSpeechSourceSeparationRequest";
import type { TranslateVideoDubbingRequest } from "../contracts/generated/TranslateVideoDubbingRequest";
import type { UnarchiveVoiceProfileRequest } from "../contracts/generated/UnarchiveVoiceProfileRequest";
import type { VideoTranslation } from "../contracts/generated/VideoTranslation";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import type { VoiceProfileHistory } from "../contracts/generated/VoiceProfileHistory";

export function listVoiceRevisions(spaceId: string) {
  return invokeCommand<VoiceLibraryRevision[]>("plugin:spaces|list_voice_revisions", { spaceId });
}

export function listVoiceProfiles(spaceId: string) {
  return invokeCommand<VoiceProfileHistory[]>("plugin:spaces|list_voice_profiles", { spaceId });
}

export function renameVoiceProfile(request: RenameVoiceProfileRequest) {
  return invokeCommand<void>("plugin:spaces|rename_voice_profile", { request });
}

export function reviseVoiceProfile(request: ReviseVoiceProfileRequest) {
  return invokeCommand<void>("plugin:spaces|revise_voice_profile", { request });
}

export function archiveVoiceProfile(request: ArchiveVoiceProfileRequest) {
  return invokeCommand<void>("plugin:spaces|archive_voice_profile", { request });
}

export function unarchiveVoiceProfile(request: UnarchiveVoiceProfileRequest) {
  return invokeCommand<void>("plugin:spaces|unarchive_voice_profile", { request });
}

export function deleteVoiceProfile(request: DeleteVoiceProfileRequest) {
  return invokeCommand<void>("plugin:spaces|delete_voice_profile", { request });
}

export function copyVoiceProfile(request: CopyVoiceProfileRequest) {
  return invokeCommand<string>("plugin:spaces|copy_voice_profile", { request });
}

function progressChannel(progress: (value: SpeechGenerationProgress) => void) {
  const channel = new Channel<SpeechGenerationProgress>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channels use onmessage.
  channel.onmessage = progress;
  return channel;
}

export function previewVoice(
  request: PreviewVoiceRequest,
  progress: (value: SpeechGenerationProgress) => void,
  audioChunk: (value: SpeechPreviewAudioChunk) => void,
) {
  const onAudioChunk = new Channel<SpeechPreviewAudioChunk>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channels use onmessage.
  onAudioChunk.onmessage = audioChunk;
  return invokeCommand<SpeechAudioOutput>("plugin:spaces|preview_voice", {
    request,
    onProgress: progressChannel(progress),
    onAudioChunk,
  });
}

export function generateSpeechProject(request: GenerateSpeechProjectRequest) {
  return invokeCommand<RunObservation>("plugin:spaces|generate_speech_project", { request });
}

export function cancelSpeechGeneration(jobId: string) {
  return invokeCommand<boolean>("plugin:spaces|cancel_speech_generation", { jobId });
}

export function discardSpeechPreview(path: string) {
  return invokeCommand<void>("plugin:spaces|discard_speech_preview", { path });
}

export async function exportSpeechRender(sourcePath: string, projectTitle: string) {
  const base = safeExportFileName(projectTitle) || "speech";
  const extension = sourcePath.split(".").at(-1)?.toLowerCase();
  if (!extension || !["wav", "mp3", "m4b", "mp4"].includes(extension))
    throw new Error("Unsupported speech export format");
  const destinationPath = await save({
    defaultPath: `${base}.${extension}`,
    filters: [{ name: extension.toUpperCase(), extensions: [extension] }],
  });
  if (!destinationPath) return false;
  await invokeCommand("plugin:spaces|export_asset", { sourcePath, destinationPath });
  return true;
}

export function getSpeechProject(projectId: string) {
  return invokeCommand<SpeechProjectSnapshot>("plugin:spaces|get_speech_project", { projectId });
}

export function saveSpeechProjectRevision(request: SaveSpeechProjectRevisionRequest) {
  return invokeCommand<SpeechProjectSnapshot>("plugin:spaces|save_speech_project_revision", { request });
}

export function translateVideoDubbing(request: TranslateVideoDubbingRequest) {
  return invokeCommand<RunObservation>("plugin:spaces|translate_video_dubbing", { request });
}

export function getVideoTranslationRun(projectId: string, expectedRevision: number) {
  return invokeCommand<RunObservation | null>("plugin:spaces|get_video_translation_run", {
    projectId,
    expectedRevision,
  });
}

export function getVideoTranslation(runId: string) {
  return invokeCommand<VideoTranslation>("plugin:spaces|get_video_translation", { runId });
}

export function startSpeechSourceSeparation(request: StartSpeechSourceSeparationRequest) {
  return invokeCommand<RunObservation>("plugin:spaces|start_speech_source_separation", { request });
}

export function getSpeechSourceSeparation(separationId: string) {
  return invokeCommand<SpeechSourceSeparationCandidate>("plugin:spaces|get_speech_source_separation", {
    separationId,
  });
}

export function reviewSpeechSourceSeparation(request: ReviewSpeechSourceSeparationRequest) {
  return invokeCommand<SpeechSourceSeparationCandidate>("plugin:spaces|review_speech_source_separation", {
    request,
  });
}

export function createPresetVoice(request: CreatePresetVoiceRequest) {
  return invokeCommand<VoiceLibraryRevision>("plugin:spaces|create_preset_voice", { request });
}

export function listSpeechProjects(spaceId: string, sourceNodeId: string) {
  return invokeCommand<SpeechProjectOverview[]>("plugin:spaces|list_speech_projects", {
    spaceId,
    sourceNodeId,
  });
}

export function createSpeechProject(request: CreateSpeechProjectRequest) {
  return invokeCommand<SpeechProjectOverview>("plugin:spaces|create_speech_project", { request });
}

export function listPresetVoices() {
  return invokeCommand<PresetVoice[]>("plugin:spaces|list_preset_voices");
}
