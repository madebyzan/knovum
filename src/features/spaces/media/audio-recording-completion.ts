import { removeSpaceAsset } from "../assets/asset-operations";
import {
  finishSpaceAudioRecording,
  type PersistedAudioSpaceAsset,
  type SpaceAudioRecordingSession,
} from "./audio-recording";
import { settleAudioRecording } from "./media-recorder-operations";

export interface CompleteAudioRecordingInput {
  recorder: MediaRecorder;
  session: SpaceAudioRecordingSession;
  spaceId: string;
  settleWriter: () => Promise<void>;
  writerFailure: () => unknown;
  isCurrent: () => boolean;
  onSettled: () => void;
  onSessionFinished: () => void;
  save: (asset: PersistedAudioSpaceAsset, spaceId: string) => Promise<void>;
  onSaved: () => void;
  cancelSession: (session: SpaceAudioRecordingSession) => Promise<void>;
  onError: (error: unknown) => void;
}

export async function completeAudioRecording(input: CompleteAudioRecordingInput) {
  try {
    await settleAudioRecording(input.recorder, input.settleWriter).finally(input.onSettled);
    if (!input.isCurrent()) return;
    if (input.writerFailure()) throw input.writerFailure();
    const asset = await finishSpaceAudioRecording(input.session);
    input.onSessionFinished();
    if (!input.isCurrent()) return removeSpaceAsset(asset.path);
    await input.save(asset, input.spaceId);
    if (input.isCurrent()) input.onSaved();
  } catch (error) {
    let failure = error;
    try {
      await input.cancelSession(input.session);
    } catch (cleanupError) {
      failure = new AggregateError([error, cleanupError], "Audio recording and session cleanup failed");
    }
    input.onError(failure);
  }
}
