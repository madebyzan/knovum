import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { CreationWorkflowStepOutput } from "./workflow";

export function nodeStepOutput(node: SpaceCanvasNode): CreationWorkflowStepOutput<SpaceCanvasNode> {
  return { output: node, outputNodeIds: [node.id] };
}
