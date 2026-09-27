import { z } from "zod";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";

const recordingSessionSchema = z.object({ sessionId: z.uuid() }).strict();
const recordedAssetSchema = z
  .object({
    name: z.string().trim().min(1).max(512),
    path: z.string().min(1).max(4096),
    kind: z.literal("audio"),
    size: z.number().int().positive().max(Number.MAX_SAFE_INTEGER),
    modifiedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    sourceRevision: z.string().regex(/^sha256:[a-f0-9]{64}$/u),
  })
  .strict();

export type PersistedAudioSpaceAsset = z.infer<typeof recordedAssetSchema>;

const RECORDING_MIME_TYPES = ["audio/webm;codecs=opus", "audio/mp4", "audio/webm"] as const;

export interface SpaceAudioRecordingSession {
  id: string;
  mimeType: string;
}

export async function startSpaceAudioRecording(mimeType: string): Promise<SpaceAudioRecordingSession> {
  const stamp = new Date().toISOString().replaceAll(/[:.]/gu, "-");
  const recordingResponse = await invoke<unknown>("plugin:spaces|start_audio_recording", {
    mimeType,
    displayName: `Audio ${stamp}`,
  });
  const session = recordingSessionSchema.parse(recordingResponse);
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
  const recordingResponse = await invoke<unknown>("plugin:spaces|finalize_audio_recording", {
    sessionId: session.id,
  });
  return recordedAssetSchema.parse(recordingResponse);
}

export function cancelSpaceAudioRecording(session: SpaceAudioRecordingSession): Promise<void> {
  return invoke("plugin:spaces|cancel_audio_recording", { sessionId: session.id });
}

export function preferredAudioRecordingMimeType(): string | undefined {
  return RECORDING_MIME_TYPES.find((mimeType) => MediaRecorder.isTypeSupported(mimeType));
}
