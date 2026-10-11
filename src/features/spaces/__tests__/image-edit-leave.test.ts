import { shallowMount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, onTestFinished, vi } from "vitest";
import { reactive } from "vue";

import { createSpaceNode } from "../canvas/node";
import { useImageEditSessionStore } from "../image-editing/image-edit-session-store";
import SpaceImageEditLeaveDialog from "../image-editing/presentation/SpaceImageEditLeaveDialog.vue";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import SpaceUnsavedChangesDialog from "../workspace/presentation/SpaceUnsavedChangesDialog.vue";
import {
  flushSpaceSessionCommits,
  flushSpaceSessionContent,
  registerSpaceSessionParticipant,
} from "../workspace/session-participants";

const context = vi.hoisted(() => ({
  space: null as null | {
    activeSpaceId: string;
    nodes: SpaceCanvasNode[];
    updateNodeContents: (...args: unknown[]) => Promise<void>;
  },
}));

vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.space! }));

beforeEach(() => {
  setActivePinia(createPinia());
  context.space = reactive({
    activeSpaceId: "space",
    nodes: [
      createSpaceNode(
        validateSpaceNodePayload({
          kind: "image",
          title: "Image",
          fileName: "image.png",
          assetPath: "/managed/image.png",
        }),
        { x: 0, y: 0 },
        "image",
      ),
    ],
    updateNodeContents: vi.fn<(...args: unknown[]) => Promise<void>>(),
  });
});

describe("image edit leave protection", () => {
  it("holds Space leave behind an admitted Apply and keeps the dirty edit when Apply fails", async () => {
    let rejectApply: ((reason: Error) => void) | undefined;
    const update = new Promise<void>((_resolve, reject) => {
      rejectApply = reject;
    });
    context.space!.updateNodeContents = vi.fn<(...args: unknown[]) => Promise<void>>(() => update);
    const edits = useImageEditSessionStore();
    edits.begin("image", "transform");
    edits.update({ rotation: 90 });
    const wrapper = shallowMount(SpaceImageEditLeaveDialog, {
      global: { mocks: { $t: (key: string) => key } },
    });
    const applying = edits.apply();
    const leaving = flushSpaceSessionContent();
    await Promise.resolve();

    expect(wrapper.findComponent(SpaceUnsavedChangesDialog).props("saving")).toBe(true);
    expect(wrapper.findComponent(SpaceUnsavedChangesDialog).props("open")).toBe(false);
    rejectApply?.(new Error("image database unavailable"));
    await expect(applying).rejects.toThrow("image database unavailable");
    await expect(leaving).rejects.toThrow("image database unavailable");
    expect(edits.dirty).toBe(true);
    expect(edits.active?.draft.rotation).toBe(90);
    wrapper.unmount();
  });

  it("keeps the image draft after accepting discard when later layout preparation fails", async () => {
    const edits = useImageEditSessionStore();
    edits.begin("image", "transform");
    edits.update({ rotation: 90 });
    const wrapper = shallowMount(SpaceImageEditLeaveDialog, {
      global: { mocks: { $t: (key: string) => key } },
    });
    onTestFinished(() => wrapper.unmount());
    const layoutFailure = new Error("layout unavailable");
    const detachLayout = registerSpaceSessionParticipant({
      phase: "layout",
      commit: async () => {
        throw layoutFailure;
      },
    });
    onTestFinished(detachLayout);

    const leaving = flushSpaceSessionCommits();
    await vi.waitFor(() => expect(wrapper.findComponent(SpaceUnsavedChangesDialog).props("open")).toBe(true));
    wrapper.findComponent(SpaceUnsavedChangesDialog).vm.$emit("discard");

    await expect(leaving).rejects.toBe(layoutFailure);
    expect(edits.dirty).toBe(true);
    expect(edits.active?.draft.rotation).toBe(90);
  });
});
