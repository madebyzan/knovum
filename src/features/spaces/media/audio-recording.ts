import { invoke } from "@tauri-apps/api/core";

import type { ImportedSpaceAsset } from "../assets/imported-asset";

export type PersistedAudioSpaceAsset = ImportedSpaceAsset & { path: string };

const RECORDING_MIME_TYPES = ["audio/webm;codecs=opus", "audio/mp4", "audio/webm"] as const;

interface AudioRecordingStartAck {
  sessionId: string;
}

interface PersistedAudioRecording {
  path: string;
  extension: string;
}

export interface SpaceAudioRecordingSession {
  id: string;
  mimeType: string;
}

export async function startSpaceAudioRecording(mimeType: string): Promise<SpaceAudioRecordingSession> {
  const session = await invoke<AudioRecordingStartAck>("plugin:spaces|start_audio_recording", { mimeType });
  return { id: session.sessionId, mimeType };
}

export async function appendSpaceAudioRecording(
  session: SpaceAudioRecordingSession,
  blob: Blob,
): Promise<void> {
  if (!blob.size) return;
  await invoke("plugin:spaces|append_audio_chunk", {
    sessionId: session.id,
    bytes: Array.from(new Uint8Array(await blob.arrayBuffer())),
  });
}

export async function finishSpaceAudioRecording(
  session: SpaceAudioRecordingSession,
): Promise<PersistedAudioSpaceAsset> {
  const recording = await invoke<PersistedAudioRecording>("plugin:spaces|finalize_audio_recording", {
    sessionId: session.id,
  });
  const stamp = new Date().toISOString().replaceAll(/[:.]/gu, "-");
  return { name: `Audio ${stamp}.${recording.extension}`, path: recording.path, kind: "audio" };
}

export function cancelSpaceAudioRecording(session: SpaceAudioRecordingSession): Promise<void> {
  return invoke("plugin:spaces|cancel_audio_recording", { sessionId: session.id });
}

export function preferredAudioRecordingMimeType(): string | undefined {
  return RECORDING_MIME_TYPES.find((mimeType) => MediaRecorder.isTypeSupported(mimeType));
}
