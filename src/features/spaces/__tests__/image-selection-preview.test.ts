import { beforeEach, afterEach, expect, it, vi } from "vitest";
import { effectScope } from "vue";

import type { ImageSelection } from "../contracts/generated/ImageSelection";
import { useImageSelectionPreview } from "../knowledge-derivations/use-image-selection-preview";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
const calls = vi.hoisted(() => ({
  preview: vi.fn<(...args: unknown[]) => Promise<string>>(),
  cancel: vi.fn<(id: string) => Promise<void>>(),
}));
vi.mock("../knowledge-derivations/image-selection-preview", () => ({ previewImageSelection: calls.preview }));
vi.mock("../../../foundation/ai/ai-runtime", () => ({ cancelAiJob: calls.cancel }));
const node: SpaceCanvasNode = {
  id: "image",
  type: "image",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "image", title: "Photo", fileName: "photo.png", assetPath: "/photo.png" },
};
let selection: ImageSelection;
beforeEach(() => {
  vi.useFakeTimers();
  calls.preview.mockReset();
  calls.cancel.mockReset().mockResolvedValue(undefined);
  selection = { points: [{ x: 0.3, y: 0.3, foreground: true }], boxRect: null, strokes: [] };
});
afterEach(() => vi.useRealTimers());
function session() {
  const scope = effectScope();
  const preview = scope.run(() =>
    useImageSelectionPreview(() => ({
      node,
      spaceId: "space",
      execution: { type: "local", model: "efficientSamTi", specId: "spec" },
      selection,
    })),
  )!;
  return { scope, preview };
}
it("serializes rapid prompts, keeps the encoding worker and displays only the newest mask", async () => {
  let first!: (mask: string) => void;
  let second!: (mask: string) => void;
  calls.preview
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          first = resolve;
        }),
    )
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          second = resolve;
        }),
    );
  const { scope, preview } = session();
  preview.queue();
  await vi.advanceTimersByTimeAsync(150);
  selection = { ...selection, points: [...selection.points, { x: 0.7, y: 0.7, foreground: false }] };
  preview.queue();
  await vi.advanceTimersByTimeAsync(150);
  expect(calls.preview).toHaveBeenCalledOnce();
  first("old-mask");
  await vi.advanceTimersByTimeAsync(0);
  expect(calls.preview).toHaveBeenCalledTimes(2);
  expect(preview.maskSource.value).toBeUndefined();
  expect(calls.cancel).not.toHaveBeenCalled();
  second("new-mask");
  await vi.advanceTimersByTimeAsync(0);
  expect(preview.maskSource.value).toBe("new-mask");
  expect(preview.busy.value).toBe(false);
  scope.stop();
});
it("cancels a closed session and ignores its late response", async () => {
  let complete!: (mask: string) => void;
  calls.preview.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        complete = resolve;
      }),
  );
  const { scope, preview } = session();
  preview.queue();
  await vi.advanceTimersByTimeAsync(150);
  preview.clear();
  complete("late-mask");
  await vi.advanceTimersByTimeAsync(0);
  expect(calls.cancel).toHaveBeenCalledOnce();
  expect(preview.maskSource.value).toBeUndefined();
  expect(preview.busy.value).toBe(false);
  scope.stop();
});
