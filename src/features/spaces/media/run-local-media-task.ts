import type { Ref } from "vue";
import { toast } from "vue-sonner";

import { admitAiRequirements } from "../../../foundation/ai/ai-readiness-admission";
import { preferredLocalModel } from "../../../foundation/ai/local-model-choice";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { LocalModelId } from "../../../shared/contracts/generated/LocalModelId";
import { refreshEnrichmentJobs } from "../enrichment/enrichment-jobs-query";

export async function runLocalMediaTask(
  capability: "transcription" | "speechAlignment",
  busy: Ref<boolean>,
  selectedModel: LocalModelId | undefined,
  task: (model: LocalModelId) => Promise<unknown>,
) {
  busy.value = true;
  try {
    if (!(await admitAiRequirements([{ kind: "localCapability", capability }]))) return;
    const model = selectedModel ?? (await preferredLocalModel(capability));
    if (!model) return;
    await task(model);
    await refreshEnrichmentJobs();
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    busy.value = false;
  }
}
