import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";

import type { CanvasEditorPort } from "../canvas/contracts/canvas-editor-port";
import { registerCanvasEditor, runCanvasSessionTransition } from "../canvas/editor-session";
import { useCanvasInteractionCancellation } from "../canvas/konva/interaction/use-canvas-interaction-cancellation";

it("cancels transient gestures without deselecting or exiting the current Group on a cancelled switch", async () => {
  const selected = shallowRef(["node"]);
  const group = shallowRef(["group"]);
  const inlineEditor = shallowRef<string | undefined>("node");
  const cancelGesture = vi.fn<() => boolean>(() => true);
  let interaction!: ReturnType<typeof useCanvasInteractionCancellation>;
  const wrapper = mount({
    setup() {
      interaction = useCanvasInteractionCancellation({
        cancelConnector: () => {},
        cancelGesture,
        cancelNodeDrag: () => {},
        cancelPointer: () => {},
        cancelStructurePreview: () => {},
        clearSelection: () => {
          selected.value = [];
        },
        exitGroup: () => {
          group.value = [];
        },
        connectorDraft: shallowRef(),
        directManipulation: shallowRef(),
        enterableContentId: inlineEditor,
        groupScopePath: () => group.value,
        isInteractionEnabled: () => true,
        marquee: shallowRef(),
        panning: shallowRef(false),
        tool: () => "select",
        directManipulationChanged: () => {},
      });
      return () => null;
    },
  });
  onTestFinished(() => wrapper.unmount());
  onTestFinished(
    registerCanvasEditor({
      ...interaction,
      getViewport: () => ({ x: 0, y: 0, zoom: 1 }),
      getSelectionSnapshot: () => ({
        nodeIds: selected.value,
        structureNodeIds: [],
        relationIds: [],
        members: [],
        nodePlacements: [],
      }),
    } as unknown as CanvasEditorPort),
  );
  await expect(
    runCanvasSessionTransition(async () => {
      throw new Error("Save failed");
    }),
  ).rejects.toThrow("Save failed");
  expect(cancelGesture).toHaveBeenCalledOnce();
  expect(inlineEditor.value).toBeUndefined();
  expect(selected.value).toEqual(["node"]);
  expect(group.value).toEqual(["group"]);
});
