import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceEditorPayload } from "../model/nodes/capabilities";
import type { SpaceSectionPayload } from "../model/nodes/node-payload";

export type SpaceSectionNode = SpaceCanvasNode & { data: SpaceSectionPayload };
export type SpaceEditorNode = SpaceCanvasNode & { data: SpaceEditorPayload };

export function isSpaceSectionNode(node: SpaceCanvasNode | undefined): node is SpaceSectionNode {
  return node?.data.kind === "section";
}

export function isSpaceEditorNode(node: SpaceCanvasNode | undefined): node is SpaceEditorNode {
  return node?.data.kind === "note" || node?.data.kind === "markdown";
}
