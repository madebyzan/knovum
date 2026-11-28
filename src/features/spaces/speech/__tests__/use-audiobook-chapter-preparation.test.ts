import { describe, expect, it, vi } from "vitest";
import { effectScope } from "vue";

vi.mock("vue-sonner", () => ({ toast: { error: vi.fn<(message: string) => void>() } }));

const cancelAiJobMock = vi.fn<() => Promise<void>>(() => Promise.resolve());
vi.mock("../../../../foundation/ai/ai-runtime", () => ({
  cancelAiJob: () => cancelAiJobMock(),
}));

const prepareStreamMock = vi.fn<(options: { text: string; target?: unknown }) => Promise<string>>();
vi.mock("../../../../foundation/ai/narration-preparation", () => ({
  prepareNarrationScriptStream: (options: { text: string; target?: unknown }) => prepareStreamMock(options),
}));

import type { ChapterDraft } from "../audiobook-draft";
import { useAudiobookChapterPreparation } from "../use-audiobook-chapter-preparation";

function makeChapter(key: string, text: string): ChapterDraft {
  return {
    key,
    title: `Chapter ${key}`,
    paragraphs: [{ key: `${key}-p1`, speakerKey: "narrator", text, deliveryInstruction: "" }],
  };
}

describe("useAudiobookChapterPreparation", () => {
  it("prepares all non-empty chapters sequentially with target and restores all atomically", async () => {
    prepareStreamMock.mockResolvedValueOnce("Cleaned one").mockResolvedValueOnce("Cleaned two");
    const scope = effectScope();
    const prep = scope.run(() => useAudiobookChapterPreparation(() => "narrator"))!;
    const chapters = [makeChapter("c1", "Raw [1]"), makeChapter("c2", "   "), makeChapter("c3", "Raw [2]")];
    const target = { type: "model" as const, connectionId: "conn-1", modelId: "gpt-4o" };

    await prep.prepareAllChapters(chapters, "clean", target);

    expect(prepareStreamMock).toHaveBeenCalledTimes(2);
    expect(prepareStreamMock).toHaveBeenNthCalledWith(1, expect.objectContaining({ target }));
    expect(chapters[0]!.paragraphs[0]!.text).toBe("Cleaned one");
    expect(chapters[2]!.paragraphs[0]!.text).toBe("Cleaned two");
    expect(prep.canRestoreBook.value).toBe(true);
    expect(prep.batchProgress.value).toBeUndefined();

    prep.restoreAllChapters(chapters);
    expect(chapters[0]!.paragraphs[0]!.text).toBe("Raw [1]");
    expect(chapters[2]!.paragraphs[0]!.text).toBe("Raw [2]");
    expect(prep.canRestoreBook.value).toBe(false);
    scope.stop();
  });

  it("stops batch preparation on cancel and preserves rollback for already completed chapters", async () => {
    let releaseSecond!: (value: string) => void;
    prepareStreamMock.mockResolvedValueOnce("Cleaned one").mockImplementationOnce(
      () =>
        new Promise<string>((resolve) => {
          releaseSecond = resolve;
        }),
    );
    const scope = effectScope();
    const prep = scope.run(() => useAudiobookChapterPreparation(() => "narrator"))!;
    const chapters = [makeChapter("c1", "Original 1"), makeChapter("c2", "Original 2")];

    const runPromise = prep.prepareAllChapters(chapters, "adapt");
    await vi.waitFor(() => expect(prep.batchProgress.value).toEqual({ current: 2, total: 2 }));
    prep.cancel();
    releaseSecond("Should not apply");
    await runPromise;

    expect(chapters[0]!.paragraphs[0]!.text).toBe("Cleaned one");
    expect(chapters[1]!.paragraphs[0]!.text).toBe("Original 2");
    expect(prep.canRestoreBook.value).toBe(true);
    prep.restoreAllChapters(chapters);
    expect(chapters[0]!.paragraphs[0]!.text).toBe("Original 1");
    scope.stop();
  });
});
