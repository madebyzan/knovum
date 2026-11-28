import { error as logError } from "@tauri-apps/plugin-log";
import { onScopeDispose, ref } from "vue";

import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";
import { useAudioInputDevices } from "./audio-input-devices";
import { useAudioLevelMeter } from "./audio-level-meter";
import {
  classifyAudioCaptureProblem,
  requireAudioRecordingSpace,
  stopAudioCapture,
  type AudioCaptureProblem,
} from "./audio-permission";
import type { AudioRecorderControllerOptions } from "./audio-recorder-options";
import { cancelSpaceAudioRecording, type SpaceAudioRecordingSession } from "./audio-recording";
import { completeAudioRecording } from "./audio-recording-completion";
import { prepareAudioRecording, type PreparedAudioRecording } from "./audio-recording-start";
import { createAudioRecordingWriter } from "./audio-recording-writer";
import { settleAudioRecording } from "./media-recorder-operations";
import { createRecordingTimer } from "./recording-timer";

export function useAudioRecorderController(options: AudioRecorderControllerOptions) {
  const isRecording = ref(false);
  const isPaused = ref(false);
  const isRequestingPermission = ref(false);
  const isSaving = ref(false);
  const captureProblem = ref<AudioCaptureProblem>();
  const timer = createRecordingTimer();
  const devices = useAudioInputDevices();
  const meter = useAudioLevelMeter();
  let recorder: MediaRecorder | undefined;
  let stream: MediaStream | undefined;
  let recorderSession: SpaceAudioRecordingSession | undefined;
  const recordingWriter = createAudioRecordingWriter();
  let generation = 0;
  let recordingSpaceId: string | undefined;

  async function start(deviceId = devices.selectedDeviceId.value || undefined) {
    if (isRecording.value || isRequestingPermission.value || isSaving.value) return;
    const availability = requireAudioRecordingSpace(options.spaceId.value);
    if ("problem" in availability) return void (captureProblem.value = availability.problem);
    const sessionGeneration = ++generation;
    isRequestingPermission.value = true;
    captureProblem.value = undefined;
    await beginRecording(availability.spaceId, sessionGeneration, deviceId);
  }

  async function beginRecording(spaceId: string, sessionGeneration: number, deviceId?: string) {
    try {
      const prepared = await prepareAudioRecording(() => isCurrent(sessionGeneration, spaceId), deviceId);
      if (prepared) {
        activateRecording(prepared, spaceId, sessionGeneration);
        void devices.refresh();
      }
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
    const sessionGeneration = generation;
    try {
      await completeAudioRecording({
        recorder: current,
        session,
        spaceId,
        settleWriter: recordingWriter.settle,
        writerFailure: recordingWriter.failure,
        isCurrent: () => isCurrent(sessionGeneration, spaceId),
        onSettled: () => sessionGeneration === generation && cleanupMedia(),
        onSessionFinished: () => recorderSession === session && (recorderSession = undefined),
        save: options.save,
        onSaved: () => (options.open.value = false),
        cancelSession: cancelRecorderSession,
        onError: (failure) => {
          if (sessionGeneration === generation) captureProblem.value = "saveFailed";
          reportFailure(failure);
        },
      });
    } finally {
      if (sessionGeneration === generation) isSaving.value = false;
    }
  }

  function handleInterruption(sessionGeneration: number) {
    if (sessionGeneration !== generation || !isRecording.value || isSaving.value) return;
    cleanupMedia();
    captureProblem.value = "interrupted";
    void cancelRecorderSession().catch(reportFailure);
  }

  function activateRecording(input: PreparedAudioRecording, spaceId: string, sessionGeneration: number) {
    stream = input.stream;
    recordingSpaceId = spaceId;
    recorder = input.recorder;
    recorderSession = input.session;
    recordingWriter.reset();
    recorder.addEventListener("dataavailable", (event) => {
      if (event.data.size && recorderSession) recordingWriter.append(recorderSession, event.data);
    });
    recorder.addEventListener("error", () => handleInterruption(sessionGeneration), { once: true });
    stream
      .getAudioTracks?.()
      .forEach((track) =>
        track.addEventListener?.("ended", () => handleInterruption(sessionGeneration), { once: true }),
      );
    meter.start(stream, () => isPaused.value);
    recorder.start(4_000);
    isRecording.value = true;
    timer.start();
  }

  function togglePause() {
    if (!recorder || recorder.state === "inactive") return;
    if (recorder.state === "recording") {
      timer.pause();
      recorder.pause();
    } else if (recorder.state === "paused") {
      timer.resume();
      recorder.resume();
    }
    isPaused.value = recorder.state === "paused";
    if (isPaused.value) meter.clear();
  }

  async function switchDevice(deviceId: string) {
    devices.selectedDeviceId.value = deviceId;
    await invalidate();
    await start(deviceId || undefined);
  }

  async function invalidate() {
    generation += 1;
    const current = recorder;
    recordingSpaceId = undefined;
    const outcomes = await Promise.allSettled([settleAudioRecording(current, recordingWriter.settle)]);
    cleanupMedia();
    const cancelled = await Promise.allSettled([cancelRecorderSession()]);
    isSaving.value = false;
    const reasons = [...outcomes, ...cancelled].flatMap((o) => (o.status === "rejected" ? [o.reason] : []));
    if (reasons.length) throw new AggregateError(reasons, "Audio recording cleanup failed");
  }

  const isCurrent = (sessionGeneration: number, spaceId: string) =>
    sessionGeneration === generation && options.open.value && options.spaceId.value === spaceId;

  function cleanupMedia() {
    timer.reset();
    meter.stop();
    if (stream) stopAudioCapture(stream);
    recorder = stream = undefined;
    isRecording.value = isPaused.value = isRequestingPermission.value = false;
  }

  async function cancelRecorderSession(session = recorderSession) {
    if (recorderSession === session) recorderSession = undefined;
    if (session) await cancelSpaceAudioRecording(session);
  }

  function reportFailure(error: unknown) {
    void logError(`Audio recording lifecycle failed: ${formatErrorDiagnostic(error)}`);
    options.failed();
  }

  onScopeDispose(() => void invalidate().catch(reportFailure));
  return {
    isRecording,
    isPaused,
    isRequestingPermission,
    isSaving,
    captureProblem,
    elapsedMs: timer.elapsedMs,
    elapsedLabel: timer.elapsedLabel,
    inputLevels: meter.levels,
    inputDevices: devices.inputDevices,
    selectedDeviceId: devices.selectedDeviceId,
    start,
    stop,
    togglePause,
    switchDevice,
    invalidate,
  };
}
