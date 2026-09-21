import { stopAudioCapture } from "./audio-permission";
import {
  cancelSpaceAudioRecording,
  preferredAudioRecordingMimeType,
  type SpaceAudioRecordingSession,
} from "./audio-recording";

export function createAudioMediaRecorder(stream: MediaStream) {
  const mime = preferredAudioRecordingMimeType();
  return new MediaRecorder(stream, mime ? { mimeType: mime } : undefined);
}

export async function cancelPendingAudioRecording(stream: MediaStream, session: SpaceAudioRecordingSession) {
  stopAudioCapture(stream);
  await cancelSpaceAudioRecording(session);
}

export async function stopMediaRecorder(recorder: MediaRecorder) {
  const stopped = new Promise<void>((resolve) =>
    recorder.addEventListener("stop", () => resolve(), { once: true }),
  );
  recorder.stop();
  await stopped;
}
