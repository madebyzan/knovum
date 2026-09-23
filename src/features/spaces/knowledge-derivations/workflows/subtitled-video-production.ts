import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import { runWorkflowTemplate } from "../../workflows/template-client";

type Store = ReturnType<typeof useSpaceStore>;

export async function startSubtitledVideoWorkflow(store: Store, source: SpaceCanvasNode) {
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  return runWorkflowTemplate({
    spaceId,
    templateId: "subtitled-video-production",
    inputNodeIds: [source.id],
    parameters: {},
  });
}
