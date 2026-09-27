import { Channel } from "@tauri-apps/api/core";

import { decodeCommandError } from "../../../foundation/ipc/command-error";
import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { ResourcePreparationProgress } from "../../../shared/contracts/generated/ResourcePreparationProgress";
import type { SpaceMediaTranscript } from "./knowledge-schema";

export type SpaceMediaTranscriptionStage =
  | "preparingResources"
  | "preparing"
  | "transcoding"
  | "transcribing"
  | "parsing"
  | "done";

export interface SpaceMediaTranscriptionProgress {
  stage: SpaceMediaTranscriptionStage;
  progress: number;
  resources?: ResourcePreparationProgress;
  partial?: {
    language: string;
    segments: SpaceMediaTranscript["segments"];
  };
}

export const MEDIA_TRANSCRIPTION_CANCELLED = "media_transcription_cancelled";

export async function transcribeSpaceMedia(
  jobId: string,
  sourcePath: string,
  sourceRevision: string,
  onProgress: (progress: SpaceMediaTranscriptionProgress) => void,
): Promise<SpaceMediaTranscript> {
  const channel = new Channel<SpaceMediaTranscriptionProgress>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes progress through onmessage, not EventTarget.
  channel.onmessage = onProgress;
  return invoke<SpaceMediaTranscript>("plugin:spaces|transcribe_media", {
    jobId,
    sourcePath,
    sourceRevision,
    onProgress: channel,
  });
}

export function cancelSpaceMediaTranscription(jobId: string) {
  return invoke<void>("plugin:spaces|cancel_media_transcription", { jobId });
}

export function isMediaTranscriptionCancellation(error: unknown) {
  return decodeCommandError(error)?.code === MEDIA_TRANSCRIPTION_CANCELLED;
}
