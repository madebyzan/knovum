import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import type { NodePayloadChange } from "../model/nodes/payload-change";
import type { VideoEditProject } from "./project";

export type UpdateNodeContents = (changes: readonly NodePayloadChange[]) => Promise<void>;

export async function saveVideoEditProject(
  updateNodeContents: UpdateNodeContents,
  node: SpaceCanvasNode,
  payload: SpaceVideoPayload,
  project: VideoEditProject,
) {
  const after: SpaceVideoPayload = { ...payload, videoEditProject: project };
  await updateNodeContents([{ id: node.id, before: payload, after }]);
}
