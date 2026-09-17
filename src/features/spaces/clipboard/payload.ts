import type { PositionedSpaceNode } from "../model/nodes/canvas-node";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import type { SpaceClipboardPayload } from "./payload-types";

export function createSpaceClipboardPayload(nodes: readonly PositionedSpaceNode[]): SpaceClipboardPayload {
  return {
    version: 1,
    nodes: nodes.map(({ node, position }) => ({
      id: node.id,
      position: { ...position },
      data: validateSpaceNodePayload(node.data),
    })),
  };
}
