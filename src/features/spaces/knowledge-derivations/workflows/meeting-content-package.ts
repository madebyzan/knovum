import type { ExecutionTarget } from "../../../../foundation/ai/schema/models";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import { startBuiltinWorkflow } from "../../workflows/run-builtin-workflow";

type Store = ReturnType<typeof useSpaceStore>;
export interface MeetingContentPackageDraft {
  summaryTitle: string;
  summaryPurpose: string;
  textTarget?: ExecutionTarget;
}

export async function createMeetingContentPackage(
  store: Store,
  source: SpaceCanvasNode,
  draft: MeetingContentPackageDraft,
) {
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  return startBuiltinWorkflow({
    spaceId,
    templateId: "meeting-content-package",
    inputNodeIds: [source.id],
    parameters: { summary: { title: draft.summaryTitle, summaryPurpose: draft.summaryPurpose } },
    targets: { "summarize-meeting": draft.textTarget },
  });
}
