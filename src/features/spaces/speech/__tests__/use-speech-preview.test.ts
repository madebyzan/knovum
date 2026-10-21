import type { error as logError } from "@tauri-apps/plugin-log";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type * as ipc from "../ipc";

const mocks = vi.hoisted(() => ({
  cancelSpeechGeneration: vi.fn<(jobId: string) => Promise<boolean>>(),
  discardSpeechPreview: vi.fn<(path: string) => Promise<void>>(),
  logError: vi.fn<typeof logError>(),
  previewVoice: vi.fn<typeof ipc.previewVoice>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => `asset:${path}` }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));
vi.mock("../ipc", () => ({
  cancelSpeechGeneration: mocks.cancelSpeechGeneration,
  discardSpeechPreview: mocks.discardSpeechPreview,
  previewVoice: mocks.previewVoice,
}));

import { deferred } from "../../../../shared/__tests__/support/deferred";
import { useSpeechPreview } from "../use-speech-preview";

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

beforeEach(() => {
  mocks.cancelSpeechGeneration.mockReset();
  mocks.cancelSpeechGeneration.mockResolvedValue(true);
  mocks.discardSpeechPreview.mockReset();
  mocks.discardSpeechPreview.mockResolvedValue();
  mocks.logError.mockReset();
  mocks.logError.mockResolvedValue();
  mocks.previewVoice.mockReset();
  vi.stubGlobal("AudioContext", PreviewAudioContext);
});

afterEach(() => vi.unstubAllGlobals());

describe("speech preview playback controls", () => {
  it("stops listening without cancelling generation", async () => {
    const pending = deferred<never>();
    mocks.previewVoice.mockReturnValue(pending.promise);
    const preview = useSpeechPreview();
    const task = preview.previewSelection("hello", "voice-1", {});
    await Promise.resolve();

    preview.stopListening();
    expect(mocks.cancelSpeechGeneration).not.toHaveBeenCalled();

    pending.reject(new Error("preview stopped"));
    await task;
  });

  it("cancels generation with the active preview identity", async () => {
    const pending = deferred<never>();
    mocks.previewVoice.mockReturnValue(pending.promise);
    const preview = useSpeechPreview();
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
    const pending = deferred<never>();
    mocks.previewVoice.mockReturnValue(pending.promise);
    const preview = useSpeechPreview();
    const task = preview.previewSelection("hello", "voice-1", {}, "  Calm narration  ");
    await vi.waitFor(() =>
      expect(mocks.previewVoice).toHaveBeenCalledWith(
        expect.objectContaining({ deliveryInstruction: "Calm narration" }),
        expect.any(Function),
        expect.any(Function),
      ),
    );
    expect(mocks.previewVoice.mock.calls[0]?.[0]).not.toHaveProperty("spaceId");

    pending.reject(new Error("cancelled", { cause: { code: "cancelled" } }));
    await task;
  });

  it("shows a rejected cancellation without losing the active generation or leaking a UI rejection", async () => {
    const pending = deferred<never>();
    mocks.previewVoice.mockReturnValue(pending.promise);
    mocks.cancelSpeechGeneration.mockRejectedValue(new Error("Cancellation rejected"));
    const preview = useSpeechPreview();
    const task = preview.previewSelection("hello", "voice-1", {});
    await Promise.resolve();
    const job = preview.jobId.value;
    await expect(preview.cancel()).resolves.toBeUndefined();
    expect(preview.jobId.value).toBe(job);
    expect(preview.error.value).toBe("Cancellation rejected");
    pending.reject(new Error("Generation failed"));
    await task;
  });

  it("does not write a teardown cancellation failure into the next editor", async () => {
    const pending = deferred<never>();
    mocks.previewVoice.mockReturnValue(pending.promise);
    mocks.cancelSpeechGeneration.mockRejectedValue(new Error("Old cancellation failure"));
    const preview = useSpeechPreview();
    const task = preview.previewSelection("hello", "voice-1", {});
    await vi.waitFor(() => expect(mocks.previewVoice).toHaveBeenCalledOnce());
    preview.reset();
    await vi.waitFor(() => expect(mocks.logError).toHaveBeenCalledOnce());
    expect(preview.error.value).toBeUndefined();
    pending.reject(new Error("cancelled", { cause: { code: "cancelled" } }));
    await task;
  });

  it("logs stale output cleanup failures without replacing the current preview error", async () => {
    let resolveOld!: (output: {
      path: string;
      sha256: string;
      sampleRate: number;
      sampleCount: number;
    }) => void;
    mocks.previewVoice
      .mockReturnValueOnce(
        new Promise((resolve) => {
          resolveOld = resolve;
        }),
      )
      .mockRejectedValueOnce(new Error("Current preview failure"));
    mocks.discardSpeechPreview.mockRejectedValue(new Error("Stale cleanup failure"));
    const preview = useSpeechPreview();
    const oldTask = preview.previewSelection("old", "voice-1", {});
    await vi.waitFor(() => expect(mocks.previewVoice).toHaveBeenCalledOnce());
    preview.reset();
    await preview.previewSelection("new", "voice-1", {});
    expect(preview.error.value).toBe("Current preview failure");

    resolveOld({ path: "/old.wav", sha256: "sha256:old", sampleRate: 24_000, sampleCount: 1 });
    await oldTask;
    await vi.waitFor(() => expect(mocks.logError).toHaveBeenCalledOnce());
    expect(mocks.logError.mock.calls[0]?.[0]).toContain("Speech preview cleanup failed:");
    expect(mocks.logError.mock.calls[0]?.[0]).toContain("Stale cleanup failure");
    expect(preview.error.value).toBe("Current preview failure");
  });
});
