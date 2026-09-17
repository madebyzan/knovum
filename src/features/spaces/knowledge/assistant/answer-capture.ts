import { toast } from "vue-sonner";

import { i18n } from "../../../../foundation/i18n/i18n";
import { commitGroundedAnswerKnowledge } from "../../editor/knowledge-capture";
import type { GroundedMessage } from "../model/grounded-message";
import type { KnowledgeAssistantActionContext } from "./action-context";

const t = i18n.global.t;

export async function saveAsKnowledge(context: KnowledgeAssistantActionContext, message: GroundedMessage) {
  try {
    await commitGroundedAnswerKnowledge(
      context.store,
      message,
      t("spaces.assistant.knowledgeClaimTitle"),
      t("spaces.assistant.groundedSourceRationale"),
    );
    toast.success(t("spaces.assistant.knowledgeSaved"));
  } catch (error) {
    toast.error(String(error));
  }
}
