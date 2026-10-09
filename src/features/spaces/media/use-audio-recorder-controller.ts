import { error as logError } from "@tauri-apps/plugin-log";
import { computed, onBeforeUnmount, ref } from "vue";

import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";
import { removeSpaceAsset } from "../assets/asset-operations";
import {
  classifyAudioCaptureProblem,
  requireAudioRecordingSpace,
  stopAudioCapture,
} from "./audio-permission";
import type { AudioRecorderControllerOptions } from "./audio-recorder-options";
import {
  cancelSpaceAudioRecording,
  finishSpaceAudioRecording,
  type SpaceAudioRecordingSession,
} from "./audio-recording";
import { prepareAudioRecording, type PreparedAudioRecording } from "./audio-recording-start";
import { createAudioRecordingWriter } from "./audio-recording-writer";
import { formatDurationMs } from "./format-duration";
import { settleAudioRecording } from "./media-recorder-operations";

interface CompletedRecordingInput {
  current: MediaRecorder;
  session: SpaceAudioRecordingSession;
  sessionGeneration: number;
  spaceId: string;
}

export function useAudioRecorderController(options: AudioRecorderControllerOptions) {
  const isRecording = ref(false);
  const isPaused = ref(false);
  const isRequestingPermission = ref(false);
  const isSaving = ref(false);
  const captureProblem = ref<ReturnType<typeof classifyAudioCaptureProblem>>();
  const elapsedSeconds = ref(0);
  const elapsedLabel = computed(() => formatDurationMs(elapsedSeconds.value * 1000));
  let recorder: MediaRecorder | undefined;
  let stream: MediaStream | undefined;
  let recorderSession: SpaceAudioRecordingSession | undefined;
  const recordingWriter = createAudioRecordingWriter();
  let timer: ReturnType<typeof globalThis.setInterval> | undefined;
  let generation = 0;
  let recordingSpaceId: string | undefined;

  async function start() {
    if (isRecording.value || isRequestingPermission.value || isSaving.value) return;
    const availability = requireAudioRecordingSpace(options.spaceId.value);
    if ("problem" in availability) {
      captureProblem.value = availability.problem;
      return;
    }
    const sessionGeneration = ++generation;
    isRequestingPermission.value = true;
    captureProblem.value = undefined;
    await beginRecording(availability.spaceId, sessionGeneration);
  }

  async function beginRecording(spaceId: string, sessionGeneration: number) {
    try {
      const prepared = await prepareAudioRecording(() => isCurrent(sessionGeneration, spaceId));
      if (prepared) activateRecording(prepared, spaceId);
    } catch (error) {
      if (sessionGeneration === generation) {
        cleanupMedia();
        captureProblem.value = classifyAudioCaptureProblem(error);
        await cancelRecorderSession().catch(reportFailure);
      }
    } finally {
      if (sessionGeneration === generation) isRequestingPermission.value = false;
    }
  }

  async function stop() {
    const current = recorder;
    const spaceId = recordingSpaceId;
    const session = recorderSession;
    if (!current || current.state === "inactive" || !spaceId || !session || isSaving.value) return;
    isSaving.value = true;
    await completeRecording({ current, session, sessionGeneration: generation, spaceId });
  }

  async function completeRecording(input: CompletedRecordingInput) {
    const { current, session, sessionGeneration, spaceId } = input;
    try {
      await settleAudioRecording(current, recordingWriter.settle).finally(() => {
        if (sessionGeneration === generation) cleanupMedia();
      });
      if (!isCurrent(sessionGeneration, spaceId)) return;
      if (recordingWriter.failure()) throw recordingWriter.failure();
      await saveRecording(session, sessionGeneration, spaceId);
    } catch (error) {
      let failure = error;
      if (sessionGeneration === generation) captureProblem.value = "unknown";
      try {
        await cancelRecorderSession(session);
      } catch (cleanupError) {
        failure = new AggregateError([error, cleanupError], "Audio recording and session cleanup failed");
      }
      reportFailure(failure);
    } finally {
      if (sessionGeneration === generation) isSaving.value = false;
    }
  }

  function activateRecording(input: PreparedAudioRecording, spaceId: string) {
    stream = input.stream;
    recordingSpaceId = spaceId;
    recorder = input.recorder;
    recorderSession = input.session;
    recordingWriter.reset();
    recorder.addEventListener("dataavailable", queueChunk);
    recorder.start(4_000);
    isRecording.value = true;
    timer = globalThis.setInterval(() => !isPaused.value && (elapsedSeconds.value += 1), 1_000);
  }

  async function saveRecording(
    session: SpaceAudioRecordingSession,
    sessionGeneration: number,
    spaceId: string,
  ) {
    const asset = await finishSpaceAudioRecording(session);
    if (recorderSession === session) recorderSession = undefined;
    if (!isCurrent(sessionGeneration, spaceId)) return removeSpaceAsset(asset.path);
    await options.save(asset, spaceId);
    if (isCurrent(sessionGeneration, spaceId)) options.open.value = false;
  }

  function togglePause() {
    if (!recorder || recorder.state === "inactive") return;
    if (recorder.state === "recording") recorder.pause();
    else if (recorder.state === "paused") recorder.resume();
    isPaused.value = recorder.state === "paused";
  }

  function queueChunk(event: BlobEvent) {
    if (!event.data.size || !recorderSession) return;
    recordingWriter.append(recorderSession, event.data);
  }

  async function invalidate() {
    generation += 1;
    const current = recorder;
    recordingSpaceId = undefined;
    const outcomes = await Promise.allSettled([settleAudioRecording(current, recordingWriter.settle)]);
    cleanupMedia();
    const cancelled = await Promise.allSettled([cancelRecorderSession()]);
    isSaving.value = false;
    const failures = [...outcomes, ...cancelled].filter((outcome) => outcome.status === "rejected");
    if (failures.length)
      throw new AggregateError(
        failures.map((failure) => failure.reason),
        "Audio recording cleanup failed",
      );
  }

  function isCurrent(sessionGeneration: number, spaceId: string) {
    return sessionGeneration === generation && options.open.value && options.spaceId.value === spaceId;
  }

  function cleanupMedia() {
    globalThis.clearInterval(timer);
    if (stream) stopAudioCapture(stream);
    recorder = undefined;
    stream = undefined;
    isRecording.value = false;
    isPaused.value = false;
    isRequestingPermission.value = false;
    elapsedSeconds.value = 0;
  }

  async function cancelRecorderSession(session = recorderSession) {
    if (recorderSession === session) recorderSession = undefined;
    if (session) await cancelSpaceAudioRecording(session);
  }

  function reportFailure(error: unknown) {
    void logError(`Audio recording lifecycle failed: ${formatErrorDiagnostic(error)}`);
    options.failed();
  }
  onBeforeUnmount(() => void invalidate().catch(reportFailure));
  return {
    isRecording,
    isPaused,
    isRequestingPermission,
    isSaving,
    captureProblem,
    elapsedLabel,
    start,
    stop,
    togglePause,
    invalidate,
  };
}
