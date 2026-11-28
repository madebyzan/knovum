import { describe, expect, it, vi } from "vitest";
import { effectScope } from "vue";

vi.mock("vue-sonner", () => ({ toast: { error: vi.fn<(message: string) => void>() } }));

const cancelAiJobMock = vi.fn<() => Promise<void>>(() => Promise.resolve());
vi.mock("../../../../foundation/ai/ai-runtime", () => ({
  cancelAiJob: () => cancelAiJobMock(),
}));

const prepareStreamMock =
  vi.fn<(options: { onEvent?: (event: { event: "delta"; text: string }) => void }) => Promise<string>>();
vi.mock("../../../../foundation/ai/narration-preparation", () => ({
  prepareNarrationScriptStream: (options: { onEvent?: (event: { event: "delta"; text: string }) => void }) =>
    prepareStreamMock(options),
}));

import { useNarrationScriptPreparation } from "../use-narration-script-preparation";

describe("useNarrationScriptPreparation", () => {
  it("streams preview, sanitizes completion, and restores previous snapshot", async () => {
    prepareStreamMock.mockImplementationOnce(async ({ onEvent }) => {
      onEvent?.({ event: "delta", text: "Hello " });
      onEvent?.({ event: "delta", text: "world [C1]." });
      return "Hello world [C1].";
    });
    const scope = effectScope();
    const prep = scope.run(() => useNarrationScriptPreparation<string>())!;
    const previews: string[] = [];
    let current = "Raw [C1] text";

    const ok = await prep.run({
      mode: "clean",
      sourceText: current,
      snapshot: current,
      onPreview: (partial) => previews.push(partial),
      onComplete: (cleaned) => {
        current = cleaned;
      },
      onRestore: (previous) => {
        current = previous;
      },
    });

    expect(ok).toBe(true);
    expect(previews).toEqual(["Hello ", "Hello world [C1]."]);
    expect(current).toBe("Hello world.");
    expect(prep.canRestore.value).toBe(true);

    prep.restore((previous) => {
      current = previous;
    });
    expect(current).toBe("Raw [C1] text");
    expect(prep.canRestore.value).toBe(false);
    scope.stop();
  });

  it("restores snapshot and cancels backend job on abort", async () => {
    let resolveStream!: (value: string) => void;
    prepareStreamMock.mockImplementationOnce(
      ({ onEvent }) =>
        new Promise<string>((resolve) => {
          onEvent?.({ event: "delta", text: "Half..." });
          resolveStream = resolve;
        }),
    );
    const scope = effectScope();
    const prep = scope.run(() => useNarrationScriptPreparation<string>())!;
    let current = "Original script";

    const runPromise = prep.run({
      mode: "adapt",
      sourceText: current,
      snapshot: current,
      onPreview: (partial) => {
        current = partial;
      },
      onComplete: (cleaned) => {
        current = cleaned;
      },
      onRestore: (previous) => {
        current = previous;
      },
    });

    expect(current).toBe("Half...");
    prep.cancel();
    expect(cancelAiJobMock).toHaveBeenCalledTimes(1);
    resolveStream("Half...");
    const ok = await runPromise;

    expect(ok).toBe(false);
    expect(current).toBe("Original script");
    expect(prep.canRestore.value).toBe(false);
    scope.stop();
  });
});
