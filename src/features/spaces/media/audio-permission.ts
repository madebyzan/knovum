import { isFunction, isNotNil } from "es-toolkit";

export type AudioCaptureProblem = "denied" | "missing" | "busy" | "unsupported" | "unknown";

export function classifyAudioCaptureProblem(reason: unknown): AudioCaptureProblem {
  if (!(reason instanceof DOMException)) return "unknown";
  if (reason.name === "NotAllowedError" || reason.name === "SecurityError") return "denied";
  if (reason.name === "NotFoundError" || reason.name === "DevicesNotFoundError") return "missing";
  if (reason.name === "NotReadableError" || reason.name === "TrackStartError") return "busy";
  return "unknown";
}

function audioCaptureSupported(): boolean {
  return isFunction(globalThis.navigator?.mediaDevices?.getUserMedia) && isNotNil(globalThis.MediaRecorder);
}

export function requireAudioRecordingSpace(spaceId: string | undefined) {
  if (!spaceId) return { problem: "unknown" as const };
  if (!audioCaptureSupported()) return { problem: "unsupported" as const };
  return { spaceId };
}

export function stopAudioCapture(stream: MediaStream) {
  stream.getTracks().forEach((track) => track.stop());
}
