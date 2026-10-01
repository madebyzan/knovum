import type { ExecutionTarget } from "../../../foundation/ai/schema/models";
import { i18n } from "../../../foundation/i18n/i18n";
import type { useSpaceStore } from "../session/space-store";
import type { GroundedDocumentSource } from "./catalog/grounded-document";
import { startDerivationAction } from "./start-action";

export interface GroundedDocumentDraft {
  title: string;
  purpose: string;
  target?: ExecutionTarget | undefined;
}
interface GenerateGroundedDocumentContext {
  store: ReturnType<typeof useSpaceStore>;
  sources: readonly GroundedDocumentSource[];
  draft: GroundedDocumentDraft;
}

export function generateGroundedDocument({ store, sources, draft }: GenerateGroundedDocumentContext) {
  return startDerivationAction({
    store,
    sources,
    actionId: "draft-document",
    parameters: { title: draft.title.trim(), purpose: draft.purpose.trim() },
    target: draft.target,
    title: i18n.global.t("spaces.creation.document.task"),
  });
}
