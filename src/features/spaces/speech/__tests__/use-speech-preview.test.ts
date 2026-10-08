import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  activeSpaceId: "space-1" as string | undefined,
  cancelSpeechGeneration: vi.fn<(jobId: string) => Promise<boolean>>(),
  discardSpeechPreview: vi.fn<(path: string) => Promise<void>>(),
  previewVoice: vi.fn<() => Promise<never>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => `asset:${path}` }));
vi.mock("../../session/space-store", () => ({
  useSpaceStore: () => mocks,
}));
vi.mock("../ipc", () => ({
  cancelSpeechGeneration: mocks.cancelSpeechGeneration,
  discardSpeechPreview: mocks.discardSpeechPreview,
  previewVoice: mocks.previewVoice,
}));

import { useSpeechPreview } from "../use-speech-preview";

const noop = () => undefined;

class PreviewAudioContext {
  state: AudioContextState = "suspended";
  currentTime = 0;
  readonly destination = {} as AudioDestinationNode;

  async resume() {
    this.state = "running";
  }

  async close() {
    this.state = "closed";
  }
}

function deferredPreview() {
  let rejectPreview: (cause?: unknown) => void = noop;
  const promise = new Promise<never>((_resolve, reject) => {
    rejectPreview = reject;
  });
  return { promise, reject: rejectPreview };
}

beforeEach(() => {
  mocks.activeSpaceId = "space-1";
  mocks.cancelSpeechGeneration.mockReset();
  mocks.cancelSpeechGeneration.mockResolvedValue(true);
  mocks.discardSpeechPreview.mockReset();
  mocks.discardSpeechPreview.mockResolvedValue();
  mocks.previewVoice.mockReset();
  vi.stubGlobal("AudioContext", PreviewAudioContext);
});

afterEach(() => vi.unstubAllGlobals());

describe("speech preview playback controls", () => {
  it("stops listening without cancelling generation", async () => {
    const pending = deferredPreview();
    mocks.previewVoice.mockReturnValue(pending.promise);
    const preview = useSpeechPreview(
      () => undefined,
      () => mocks.activeSpaceId,
    );
    const task = preview.previewSelection("hello", "voice-1", {});
    await Promise.resolve();

    preview.stopListening();
    expect(mocks.cancelSpeechGeneration).not.toHaveBeenCalled();

    pending.reject(new Error("preview stopped"));
    await task;
  });

  it("cancels generation with the active preview identity", async () => {
    const pending = deferredPreview();
    mocks.previewVoice.mockReturnValue(pending.promise);
    const preview = useSpeechPreview(
      () => undefined,
      () => mocks.activeSpaceId,
    );
    const task = preview.previewSelection("hello", "voice-1", {});
    await Promise.resolve();
    const activeJobId = preview.jobId.value;

    await preview.cancel();
    expect(activeJobId).toBeTypeOf("string");
    expect(mocks.cancelSpeechGeneration).toHaveBeenCalledExactlyOnceWith(activeJobId);

    pending.reject(new Error("preview cancelled"));
    await task;
  });

  it("sends a trimmed delivery instruction with the preview request", async () => {
    const pending = deferredPreview();
    mocks.previewVoice.mockReturnValue(pending.promise);
    const preview = useSpeechPreview(
      () => undefined,
      () => mocks.activeSpaceId,
    );
    const task = preview.previewSelection("hello", "voice-1", {}, "  Calm narration  ");
    await vi.waitFor(() =>
      expect(mocks.previewVoice).toHaveBeenCalledWith(
        expect.objectContaining({ deliveryInstruction: "Calm narration" }),
        expect.any(Function),
        expect.any(Function),
      ),
    );

    pending.reject(new Error("cancelled", { cause: { code: "cancelled" } }));
    await task;
  });

  it("shows a rejected cancellation without losing the active generation or leaking a UI rejection", async () => {
    const pending = deferredPreview();
    mocks.previewVoice.mockReturnValue(pending.promise);
    mocks.cancelSpeechGeneration.mockRejectedValue(new Error("Cancellation rejected"));
    const report = vi.fn<(message?: string) => void>();
    const preview = useSpeechPreview(report, () => mocks.activeSpaceId);
    const task = preview.previewSelection("hello", "voice-1", {});
    await Promise.resolve();
    const job = preview.jobId.value;
    await expect(preview.cancel()).resolves.toBeUndefined();
    expect(preview.jobId.value).toBe(job);
    expect(report).toHaveBeenCalledWith("Cancellation rejected");
    pending.reject(new Error("Generation failed"));
    await task;
  });

  it("does not write a teardown cancellation failure into the next editor", async () => {
    const pending = deferredPreview();
    mocks.previewVoice.mockReturnValue(pending.promise);
    mocks.cancelSpeechGeneration.mockRejectedValue(new Error("Old cancellation failure"));
    const report = vi.fn<(message?: string) => void>();
    const preview = useSpeechPreview(report, () => mocks.activeSpaceId);
    const task = preview.previewSelection("hello", "voice-1", {});
    await vi.waitFor(() => expect(mocks.previewVoice).toHaveBeenCalledOnce());
    preview.reset();
    await Promise.resolve();
    expect(report).not.toHaveBeenCalled();
    pending.reject(new Error("cancelled", { cause: { code: "cancelled" } }));
    await task;
  });
});
