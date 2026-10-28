import { admitAiRequirements } from "../../../../foundation/ai/ai-readiness-admission";
import {
  isResourcePreparationCancelled,
  startResourcePreparation,
} from "../../../../foundation/runtime/resource-preparation";

/**
 * Prepares the embedding model knowledge search and Related Content read. The person chose to turn
 * either on, so the download starts here and is followed in Tasks. Resolves false when nothing was
 * prepared because the person cancelled the download or the model cannot run on this computer.
 */
export async function prepareTextEmbedding(title: string): Promise<boolean> {
  if (!(await admitAiRequirements([{ kind: "localCapability", capability: "embedding" }]))) return false;
  try {
    await startResourcePreparation({ kind: "textEmbedding" }, title).completion;
    return true;
  } catch (error) {
    if (isResourcePreparationCancelled(error)) return false;
    throw error;
  }
}
