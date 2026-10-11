import { i18n } from "../../../foundation/i18n/instance";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { GroundedDocumentSource } from "./catalog/grounded-document";
import { startDerivationAction } from "./start-action";

export interface GroundedDocumentDraft {
  title: string;
  purpose: string;
  target?: TaskTargetRef | undefined;
}
interface GenerateGroundedDocumentContext {
  sources: readonly GroundedDocumentSource[];
  draft: GroundedDocumentDraft;
}

export function generateGroundedDocument({ sources, draft }: GenerateGroundedDocumentContext) {
  return startDerivationAction({
    sources,
    actionId: "draft-document",
    parameters: { title: draft.title.trim(), purpose: draft.purpose.trim() },
    target: draft.target,
    title: i18n.global.t("spaces.creation.document.task"),
  });
}
