import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";
import { nextTick, reactive } from "vue";

import type { ImageOperationTargetOption } from "../contracts/generated/ImageOperationTargetOption";
import { imageSourceNode } from "./support/image-operation-fixtures";

const context = vi.hoisted(() => ({ space: null as null | Record<string, unknown> }));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.space! }));
vi.mock("../knowledge-derivations/image-operation", () => ({
  selectedImageTarget: (targets: readonly ImageOperationTargetOption[]) =>
    targets.find((target) => target.selected)!.execution,
  getImageOperationCapabilities: vi.fn<() => Promise<unknown>>().mockResolvedValue({
    targets: [
      {
        capability: "imageSelection",
        selected: true,
        execution: { type: "local", model: "efficientSamTi", specId: "spec" },
        availability: { state: "ready" },
      },
    ],
  }),
}));
vi.mock("../knowledge-derivations/image-selection-derivation", () => ({
  deriveImageSelection: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));
vi.mock("../knowledge-derivations/use-image-selection-preview", async () => {
  const { shallowRef } = await import("vue");
  return {
    useImageSelectionPreview: () => ({
      maskSource: shallowRef<string>("mask"),
      busy: shallowRef(false),
      error: shallowRef<string>(),
      queue: vi.fn<() => void>(),
      clear: vi.fn<() => void>(),
    }),
  };
});
import { getImageOperationCapabilities } from "../knowledge-derivations/image-operation";
import { deriveImageSelection } from "../knowledge-derivations/image-selection-derivation";
import { useImageSelectionSessionStore } from "../knowledge-derivations/image-selection-session-store";

beforeEach(() => {
  setActivePinia(createPinia());
  context.space = reactive({ activeSpaceId: "space", nodes: [imageSourceNode()] });
});
it("undoes points, boxes and brush refinements in the order the user made them", async () => {
  const store = useImageSelectionSessionStore();
  expect(store.canUndo).toBe(false);
  await store.begin(imageSourceNode());
  store.addPoint(0.3, 0.4);
  expect(store.canUndo).toBe(true);
  store.setBox({ x: 0.1, y: 0.1, width: 0.6, height: 0.6 });
  store.tool = "brush";
  store.addStroke([0.2, 0.3, 0.4, 0.5]);
  store.tool = "exclude";
  store.addPoint(0.7, 0.8);
  store.undo();
  expect(store.active?.selection).toMatchObject({
    points: [{ x: 0.3, y: 0.4 }],
    strokes: [expect.anything()],
  });
  store.undo();
  expect(store.active?.selection.strokes).toEqual([]);
  expect(store.active?.selection.boxRect).not.toBeNull();
  store.undo();
  expect(store.active?.selection.boxRect).toBeNull();
  expect(store.active?.selection.points).toHaveLength(1);
  store.undo();
  expect(store.active?.selection.points).toEqual([]);
  expect(store.canUndo).toBe(false);
});

it("discards a stale draft after an in-flight derivation fails", async () => {
  const store = useImageSelectionSessionStore();
  const node = imageSourceNode();
  context.space!.nodes = [node];
  await store.begin(node);
  let rejectDerivation!: (cause: Error) => void;
  vi.mocked(deriveImageSelection).mockImplementationOnce(
    () =>
      new Promise((_resolve, reject) => {
        rejectDerivation = reject;
      }),
  );
  const submission = store.submit("extract").catch((cause: unknown) => cause);
  node.data.sourceRevision = "revision-2";
  context.space!.nodes = [node];
  await nextTick();
  rejectDerivation(new Error("Source changed"));
  expect(await submission).toMatchObject({ message: "Source changed" });
  await nextTick();
  expect(store.active).toBeUndefined();
});

it("keeps prompts entered while execution choices load", async () => {
  const store = useImageSelectionSessionStore();
  const capabilities = await getImageOperationCapabilities();
  let finishLoading!: () => void;
  vi.mocked(getImageOperationCapabilities).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finishLoading = () => resolve(capabilities);
      }),
  );
  const opening = store.begin(imageSourceNode());
  store.addPoint(0.3, 0.4);
  finishLoading();
  await opening;
  expect(store.active?.execution).toMatchObject({ model: "efficientSamTi" });
  expect(store.active?.selection.points).toHaveLength(1);
});

it("ignores choices returned for a closed session when the same node reopens", async () => {
  const store = useImageSelectionSessionStore();
  const capabilities = structuredClone(await getImageOperationCapabilities());
  capabilities.targets[0]!.execution = { type: "local", model: "efficientSamS", specId: "old-session" };
  let finishLoading!: () => void;
  vi.mocked(getImageOperationCapabilities).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finishLoading = () => resolve(capabilities);
      }),
  );
  const opening = store.begin(imageSourceNode());
  store.cancel();
  await store.begin(imageSourceNode());
  finishLoading();
  await opening;
  expect(store.active?.execution).toMatchObject({ model: "efficientSamTi", specId: "spec" });
});
it("discards the draft and its undo history when the source revision changes", async () => {
  const store = useImageSelectionSessionStore();
  const node = imageSourceNode();
  context.space!.nodes = [node];
  await store.begin(node);
  store.addPoint(0.3, 0.4);
  node.data.sourceRevision = "revision-2";
  context.space!.nodes = [node];
  await nextTick();
  expect(store.active).toBeUndefined();
  await store.begin(node);
  store.undo();
  expect(store.active?.selection.points).toEqual([]);
});
