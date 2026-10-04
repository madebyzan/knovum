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
    const preview = useSpeechPreview(() => undefined);
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
    const preview = useSpeechPreview(() => undefined);
    const task = preview.previewSelection("hello", "voice-1", {});
    await Promise.resolve();
    const activeJobId = preview.jobId.value;

    await preview.cancel();
    expect(activeJobId).toBeTypeOf("string");
    expect(mocks.cancelSpeechGeneration).toHaveBeenCalledExactlyOnceWith(activeJobId);

    pending.reject(new Error("preview cancelled"));
    await task;
  });
});
