import { describe, expect, it, vi } from "vitest";

import {
  ConnectorGestureController,
  type CanvasConnectorDraft,
} from "../canvas/konva/interaction/connector-gesture-controller";

const layouts = [
  {
    nodeId: "source",
    frame: { x: 10, y: 20, width: 100, height: 80, rotation: 0 },
    sizeAuthority: "frame",
    zIndex: 0,
  },
  {
    nodeId: "target",
    frame: { x: 220, y: 20, width: 120, height: 80, rotation: 0 },
    sizeAuthority: "frame",
    zIndex: 1,
  },
] as const;

describe("canvas connector gesture", () => {
  it("creates a directional JSON Canvas connector when released over another node", () => {
    const changed = vi.fn<(draft?: CanvasConnectorDraft) => void>();
    const controller = new ConnectorGestureController(() => layouts, changed);

    controller.begin("source", "right");
    controller.move({ x: 225, y: 60 });
    const connector = controller.finish({ x: 225, y: 60 });

    expect(connector).toMatchObject({
      sourceNodeId: "source",
      sourceSide: "right",
      sourceEnd: "none",
      targetNodeId: "target",
      targetSide: "left",
      targetEnd: "arrow",
    });
    expect(changed).toHaveBeenLastCalledWith();
  });

  it("does not create a connector when released on empty canvas", () => {
    const controller = new ConnectorGestureController(
      () => layouts,
      vi.fn<(draft?: CanvasConnectorDraft) => void>(),
    );
    controller.begin("source", "right");
    expect(controller.finish({ x: 150, y: 160 })).toBeUndefined();
  });

  it("magnetizes to the nearest target anchor using a screen-stable threshold", () => {
    const changed = vi.fn<(draft?: CanvasConnectorDraft) => void>();
    const controller = new ConnectorGestureController(
      () => layouts,
      changed,
      () => 2,
    );
    controller.begin("source", "right");
    controller.move({ x: 211.5, y: 60 });

    expect(changed).toHaveBeenLastCalledWith(
      expect.objectContaining({
        target: { nodeId: "target", side: "left", anchor: { x: 220, y: 60 } },
        to: { x: 220, y: 60 },
      }),
    );
  });

  it("clears a previous candidate when the pointer leaves its magnetic range", () => {
    const changed = vi.fn<(draft?: CanvasConnectorDraft) => void>();
    const controller = new ConnectorGestureController(() => layouts, changed);
    controller.begin("source", "right");
    controller.move({ x: 210, y: 60 });
    controller.move({ x: 160, y: 160 });

    const latest = changed.mock.calls.at(-1)?.[0];
    expect(latest).toMatchObject({ to: { x: 160, y: 160 } });
    expect(latest).not.toHaveProperty("target");
  });
});
