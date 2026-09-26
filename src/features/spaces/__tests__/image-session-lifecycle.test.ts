import { shallowMount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, reactive } from "vue";

import { createSpaceNode } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";

const context = vi.hoisted(() => ({
  space: null as null | {
    activeSpaceId: string;
    nodes: SpaceCanvasNode[];
    updateManyData: (...args: unknown[]) => Promise<void>;
  },
}));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.space! }));
import { useImageCommentSession } from "../comments/image-comment-session";
import { defaultImageEdit } from "../image-editing/image-edit";
import { useImageEditSession } from "../image-editing/image-edit-session";
import SpaceCanvasImageActivePanel from "../presentation/canvas/SpaceCanvasImageActivePanel.vue";
import SpaceImageTransformPanel from "../presentation/canvas/SpaceImageTransformPanel.vue";

beforeEach(() => {
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
    updateManyData: vi.fn<(...args: unknown[]) => Promise<void>>().mockResolvedValue(undefined),
  });
});

describe("image session ownership", () => {
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
    const wrapper = shallowMount(SpaceCanvasImageActivePanel, {
      props: { node },
    });
    const transform = wrapper.findComponent(SpaceImageTransformPanel);
    expect(transform.props("edit").rotation).toBe(0);
    transform.vm.$emit("update", { rotation: 90 });
    await nextTick();
    expect(transform.props("edit").rotation).toBe(90);
    expect(node.data.imageEdit?.rotation).toBe(0);
    expect(context.space!.updateManyData).not.toHaveBeenCalled();
    wrapper.unmount();
  });

  it("clears edits, comments, drafts and dimensions before another Space reuses the same node ID", async () => {
    const edits = useImageEditSession();
    const comments = useImageCommentSession();
    edits.setImageDimensions("same-id", 100, 100);
    edits.begin("same-id", "crop");
    comments.begin("same-id");
    comments.addToConversation("same-id", "old comment");
    context.space!.activeSpaceId = "second";
    expect(edits.active).toBeUndefined();
    expect(edits.dimensionsFor("same-id")).toBeUndefined();
    expect(comments.activeNodeId).toBeUndefined();
    expect(comments.takeConversationDraft()).toBeUndefined();
    await edits.apply();
    expect(context.space!.updateManyData).not.toHaveBeenCalled();
  });

  it("releases image state when its node is deleted", () => {
    const edits = useImageEditSession();
    const comments = useImageCommentSession();
    edits.setImageDimensions("same-id", 100, 100);
    edits.begin("same-id", "adjust");
    comments.begin("same-id");
    comments.addToConversation("same-id", "deleted comment");
    context.space!.nodes = [];
    expect(edits.active).toBeUndefined();
    expect(edits.dimensionsFor("same-id")).toBeUndefined();
    expect(comments.activeNodeId).toBeUndefined();
    expect(comments.takeConversationDraft()).toBeUndefined();
  });
});
