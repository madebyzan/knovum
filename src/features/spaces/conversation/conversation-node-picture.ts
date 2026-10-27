import { spaceCanvasOverviewSource } from "../assets/node-preview-source";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

/**
 * The picture of a canvas node wherever the conversation shows it — waiting in the composer, sent
 * with a message, or added by a reply: the node's current overview, so all three agree and follow
 * the node. Empty when the node has no picture or is no longer on the canvas.
 */
export function conversationNodePicture(node: SpaceCanvasNode | undefined): string {
  return node ? spaceCanvasOverviewSource(node.data) : "";
}
