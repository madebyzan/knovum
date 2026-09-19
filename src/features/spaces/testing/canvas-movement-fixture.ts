import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { canvasNote } from "./canvas-fixtures";

export function movementScene(): SpaceCanvasNode[] {
  const a = canvasNote("a");
  const b = canvasNote("b");
  a.data = { ...a.data, sectionId: "s1" };
  b.data = { ...b.data, sectionId: "s2" };
  a.initialPosition = { x: 40, y: 50 };
  b.initialPosition = { x: 1_080, y: 250 };
  return [
    { id: "s1", type: "section", data: { kind: "section", title: "A" }, initialPosition: { x: 0, y: 0 } },
    {
      id: "s2",
      type: "section",
      data: { kind: "section", title: "B" },
      initialPosition: { x: 1_000, y: 200 },
    },
    a,
    b,
  ];
}
