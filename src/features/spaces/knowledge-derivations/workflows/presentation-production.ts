import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import type { useSpaceStore } from "../../session/space-store";
import { startBuiltinWorkflow } from "../../workflows/run-builtin-workflow";
import type { GroundedDocumentSource } from "../catalog/grounded-document";

type Store = ReturnType<typeof useSpaceStore>;
export interface PresentationGenerationDraft {
  title: string;
  purpose: string;
  textTarget?: ExecutionTarget;
}

export async function generatePresentationFromSources(options: {
  store: Store;
  sources: readonly GroundedDocumentSource[];
  draft: PresentationGenerationDraft;
}) {
  const spaceId = options.store.activeSpaceId;
  if (!spaceId) return;
  return startBuiltinWorkflow({
    spaceId,
    templateId: "presentation-production",
    inputNodeIds: options.sources.map(({ nodeId }) => nodeId),
    parameters: { document: { title: options.draft.title, purpose: options.draft.purpose } },
    targets: { "draft-document": options.draft.textTarget },
  });
}
