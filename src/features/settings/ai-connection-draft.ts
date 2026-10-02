import { cloneDeep } from "es-toolkit";

import type { AiConnectionSettings } from "../../shared/contracts/generated/AiConnectionSettings";

/** undefined keeps the stored key, null removes it, and a string replaces it. */
export type AiConnectionSecretChange = string | null | undefined;

export interface AiConnectionSubmission {
  connection: AiConnectionSettings;
  secret: AiConnectionSecretChange;
}

export function createAiConnectionDraft(connection: AiConnectionSettings): AiConnectionSettings {
  return cloneDeep(connection);
}

export function finalizeAiConnectionDraft(draft: AiConnectionSettings): AiConnectionSettings {
  return {
    id: draft.id,
    name: draft.name.trim(),
    provider: draft.provider,
    baseUrl: draft.baseUrl.trim(),
    models: draft.models.map((model) => ({
      id: model.id,
      name: model.name.trim(),
      model: model.model.trim(),
      capabilities: [...model.capabilities],
    })),
  };
}
