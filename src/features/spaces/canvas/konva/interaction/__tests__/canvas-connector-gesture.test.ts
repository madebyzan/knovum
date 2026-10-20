import { describe, expect, it, vi } from "vitest";

import { ConnectorGestureController, type CanvasConnectorDraft } from "../connector-gesture-controller";
import { createConnectorTargetIndex } from "../connector-target-index";

const layouts = [
  {
    nodeId: "source",
    frame: { x: 10, y: 20, width: 100, height: 80, rotation: 0 },
    sizingMode: "frame",
    zIndex: 0,
  },
  {
    nodeId: "target",
    frame: { x: 220, y: 20, width: 120, height: 80, rotation: 0 },
    sizingMode: "frame",
    zIndex: 1,
  },
] as const;
const targets = () => createConnectorTargetIndex(layouts);
const stacked = [
  {
    nodeId: "below",
    frame: { x: 0, y: 0, width: 200, height: 200, rotation: 0 },
    sizingMode: "frame",
    zIndex: 0,
  },
  {
    nodeId: "middle",
    frame: { x: 10, y: 10, width: 200, height: 200, rotation: 0 },
    sizingMode: "frame",
    zIndex: 1,
  },
  {
    nodeId: "above",
    frame: { x: 20, y: 20, width: 200, height: 200, rotation: 0 },
    sizingMode: "frame",
    zIndex: 2,
  },
] as const;
const tilted = [
  {
    nodeId: "source",
    frame: { x: 500, y: 500, width: 40, height: 40, rotation: 0 },
    sizingMode: "frame",
    zIndex: 0,
  },
  {
    nodeId: "tilted",
    frame: { x: 0, y: 0, width: 200, height: 20, rotation: 45 },
    sizingMode: "frame",
    zIndex: 1,
  },
] as const;
const draftRecorder = () => vi.fn<(draft?: CanvasConnectorDraft) => void>();

describe("canvas connector gesture", () => {
  it("creates a directional JSON Canvas connector when released over another node", () => {
    const changed = draftRecorder();
    const controller = new ConnectorGestureController(targets, changed);

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
    const controller = new ConnectorGestureController(targets, draftRecorder());
    controller.begin("source", "right");
    expect(controller.finish({ x: 150, y: 160 })).toBeUndefined();
  });

  it("magnetizes to the nearest target anchor using a screen-stable threshold", () => {
    const changed = draftRecorder();
    const controller = new ConnectorGestureController(targets, changed, () => 2);
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
    const changed = draftRecorder();
    const controller = new ConnectorGestureController(targets, changed);
    controller.begin("source", "right");
    controller.move({ x: 210, y: 60 });
    controller.move({ x: 160, y: 160 });

    const latest = changed.mock.calls.at(-1)?.[0];
    expect(latest).toMatchObject({ to: { x: 160, y: 160 } });
    expect(latest).not.toHaveProperty("target");
  });

  it("resolves the topmost node when several overlap the pointer", () => {
    const changed = draftRecorder();
    const controller = new ConnectorGestureController(() => createConnectorTargetIndex(stacked), changed);

    controller.begin("below", "right");
    controller.move({ x: 100, y: 100 });

    expect(changed.mock.calls.at(-1)?.[0]?.target).toMatchObject({ nodeId: "above" });
  });

  it("targets a rotated frame the pointer is really inside", () => {
    const changed = draftRecorder();
    const controller = new ConnectorGestureController(() => createConnectorTargetIndex(tilted), changed);

    controller.begin("source", "right");
    controller.move({ x: 70, y: 70 });

    expect(changed.mock.calls.at(-1)?.[0]?.target).toMatchObject({ nodeId: "tilted", side: "top" });
  });

  it("ignores a rotated frame whose bounds cover the pointer but whose body does not", () => {
    const changed = draftRecorder();
    const controller = new ConnectorGestureController(() => createConnectorTargetIndex(tilted), changed);

    controller.begin("source", "right");
    controller.move({ x: 5, y: 100 });

    expect(changed.mock.calls.at(-1)?.[0]).not.toHaveProperty("target");
  });
});
