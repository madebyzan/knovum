import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope, ref } from "vue";

import { useSpaceAudioRecorderController } from "../media/use-audio-recorder-controller";

const calls = vi.hoisted(() => ({
  cancel: vi.fn<() => Promise<void>>(),
  finish: vi.fn<() => Promise<unknown>>(),
  stop: vi.fn<() => void>(),
  log: vi.fn<() => Promise<void>>(),
  settle: vi.fn<() => Promise<void>>(),
}));
vi.mock("@tauri-apps/plugin-log", () => ({ error: calls.log }));
vi.mock("../media/audio-permission", () => ({
  requireAudioRecordingSpace: (spaceId: string) => ({ spaceId }),
  classifyAudioCaptureProblem: () => "unknown",
  stopAudioCapture: calls.stop,
}));
vi.mock("../media/audio-recording", () => ({
  cancelSpaceAudioRecording: calls.cancel,
  finishSpaceAudioRecording: calls.finish,
}));
vi.mock("../assets/asset-operations", () => ({ removeSpaceAsset: vi.fn<() => Promise<void>>() }));
vi.mock("../media/audio-recording-start", () => ({
  prepareAudioRecording: async () => ({
    recorder,
    stream: {},
    session: { id: "session", mimeType: "audio/webm" },
  }),
}));
vi.mock("../media/audio-recording-writer", () => ({
  createAudioRecordingWriter: () => ({
    reset() {},
    append() {},
    settle: calls.settle,
    failure: () => undefined,
  }),
}));
const recorder = Object.assign(new EventTarget(), {
  state: "inactive",
  start: vi.fn<() => void>(() => {
    recorder.state = "recording";
  }),
  stop: vi.fn<() => void>(() => {
    recorder.state = "inactive";
    recorder.dispatchEvent(new Event("stop"));
  }),
});
let scope: ReturnType<typeof effectScope>;
beforeEach(() => {
  vi.useFakeTimers();
  vi.clearAllMocks();
  calls.cancel.mockResolvedValue(undefined);
  calls.log.mockResolvedValue(undefined);
  calls.settle.mockResolvedValue(undefined);
  recorder.state = "inactive";
  scope = effectScope();
});
afterEach(() => {
  scope.stop();
  vi.useRealTimers();
});

function createController() {
  const failed = vi.fn<() => void>();
  const controller = scope.run(() =>
    useSpaceAudioRecorderController({
      open: ref(true),
      spaceId: ref("space"),
      save: vi.fn<() => Promise<void>>(),
      failed,
      close: vi.fn<() => void>(),
    }),
  )!;
  return { controller, failed };
}

it("releases capture and the backend session when stopping fails", async () => {
  recorder.stop.mockImplementationOnce(() => {
    throw new Error("stop failed");
  });
  const { controller, failed } = createController();
  await controller.start();
  await controller.stop();
  expect(controller.isSaving.value).toBe(false);
  expect(controller.isRecording.value).toBe(false);
  expect(calls.cancel).toHaveBeenCalledOnce();
  expect(calls.stop).toHaveBeenCalledOnce();
  expect(failed).toHaveBeenCalledOnce();
});

it("releases the microphone before waiting for failed-session cancellation", async () => {
  recorder.stop.mockImplementationOnce(() => {
    throw new Error("stop failed");
  });
  let release!: () => void;
  calls.cancel.mockReturnValueOnce(
    new Promise<void>((resolve) => {
      release = resolve;
    }),
  );
  const { controller } = createController();
  await controller.start();
  const stopping = controller.stop();
  await vi.advanceTimersByTimeAsync(0);
  const releasedBeforeCancellation = calls.stop.mock.calls.length;
  const recordingBeforeCancellation = controller.isRecording.value;
  release();
  await stopping;
  expect(releasedBeforeCancellation).toBe(1);
  expect(recordingBeforeCancellation).toBe(false);
});

it("reports the original failure and cancellation failure while clearing UI state", async () => {
  recorder.stop.mockImplementationOnce(() => {
    throw new Error("stop failed");
  });
  calls.cancel.mockRejectedValueOnce(new Error("session cleanup failed"));
  const { controller, failed } = createController();
  await controller.start();
  await controller.stop();
  expect(controller.isSaving.value).toBe(false);
  expect(failed).toHaveBeenCalledOnce();
  expect(calls.log).toHaveBeenCalledWith(expect.stringContaining("stop failed"));
  expect(calls.log).toHaveBeenCalledWith(expect.stringContaining("session cleanup failed"));
});

it("cancels the session and releases capture when pending chunk settlement fails", async () => {
  calls.settle.mockRejectedValueOnce(new Error("chunk write failed"));
  const { controller, failed } = createController();
  await controller.start();
  await controller.stop();
  expect(calls.finish).not.toHaveBeenCalled();
  expect(calls.cancel).toHaveBeenCalledOnce();
  expect(calls.stop).toHaveBeenCalledOnce();
  expect(controller.isSaving.value).toBe(false);
  expect(failed).toHaveBeenCalledOnce();
});

it("releases media even when invalidation cannot stop capture or cancel its session", async () => {
  recorder.stop.mockImplementationOnce(() => {
    throw new Error("stop failed");
  });
  calls.cancel.mockRejectedValueOnce(new Error("cancel failed"));
  const { controller } = createController();
  await controller.start();
  await expect(controller.invalidate()).rejects.toBeInstanceOf(AggregateError);
  expect(calls.stop).toHaveBeenCalledOnce();
  expect(controller.isSaving.value).toBe(false);
  expect(controller.isRecording.value).toBe(false);
});
