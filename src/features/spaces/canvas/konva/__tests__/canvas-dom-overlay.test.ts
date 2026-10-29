import { mount } from "@vue/test-utils";
import { expect, it } from "vitest";
import { h } from "vue";

import { createSpaceNotePayload } from "../../../document-editing/tiptap-content";
import { createSpaceNode } from "../../node";
import CanvasDomOverlay from "../rendering/CanvasDomOverlay.vue";
import { projectCanvasScene } from "../scene/scene-projection";
import { createCanvasCamera } from "../viewport/canvas-camera";

const scene = projectCanvasScene({
  nodes: [createSpaceNode(createSpaceNotePayload("note"), { x: 0, y: 0 }, "note")],
  relations: [],
  document: {
    version: 1,
    viewport: { x: 0, y: 0, zoom: 1 },
    nodes: [
      {
        nodeId: "note",
        frame: { x: 0, y: 0, width: 100, height: 80, rotation: 0 },
        sizingMode: "frame",
        zIndex: 0,
      },
    ],
  },
});

it("keeps the cards of a covered canvas mounted, so returning to it rebuilds nothing", async () => {
  const wrapper = mount(CanvasDomOverlay, {
    props: {
      camera: createCanvasCamera({ x: 0, y: 0, zoom: 1 }),
      scene,
      interactionEnabled: true,
      renderingEnabled: true,
      layoutPreviewByNodeId: new Map(),
      interactiveNodeId: undefined,
      visibleNodeIds: new Set(["note"]),
      selectedNodeIds: new Set<string>(),
      derivedOutputCountByNodeId: new Map(),
      sourceUpdateCountByNodeId: new Map(),
      zoom: 1,
    },
    slots: { default: ({ node }: { node: { id: string } }) => h("p", { "data-card": node.id }) },
  });
  const card = wrapper.get("[data-card='note']").element;

  await wrapper.setProps({ renderingEnabled: false, interactionEnabled: false });

  expect(wrapper.get("[data-card='note']").element).toBe(card);
  expect(wrapper.attributes("inert")).toBeDefined();
  wrapper.unmount();
});
