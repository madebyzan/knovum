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
  if (recorder.state === "inactive") return;
  let onStop!: () => void;
  const stopped = new Promise<void>((resolve) => {
    onStop = resolve;
  });
  recorder.addEventListener("stop", onStop, { once: true });
  try {
    recorder.stop();
    await stopped;
  } finally {
    recorder.removeEventListener("stop", onStop);
  }
}

export async function settleAudioRecording(recorder: MediaRecorder | undefined, settle: () => Promise<void>) {
  const failures: unknown[] = [];
  try {
    if (recorder) await stopMediaRecorder(recorder);
  } catch (error) {
    failures.push(error);
  }
  try {
    await settle();
  } catch (error) {
    failures.push(error);
  }
  if (failures.length) throw new AggregateError(failures, "Audio capture settlement failed");
}
