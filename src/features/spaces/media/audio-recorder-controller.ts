import { computed, onBeforeUnmount, ref } from "vue";

import { removeSpaceAsset } from "../assets/asset-operations";
import {
  classifyAudioCaptureProblem,
  requireAudioRecordingSpace,
  stopAudioCapture,
} from "./audio-permission";
import type { AudioRecorderControllerOptions } from "./audio-recorder-contract";
import {
  appendSpaceAudioRecording,
  cancelSpaceAudioRecording,
  finishSpaceAudioRecording,
  preferredAudioRecordingMimeType,
  startSpaceAudioRecording,
  type SpaceAudioRecordingSession,
} from "./audio-recording";
import { formatDurationMs } from "./format-duration";

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
  let writeQueue = Promise.resolve();
  let writeFailure: unknown;
  let timer = 0;
  let generation = 0;
  let recordingSpaceId: string | undefined;

  async function start() {
    if (isRecording.value || isRequestingPermission.value || isSaving.value) return;
    const availability = requireAudioRecordingSpace(options.spaceId.value);
    if ("problem" in availability) {
      captureProblem.value = availability.problem;
      return;
    }
    const { spaceId } = availability;
    const sessionGeneration = ++generation;
    isRequestingPermission.value = true;
    captureProblem.value = undefined;
    let candidate: MediaStream | undefined;
    let session: SpaceAudioRecordingSession | undefined;
    try {
      candidate = await navigator.mediaDevices.getUserMedia({ audio: true });
      if (!isCurrent(sessionGeneration, spaceId)) return stopAudioCapture(candidate);
      const mime = preferredAudioRecordingMimeType();
      const transport = new MediaRecorder(candidate, mime ? { mimeType: mime } : undefined);
      session = await startSpaceAudioRecording(transport.mimeType);
      if (!isCurrent(sessionGeneration, spaceId)) {
        stopAudioCapture(candidate);
        await cancelSpaceAudioRecording(session);
        return;
      }
      stream = candidate;
      recordingSpaceId = spaceId;
      recorder = transport;
      recorderSession = session;
      writeQueue = Promise.resolve();
      writeFailure = undefined;
      recorder.addEventListener("dataavailable", queueChunk);
      recorder.start(4_000);
      isRecording.value = true;
      timer = globalThis.setInterval(() => !isPaused.value && (elapsedSeconds.value += 1), 1_000);
    } catch (error) {
      if (candidate) stopAudioCapture(candidate);
      if (session) await cancelSpaceAudioRecording(session);
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
    const stopped = new Promise<void>((resolve) =>
      current.addEventListener("stop", () => resolve(), { once: true }),
    );
    current.stop();
    await stopped;
    await writeQueue;
    cleanupMedia();
    if (!isCurrent(sessionGeneration, spaceId)) return;
    try {
      if (writeFailure) throw writeFailure;
      const asset = await finishSpaceAudioRecording(session);
      recorderSession = undefined;
      if (!isCurrent(sessionGeneration, spaceId)) {
        await removeSpaceAsset(asset.path);
        return;
      }
      options.saved(asset, spaceId);
      options.close();
    } catch {
      await cancelRecorderSession();
      if (sessionGeneration === generation) captureProblem.value = "unknown";
      options.failed();
    } finally {
      if (sessionGeneration === generation) isSaving.value = false;
    }
  }

  function togglePause() {
    if (!recorder || recorder.state === "inactive") return;
    if (recorder.state === "recording") recorder.pause();
    else if (recorder.state === "paused") recorder.resume();
    isPaused.value = recorder.state === "paused";
  }

  function queueChunk(event: BlobEvent) {
    if (!event.data.size || !recorderSession) return;
    const session = recorderSession;
    writeQueue = writeQueue
      .then(() => appendSpaceAudioRecording(session, event.data))
      .catch((error) => {
        writeFailure = error;
      });
  }

  async function invalidate() {
    generation += 1;
    const current = recorder;
    recordingSpaceId = undefined;
    if (current && current.state !== "inactive") {
      const stopped = new Promise<void>((resolve) =>
        current.addEventListener("stop", () => resolve(), { once: true }),
      );
      current.stop();
      await stopped;
    }
    await writeQueue;
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
