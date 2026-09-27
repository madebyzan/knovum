import { computed, onBeforeUnmount, ref } from "vue";

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
import { stopMediaRecorder } from "./media-recorder-operations";

export function createSpaceAudioRecorderController(options: AudioRecorderControllerOptions) {
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
  let timer = 0;
  let generation = 0;
  let recordingSpaceId: string | undefined;

  async function start() {
    if (isRecording.value || isRequestingPermission.value || isSaving.value) return;
    const availability = requireAudioRecordingSpace(options.spaceId.value);
    if ("problem" in availability) return reportCaptureProblem(availability.problem);
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
        recorderSession = undefined;
        cleanupMedia();
        captureProblem.value = classifyAudioCaptureProblem(error);
      }
    } finally {
      if (sessionGeneration === generation) isRequestingPermission.value = false;
    }
  }

  async function stop() {
    const current = recorder;
    const sessionGeneration = generation;
    const spaceId = recordingSpaceId;
    const session = recorderSession;
    if (!current || current.state === "inactive" || !spaceId || !session || isSaving.value) return;
    isSaving.value = true;
    await completeRecording({ current, session, sessionGeneration, spaceId });
  }

  async function completeRecording(input: {
    current: MediaRecorder;
    session: SpaceAudioRecordingSession;
    sessionGeneration: number;
    spaceId: string;
  }) {
    const { current, session, sessionGeneration, spaceId } = input;
    await stopMediaRecorder(current);
    await recordingWriter.settle();
    cleanupMedia();
    if (!isCurrent(sessionGeneration, spaceId)) return;
    try {
      if (recordingWriter.failure()) throw recordingWriter.failure();
      await saveRecording(session, sessionGeneration, spaceId);
    } catch {
      await cancelRecorderSession();
      if (sessionGeneration === generation) captureProblem.value = "unknown";
      options.failed();
    } finally {
      if (sessionGeneration === generation) isSaving.value = false;
    }
  }

  function reportCaptureProblem(problem: NonNullable<typeof captureProblem.value>) {
    captureProblem.value = problem;
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
    recorderSession = undefined;
    if (!isCurrent(sessionGeneration, spaceId)) return removeSpaceAsset(asset.path);
    await options.save(asset, spaceId);
    options.close();
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
    if (current && current.state !== "inactive") await stopMediaRecorder(current);
    await recordingWriter.settle();
    cleanupMedia();
    await cancelRecorderSession();
    isSaving.value = false;
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

  async function cancelRecorderSession() {
    const session = recorderSession;
    recorderSession = undefined;
    if (session) await cancelSpaceAudioRecording(session);
  }

  onBeforeUnmount(() => void invalidate());
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
