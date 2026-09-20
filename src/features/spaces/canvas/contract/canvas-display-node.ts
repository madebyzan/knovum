import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
export interface CanvasContainerDisplay {
  id: string;
  data:
    | { kind: "group"; title: string; collapsed: false; contentCount: number }
    | { kind: "section"; title: string; collapsed: boolean; contentCount: number };
}
/** Containers are structural handles, never persistable knowledge content. */
export type CanvasDisplayNode = SpaceCanvasNode | CanvasContainerDisplay;
