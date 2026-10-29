import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import { createSpaceTextPayload } from "../../../../document-editing/tiptap-content";
import { canvasNodeInteraction } from "../../../canvas-node-interaction";
import { createSpaceNode } from "../../../node";
import { canvasPreviewPolicy } from "../../../preview/policy";
import CanvasDomNodeOverlay from "../CanvasDomNodeOverlay.vue";

function overlay(props: {
  selected: boolean;
  interactive: boolean;
  derivedOutputs: number;
  sourceUpdates?: number;
}) {
  const node = createSpaceNode(createSpaceTextPayload("source"), { x: 0, y: 0 }, "source");
  return mount(CanvasDomNodeOverlay, {
    props: {
      sourceUpdates: 0,
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

const badge = (wrapper: ReturnType<typeof overlay>) => wrapper.find('[data-badge="source-updates"]');

describe("source updates badge", () => {
  it("counts changed sources, and stays while the node is selected so the inspector can be found", () => {
    expect(
      badge(overlay({ selected: false, interactive: false, derivedOutputs: 0, sourceUpdates: 2 })).text(),
    ).toBe("2");
    const selected = overlay({ selected: true, interactive: false, derivedOutputs: 3, sourceUpdates: 1 });
    expect(badge(selected).text()).toBe("1");
    expect(selected.find('[data-badge="derived-outputs"]').exists()).toBe(false);
  });

  it("leaves the node being edited alone", () => {
    expect(overlay({ selected: false, interactive: true, derivedOutputs: 0, sourceUpdates: 2 }).text()).toBe(
      "",
    );
  });
});
