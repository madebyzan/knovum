import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import { createSpaceTextPayload } from "../../../../document-editing/tiptap-content";
import { canvasNodeInteraction } from "../../../canvas-node-interaction";
import { createSpaceNode } from "../../../node";
import { canvasPreviewPolicy } from "../../../preview/policy";
import CanvasDomNodeOverlay from "../CanvasDomNodeOverlay.vue";

function overlay(props: { selected: boolean; interactive: boolean; derivedOutputs: number }) {
  const node = createSpaceNode(createSpaceTextPayload("source"), { x: 0, y: 0 }, "source");
  return mount(CanvasDomNodeOverlay, {
    props: {
      ...props,
      previewPolicy: canvasPreviewPolicy(100, 1),
      layoutPreviewByNodeId: new Map(),
      sceneNode: {
        node,
        layout: {
          nodeId: "source",
          frame: { x: 0, y: 0, width: 240, height: 160, rotation: 0 },
          sizingMode: "frame",
          zIndex: 0,
        },
        interaction: canvasNodeInteraction(node, false),
        paintOrder: 0,
      },
    },
  });
}

describe("derived outputs badge", () => {
  it("counts the outputs of a source at rest", () => {
    expect(overlay({ selected: false, interactive: false, derivedOutputs: 8 }).text()).toBe("8");
  });

  it("gives way to the lineage itself once the source is selected or being edited", () => {
    expect(overlay({ selected: true, interactive: false, derivedOutputs: 8 }).text()).toBe("");
    expect(overlay({ selected: false, interactive: true, derivedOutputs: 8 }).text()).toBe("");
  });

  it("stays off content nothing was derived from", () => {
    expect(overlay({ selected: false, interactive: false, derivedOutputs: 0 }).text()).toBe("");
  });
});
