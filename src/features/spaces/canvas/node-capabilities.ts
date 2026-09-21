import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceEditorPayload } from "../model/nodes/capabilities";

export type SpaceEditorNode = SpaceCanvasNode & { data: SpaceEditorPayload };

export function isSpaceEditorNode(node: SpaceCanvasNode | undefined): node is SpaceEditorNode {
  return node?.data.kind === "note" || node?.data.kind === "markdown";
}
