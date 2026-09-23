import { stopAudioCapture } from "./audio-permission";
import {
  cancelSpaceAudioRecording,
  startSpaceAudioRecording,
  type SpaceAudioRecordingSession,
} from "./audio-recording";
import { cancelPendingAudioRecording, createAudioMediaRecorder } from "./media-recorder-operations";

export interface PreparedAudioRecording {
  stream: MediaStream;
  recorder: MediaRecorder;
  session: SpaceAudioRecordingSession;
}

export async function prepareAudioRecording(isCurrent: () => boolean) {
  let stream: MediaStream | undefined;
  let session: SpaceAudioRecordingSession | undefined;
  try {
    stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    if (!isCurrent()) return stopAudioCapture(stream);
    const recorder = createAudioMediaRecorder(stream);
    session = await startSpaceAudioRecording(recorder.mimeType);
    if (!isCurrent()) return cancelPendingAudioRecording(stream, session);
    return { stream, recorder, session } satisfies PreparedAudioRecording;
  } catch (error) {
    if (stream) stopAudioCapture(stream);
    if (session) await cancelSpaceAudioRecording(session);
    throw error;
  }
}
