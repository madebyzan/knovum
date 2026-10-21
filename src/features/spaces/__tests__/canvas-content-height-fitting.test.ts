import { describe, expect, it, vi } from "vitest";

import { canvasNodeInteraction } from "../canvas/canvas-node-interaction";
import { createCanvasContentHeightFitting } from "../canvas/konva/content-height-fitting";
import type { CanvasScene, CanvasSceneNode } from "../canvas/konva/scene/canvas-scene";
import { createSpaceNode } from "../canvas/node";
import type { SpaceEditCommand } from "../contracts/generated/SpaceEditCommand";
import { createSpaceTextPayload } from "../document-editing/tiptap-content";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";

function fixture(count: number) {
  const snapshot: SpaceEditSnapshot = { editRevision: 0, structure: [], layouts: [], relationStyles: [] };
  const nodes = new Map<string, CanvasSceneNode>();
  for (let index = 0; index < count; index++) {
    const id = String(index);
    const node = createSpaceNode(createSpaceTextPayload("Text"), { x: 0, y: 0 }, id);
    snapshot.structure.push({
      kind: "content",
      id: `s${id}`,
      nodeId: id,
      parentId: null,
      order: index,
      locked: false,
    });
    snapshot.layouts.push({
      kind: "content",
      structureNodeId: `s${id}`,
      transform: { x: 0, y: 0, rotation: 0, scale: 1 },
      size: { width: 240, height: 100 },
      sizingMode: "frame",
    });
    nodes.set(id, {
      node,
      paintOrder: index,
      interaction: canvasNodeInteraction(node),
      layout: {
        nodeId: id,
        frame: { x: 0, y: 0, width: 240, height: 100, rotation: 0 },
        sizingMode: "frame",
        zIndex: index,
      },
    });
  }
  const scene: CanvasScene = {
    nodes: [...nodes.values()],
    nodesById: nodes,
    relations: [],
    relationKeyByRelationId: new Map(),
    sectionBackdrops: [],
  };
  const execute = vi.fn<(command: SpaceEditCommand) => Promise<void>>(async () => {});
  const capture = vi.fn<() => SpaceEditSnapshot>(() => snapshot);
  const onError = vi.fn<(error: unknown) => void>();
  const fitting = createCanvasContentHeightFitting({
    scene: () => scene,
    snapshot: capture,
    nodeById: (nodeId) => nodes.get(nodeId)?.node,
    onExecuteSpaceEdit: execute,
    onError,
  });
  const measurement = {
    contentRevision: null,
    height: 120,
    sizingMode: "frame" as const,
    typographyScale: 1,
    width: 240,
  };
  return { capture, execute, fitting, measurement, nodes, onError, snapshot };
}

describe("canvas content height fitting", () => {
  it("submits a large batch from one snapshot without scanning arrays for each node", async () => {
    const value = fixture(2000);
    const scan = vi.spyOn(value.snapshot.layouts, "find");
    for (let index = 1800; index < 2000; index++) value.fitting.fit(String(index), value.measurement);
    await vi.waitFor(() => expect(value.execute).toHaveBeenCalledOnce());
    expect(value.capture).toHaveBeenCalledOnce();
    expect(scan).not.toHaveBeenCalled();
    expect(value.execute).toHaveBeenCalledWith({
      kind: "content.update-heights",
      updates: Array.from({ length: 200 }, (_, index) => ({
        structureNodeId: `s${index + 1800}`,
        contentRevision: null,
        height: 120,
        measuredWidth: 240,
        sizingMode: "frame",
        typographyScale: 1,
      })),
    });
  });

  it("discards a queued measurement when its source changes before submission", async () => {
    const value = fixture(1);
    value.fitting.fit("0", value.measurement);
    await Promise.resolve();
    const layout = value.snapshot.layouts[0]!;
    if (layout.kind === "content") layout.size.width = 300;
    await Promise.resolve();
    expect(value.execute).not.toHaveBeenCalled();
  });

  it("does not rebuild a snapshot for an empty or rejected batch", async () => {
    const value = fixture(1);
    value.fitting.flush();
    value.fitting.fit("0", { ...value.measurement, width: 300 });
    await Promise.resolve();
    expect(value.capture).not.toHaveBeenCalled();
    expect(value.execute).not.toHaveBeenCalled();
  });

  it.each([0.3, 2])(
    "commits scale %s measurements in local units and skips unchanged heights",
    async (scale) => {
      const value = fixture(1);
      const layout = value.snapshot.layouts[0]!;
      if (layout.kind !== "content") throw new Error("Expected content fixture");
      layout.transform.scale = scale;
      const frame = value.nodes.get("0")!.layout.frame;
      frame.width = 240 * scale;
      frame.height = 100 * scale;
      const measurement = { ...value.measurement, width: 240 * scale, height: 220 * scale };
      value.fitting.fit("0", measurement);
      await vi.waitFor(() => expect(value.execute).toHaveBeenCalledOnce());
      expect(value.execute).toHaveBeenCalledWith({
        kind: "content.update-heights",
        updates: [
          {
            contentRevision: null,
            sizingMode: "frame",
            typographyScale: 1,
            structureNodeId: "s0",
            height: 220,
            measuredWidth: 240,
          },
        ],
      });
      layout.size.height = 220;
      frame.height = 220 * scale;
      value.fitting.fit("0", measurement);
      value.fitting.flush();
      await Promise.resolve();
      expect(value.execute).toHaveBeenCalledOnce();
    },
  );

  it("discards a world measurement after its local transform changes", async () => {
    const value = fixture(1);
    value.fitting.fit("0", value.measurement);
    await Promise.resolve();
    const layout = value.snapshot.layouts[0]!;
    if (layout.kind === "content") layout.transform.scale = 2;
    await Promise.resolve();
    expect(value.execute).not.toHaveBeenCalled();
  });

  it("drops a node removed between measurement and submission", async () => {
    const value = fixture(1);
    value.fitting.fit("0", value.measurement);
    await Promise.resolve();
    value.nodes.delete("0");
    await Promise.resolve();
    expect(value.execute).not.toHaveBeenCalled();
  });

  it("serializes submissions and reports a failed edit before accepting the next one", async () => {
    const value = fixture(1);
    const error = new Error("edit rejected");
    let rejectSubmission!: (reason: Error) => void;
    value.execute.mockImplementationOnce(
      () =>
        new Promise<void>((_resolve, reject) => {
          rejectSubmission = reject;
        }),
    );
    value.fitting.fit("0", value.measurement);
    await vi.waitFor(() => expect(value.execute).toHaveBeenCalledTimes(1));
    value.fitting.fit("0", { ...value.measurement, height: 140 });
    value.fitting.flush();
    await Promise.resolve();
    expect(value.execute).toHaveBeenCalledTimes(1);
    rejectSubmission(error);
    await vi.waitFor(() => expect(value.execute).toHaveBeenCalledTimes(2));
    expect(value.onError).toHaveBeenCalledWith(error);
  });
});
