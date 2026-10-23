import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  previewSelection: vi.fn<(text: string, revisionId: string, lexicon: object) => Promise<void>>(),
  reset: vi.fn<() => void>(),
  peak: vi.fn<() => number>(),
}));

vi.mock("../use-speech-preview", () => ({
  useSpeechPreview: () => ({
    error: { value: undefined },
    previewSelection: mocks.previewSelection,
    reset: mocks.reset,
    peak: mocks.peak,
  }),
}));

import { useVoiceAudition } from "../use-voice-audition";

describe("voice audition", () => {
  beforeEach(() => {
    mocks.previewSelection.mockReset().mockResolvedValue();
    mocks.reset.mockReset();
    mocks.peak.mockReset().mockReturnValue(0);
  });

  it("synthesises the sample line with the voice's active revision", async () => {
    const audition = useVoiceAudition(() => "Sample line");

    await audition.audition("voice-1", "revision-9");

    expect(mocks.previewSelection).toHaveBeenCalledExactlyOnceWith("Sample line", "revision-9", {});
  });

  it("marks only the playing row and reports every other row as busy", () => {
    const audition = useVoiceAudition(() => "Sample line");
    mocks.previewSelection.mockReturnValue(new Promise(() => {}));

    void audition.audition("voice-1", "revision-9");

    expect(audition.stateFor("voice-1")).toMatchObject({ playing: true, busy: false });
    expect(audition.stateFor("voice-2")).toMatchObject({ playing: false, busy: true });
  });

  it("stops the voice that is speaking before another one starts", () => {
    const audition = useVoiceAudition(() => "Sample line");
    mocks.previewSelection.mockReturnValue(new Promise(() => {}));

    void audition.audition("voice-1", "revision-9");
    void audition.audition("voice-2", "revision-7");

    expect(mocks.reset).toHaveBeenCalled();
    expect(audition.stateFor("voice-1").playing).toBe(false);
    expect(audition.stateFor("voice-2").playing).toBe(true);
  });

  it("stops rather than restarts when the playing row is pressed again", () => {
    const audition = useVoiceAudition(() => "Sample line");
    mocks.previewSelection.mockReturnValue(new Promise(() => {}));

    void audition.audition("voice-1", "revision-9");
    void audition.audition("voice-1", "revision-9");

    expect(mocks.previewSelection).toHaveBeenCalledOnce();
    expect(audition.stateFor("voice-1").playing).toBe(false);
  });
});
