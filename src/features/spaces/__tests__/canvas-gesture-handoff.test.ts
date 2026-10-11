import { describe, expect, it } from "vitest";
import { nextTick } from "vue";

import { structureGestureFixture as fixture } from "./support/structure-gesture-fixture";

describe("structure gesture handoff", () => {
  it("keeps Section and descendant geometry at the drop position through asynchronous commit", async () => {
    const value = fixture();
    value.gestures.beginMove("section", value.node);
    value.node.position({ x: 120, y: 90 });
    value.gestures.move();
    await nextTick();
    const dropped = value.document.value;
    const droppedPoints = value.points.value;
    value.renderedPoints.length = 0;
    value.rendered.length = 0;
    const finished = value.gestures.finish();
    await value.started.promise;
    expect(value.preview.value).toBeDefined();
    await value.gestures.finish();
    expect(value.points.value).toEqual(droppedPoints);
    expect(value.gestures.directManipulation.value).toBe("drag");
    value.commit.resolve(undefined);
    await finished;
    await nextTick();
    expect(value.preview.value).toBeUndefined();
    expect(value.document.value).toEqual(dropped);
    expect(value.rendered.every((document) => JSON.stringify(document) === JSON.stringify(dropped))).toBe(
      true,
    );
    expect(value.commands).toHaveLength(1);
    expect(
      value.renderedPoints.every((points) => JSON.stringify(points) === JSON.stringify(droppedPoints)),
    ).toBe(true);
    expect(value.gestures.directManipulation.value).toBeUndefined();
  });

  it("restores the original geometry when persistence rejects the gesture", async () => {
    const value = fixture();
    const before = value.document.value;
    value.gestures.beginMove("section", value.node);
    value.node.position({ x: 120, y: 90 });
    value.gestures.move();
    const finished = value.gestures.finish();
    await value.started.promise;
    value.commit.resolve(new Error("edit rejected"));
    await expect(finished).rejects.toThrow("edit rejected");
    expect(value.preview.value).toBeUndefined();
    expect(value.document.value).toEqual(before);
    expect(value.node.position()).toEqual({ x: 0, y: 0 });
  });

  it("cancels before stopping native drag so its dragend cannot commit", async () => {
    const value = fixture();
    value.gestures.beginMove("section", value.node);
    value.node.position({ x: 120, y: 90 });
    value.gestures.move();
    expect(value.gestures.cancel()).toBe(true);
    await value.gestures.finish();
    expect(value.commands).toEqual([]);
    expect(value.preview.value).toBeUndefined();
    expect(value.node.position()).toEqual({ x: 0, y: 0 });
  });

  it("does not persist a native drag that ends without a layout change", async () => {
    const value = fixture();
    value.gestures.beginMove("section", value.node);
    value.gestures.move();
    await value.gestures.finish();
    expect(value.commands).toEqual([]);
    expect(value.preview.value).toBeUndefined();
  });

  it("nudges the selected Section root instead of requiring selected content ids", async () => {
    const value = fixture();
    value.commit.resolve(undefined);
    await value.gestures.nudge({ x: 1, y: 0 });
    expect(value.commands).toEqual([{ kind: "structure.move", rootIds: ["section"], delta: { x: 1, y: 0 } }]);
    expect(value.document.value.nodes[0]?.frame.x).toBe(41);
  });
});
