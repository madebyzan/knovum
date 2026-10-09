import { shallowMount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, reactive } from "vue";

import { createSpaceNode } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";

const context = vi.hoisted(() => ({
  space: null as null | {
    activeSpaceId: string;
    nodes: SpaceCanvasNode[];
    updateNodeContents: (...args: unknown[]) => Promise<void>;
  },
}));
const deriveImage = vi.hoisted(() =>
  vi.fn<
    (_space: unknown, _node: unknown, _operation: unknown, onOutputCreated?: () => void) => Promise<void>
  >((_space, _node, _operation, onOutputCreated) => {
    onOutputCreated?.();
    return Promise.resolve(undefined);
  }),
);
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.space! }));
vi.mock("../knowledge-derivations/image-operation", () => ({ deriveImage }));
import { useImageCommentSession } from "../comments/image-comment-session";
import { defaultImageEdit } from "../image-editing/image-edit";
import { useImageEditSession } from "../image-editing/image-edit-session";
import { useImageRemovalSession } from "../knowledge-derivations/image-removal-session";
import { useImageSourceDimensions } from "../presentation/canvas/image-source-dimensions";
import SpaceCanvasImageEditing from "../presentation/canvas/SpaceCanvasImageEditing.vue";
import SpaceImageTransformControls from "../presentation/canvas/SpaceImageTransformControls.vue";

beforeEach(() => {
  deriveImage.mockClear();
  setActivePinia(createPinia());
  context.space = reactive({
    activeSpaceId: "first",
    nodes: [
      createSpaceNode(
        validateSpaceNodePayload({
          kind: "image",
          title: "Image",
          fileName: "image.png",
          assetPath: "/managed/image.png",
        }),
        { x: 0, y: 0 },
        "same-id",
      ),
    ],
    updateNodeContents: vi.fn<(...args: unknown[]) => Promise<void>>().mockResolvedValue(undefined),
  });
});

describe("image session ownership", () => {
  it("does not reuse dimensions from an older source revision", () => {
    const dimensions = useImageSourceDimensions();
    dimensions.set("same-id", "revision-1", 100, 200);
    expect(dimensions.forNode("same-id", "revision-1")).toMatchObject({ width: 100, height: 200 });
    expect(dimensions.forNode("same-id", "revision-2")).toBeUndefined();
  });

  it("sends object removal to derivation without changing the source edit", async () => {
    const edits = useImageEditSession();
    const removal = useImageRemovalSession();
    const source = context.space!.nodes[0]!;
    if (source.data.kind !== "image") throw new Error("Expected an image fixture");
    removal.begin(source.id);
    removal.addStroke([0.5, 0.5, 0.6, 0.6]);
    expect(edits.active).toBeUndefined();
    await removal.generate();
    expect(deriveImage).toHaveBeenCalledWith(
      context.space,
      source,
      {
        kind: "remove-object",
        strokes: [{ diameter: 0.045, points: [0.5, 0.5, 0.6, 0.6] }],
      },
      expect.any(Function),
    );
    expect(removal.active).toBeUndefined();
    expect(context.space!.updateNodeContents).not.toHaveBeenCalled();
    expect(source.data.imageEdit).toBeUndefined();
  });

  it("keeps the selection when derivation preparation fails", async () => {
    const removal = useImageRemovalSession();
    removal.begin("same-id");
    removal.addStroke([0.5, 0.5, 0.6, 0.6]);
    deriveImage.mockRejectedValueOnce(new Error("Output path unavailable"));
    await expect(removal.generate()).rejects.toThrow("Output path unavailable");
    expect(removal.active?.strokes).toHaveLength(1);
    expect(removal.submitting).toBe(false);
  });

  it("bounds object-removal strokes before the derivation request", () => {
    const removal = useImageRemovalSession();
    removal.begin("same-id");
    for (let index = 0; index < 100; index++) removal.addStroke([0.5, 0.5, 0.6, 0.6]);
    expect(removal.canDraw).toBe(false);
    expect(() => removal.addStroke([0.5, 0.5, 0.6, 0.6])).toThrow("exceeds its limits");
    expect(removal.active?.strokes).toHaveLength(100);
  });

  it("renders the readonly edit draft and updates it without changing the saved image", async () => {
    const data = validateSpaceNodePayload({
      kind: "image",
      title: "Image",
      fileName: "image.png",
      assetPath: "/managed/image.png",
      imageEdit: defaultImageEdit(),
    });
    if (data.kind !== "image") throw new Error("Expected an image fixture");
    const node = { ...createSpaceNode(data, { x: 0, y: 0 }, "same-id"), data };
    context.space!.nodes = [node];
    const edits = useImageEditSession();
    edits.begin("same-id", "transform");
    const wrapper = shallowMount(SpaceCanvasImageEditing, {
      props: { node },
    });
    const transform = wrapper.findComponent(SpaceImageTransformControls);
    expect(transform.props("edit").rotation).toBe(0);
    transform.vm.$emit("update", { rotation: 90 });
    await nextTick();
    expect(transform.props("edit").rotation).toBe(90);
    expect(node.data.imageEdit?.rotation).toBe(0);
    expect(context.space!.updateNodeContents).not.toHaveBeenCalled();
    wrapper.unmount();
  });

  it("clears edits, comments, drafts and dimensions before another Space reuses the same node ID", async () => {
    const edits = useImageEditSession();
    const removal = useImageRemovalSession();
    const dimensions = useImageSourceDimensions();
    const comments = useImageCommentSession();
    dimensions.set("same-id", "revision-1", 100, 100);
    edits.begin("same-id", "crop");
    removal.begin("same-id");
    comments.begin("same-id");
    comments.addToConversation("same-id", {
      x: 0.5,
      y: 0.5,
      text: "old comment",
    });
    edits.cancel();
    context.space!.activeSpaceId = "second";
    expect(edits.active).toBeUndefined();
    expect(removal.active).toBeUndefined();
    expect(dimensions.forNode("same-id", "revision-1")).toBeUndefined();
    expect(comments.activeNodeId).toBeUndefined();
    expect(comments.takeConversationDraft()).toBeUndefined();
    await edits.apply();
    expect(context.space!.updateNodeContents).not.toHaveBeenCalled();
  });

  it("releases image state when its node is deleted", () => {
    const edits = useImageEditSession();
    const removal = useImageRemovalSession();
    const dimensions = useImageSourceDimensions();
    const comments = useImageCommentSession();
    dimensions.set("same-id", "revision-1", 100, 100);
    edits.begin("same-id", "adjust");
    removal.begin("same-id");
    comments.begin("same-id");
    comments.addToConversation("same-id", {
      x: 0.5,
      y: 0.5,
      text: "deleted comment",
    });
    context.space!.nodes = [];
    expect(edits.active).toBeUndefined();
    expect(removal.active).toBeUndefined();
    expect(dimensions.forNode("same-id", "revision-1")).toBeUndefined();
    expect(comments.activeNodeId).toBeUndefined();
    expect(comments.takeConversationDraft()).toBeUndefined();
  });
});
