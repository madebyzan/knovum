import Konva from "konva";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { effectScope, nextTick, shallowRef } from "vue";

import { createSpaceNotePayload } from "../../../document-editing/tiptap-content";
import { createSpaceNode } from "../../node";
import type { CanvasNodeLayoutV1 } from "../../schema/document";
import { useCanvasOverviewLayer } from "../rendering/use-canvas-overview-layer";
import type { CanvasPalette } from "../rendering/use-canvas-palette";
import type { CanvasSceneNode } from "../scene/canvas-scene";
import { projectCanvasScene } from "../scene/scene-projection";

const color = "#888888";
const palette: CanvasPalette = {
  typography: { fontFamily: "sans-serif", labelSize: 12, captionSize: 11 },
  accent: color,
  border: color,
  borderStrong: color,
  canvas: color,
  foreground: color,
  foregroundSecondary: color,
  grid: color,
  measurement: color,
  contentFillByPreset: { yellow: color, green: color, blue: color, pink: color, gray: color },
  panel: color,
  relationByColor: { neutral: color, blue: color, green: color, amber: color, red: color, purple: color },
};

/** Konva needs a 2D context; drawing is not under test, so every call is inert. */
const inertContext = {
  canvas: {},
  measureText: () => ({ width: 0 }),
  getImageData: () => ({ data: new Uint8ClampedArray(4) }),
};
beforeEach(() => {
  const context = new Proxy({} as CanvasRenderingContext2D, {
    get: (_target, property) => inertContext[property as keyof typeof inertContext] ?? (() => undefined),
    set: () => true,
  });
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(context);
});

function notes(...nodeIds: string[]): readonly CanvasSceneNode[] {
  return projectCanvasScene({
    nodes: nodeIds.map((id) => createSpaceNode(createSpaceNotePayload(id, `${id} text`), { x: 0, y: 0 }, id)),
    relations: [],
    document: {
      version: 1,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: nodeIds.map((nodeId, index) => ({
        nodeId,
        frame: { x: index * 300, y: 0, width: 200, height: 160, rotation: 0 },
        sizingMode: "frame",
        zIndex: index,
      })),
    },
  }).nodes;
}

function drawOverview(initial: readonly CanvasSceneNode[], zoom = 0.2) {
  const layer = new Konva.Layer();
  const nodes = shallowRef<readonly CanvasSceneNode[]>(initial);
  const previews = shallowRef<ReadonlyMap<string, CanvasNodeLayoutV1>>(new Map());
  const scale = shallowRef(zoom);
  const scope = effectScope();
  scope.run(() =>
    useCanvasOverviewLayer({
      layer: shallowRef(layer),
      nodes,
      layoutPreviewByNodeId: () => previews.value,
      palette: shallowRef(palette),
      zoom: scale,
    }),
  );
  onTestFinished(() => {
    scope.stop();
    layer.destroy();
  });
  const drawing = (index: number) => (layer.getChildren()[index] as Konva.Group).getChildren()[0];
  return { layer, nodes, previews, scale, drawing };
}

it("draws each overview node at its frame in paint order", async () => {
  const { layer } = drawOverview(notes("first", "second"));
  await nextTick();

  expect(layer.getChildren().map((group) => group.position())).toEqual([
    { x: 0, y: 0 },
    { x: 300, y: 0 },
  ]);
});

it("moves a node with its layout preview without redrawing it", async () => {
  const scene = notes("dragged");
  const { layer, previews, drawing } = drawOverview(scene);
  await nextTick();
  const drawn = drawing(0);

  previews.value = new Map([
    ["dragged", { ...scene[0]!.layout, frame: { ...scene[0]!.layout.frame, x: 40, y: 25 } }],
  ]);
  await nextTick();

  expect(layer.getChildren()[0]!.position()).toEqual({ x: 40, y: 25 });
  expect(drawing(0)).toBe(drawn);
});

it("redraws a node only when what its drawing shows changes", async () => {
  const { nodes, scale, drawing } = drawOverview(notes("note"));
  await nextTick();
  const drawn = drawing(0);

  scale.value = 0.25;
  await nextTick();
  expect(drawing(0)).toBe(drawn);

  scale.value = 0.01;
  await nextTick();
  expect(drawing(0)).not.toBe(drawn);

  const relabelled = drawing(0);
  nodes.value = notes("note");
  await nextTick();
  expect(drawing(0)).not.toBe(relabelled);
});

it("drops a node once it mounts its content", async () => {
  const { layer, nodes } = drawOverview(notes("first", "second"));
  await nextTick();

  nodes.value = nodes.value.slice(1);
  await nextTick();

  expect(layer.getChildren()).toHaveLength(1);
  expect(layer.getChildren()[0]!.position()).toEqual({ x: 300, y: 0 });
});
