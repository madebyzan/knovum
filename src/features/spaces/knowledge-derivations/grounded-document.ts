import { i18n } from "../../../foundation/i18n/i18n";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { GroundedDocumentSource } from "./catalog/grounded-document";
import { startDerivationAction, type DerivationActionStore } from "./start-action";

export interface GroundedDocumentDraft {
  title: string;
  purpose: string;
  target?: TaskTargetRef | undefined;
}
interface GenerateGroundedDocumentContext {
  store: DerivationActionStore;
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
