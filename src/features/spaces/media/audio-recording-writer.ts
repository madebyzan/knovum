import { appendSpaceAudioRecording, type SpaceAudioRecordingSession } from "./audio-recording";

export function createAudioRecordingWriter() {
  let pending = Promise.resolve();
  let failure: unknown;

  return {
    append(session: SpaceAudioRecordingSession, chunk: Blob) {
      pending = pending
        .then(() => appendSpaceAudioRecording(session, chunk))
        .catch((error) => {
          failure = error;
        });
    },
    failure: () => failure,
    reset() {
      pending = Promise.resolve();
      failure = undefined;
    },
    settle: () => pending,
  };
}
