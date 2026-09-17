import type { AiConnectionSettings, AiModelDefinition } from "../../foundation/ai/ai-models";

function copyModel(model: AiModelDefinition): AiModelDefinition {
  return {
    id: model.id,
    name: model.name,
    model: model.model,
    capabilities: [...model.capabilities],
  };
}

export function createAiConnectionDraft(connection: AiConnectionSettings): AiConnectionSettings {
  return {
    id: connection.id,
    name: connection.name,
    provider: connection.provider,
    baseUrl: connection.baseUrl,
    models: connection.models.map(copyModel),
  };
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
