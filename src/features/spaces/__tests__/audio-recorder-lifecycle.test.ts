import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope, ref } from "vue";

import type { PersistedAudioSpaceAsset } from "../media/audio-recording";
import { useSpaceAudioRecorderController } from "../media/use-audio-recorder-controller";

const calls = vi.hoisted(() => ({
  cancel: vi.fn<() => Promise<void>>(),
  finish: vi.fn<() => Promise<PersistedAudioSpaceAsset>>(),
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
  const open = ref(true);
  const spaceId = ref<string | undefined>("space");
  const save = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
  const failed = vi.fn<() => void>();
  const controller = scope.run(() =>
    useSpaceAudioRecorderController({
      open,
      spaceId,
      save,
      failed,
    }),
  )!;
  return { controller, failed, open, spaceId, save };
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

const asset: PersistedAudioSpaceAsset = {
  name: "Recording",
  path: "/recording.webm",
  kind: "audio",
  size: 100,
  modifiedAt: 1,
  sourceRevision: `sha256:${"a".repeat(64)}`,
};

it("closes its model only after the recording is saved", async () => {
  calls.finish.mockResolvedValueOnce(asset);
  const { controller, open, save } = createController();
  await controller.start();
  await controller.stop();
  expect(save).toHaveBeenCalledWith(asset, "space");
  expect(open.value).toBe(false);
});

it("does not close a new context when an earlier save finishes", async () => {
  calls.finish.mockResolvedValueOnce(asset);
  const { controller, open, spaceId, save } = createController();
  let finishSave!: () => void;
  save.mockReturnValueOnce(
    new Promise<void>((resolve) => {
      finishSave = resolve;
    }),
  );
  await controller.start();
  const stopping = controller.stop();
  await vi.advanceTimersByTimeAsync(0);
  expect(save).toHaveBeenCalledOnce();
  spaceId.value = "other-space";
  finishSave();
  await stopping;
  expect(open.value).toBe(true);
});
