import type { LocalModelAvailability } from "../../../../shared/contracts/generated/LocalModelAvailability";
import type { AiReadinessSources } from "../../ai-readiness";
import { DEFAULT_AI_MODEL_SETTINGS, type AiModelSettings } from "../../schema/model-settings";

export function model(overrides: Partial<LocalModelAvailability>): LocalModelAvailability {
  return {
    id: "whisperSmall",
    name: "Whisper Small",
    capability: "transcription",
    operations: [],
    referenceTranscript: "unused",
    group: "audio",
    essential: true,
    selected: true,
    selectable: true,
    fixed: false,
    totalBytes: 500,
    installedBytes: 0,
    installed: false,
    verified: false,
    memoryAdvisoryBytes: null,
    ...overrides,
  };
}

export function settingsWith(update: (settings: AiModelSettings) => void): AiModelSettings {
  const settings = structuredClone(DEFAULT_AI_MODEL_SETTINGS);
  update(settings);
  return settings;
}

export const writer = (settings: AiModelSettings) =>
  settings.connections.push({
    id: "provider",
    name: "Provider",
    provider: "openai",
    baseUrl: "https://api.openai.com/v1/",
    models: [
      {
        id: "writer",
        name: "Writer",
        model: "writer",
        capabilities: ["text-generation"],
        builtinAgent: false,
      },
    ],
  });

export function sources(overrides: Partial<AiReadinessSources>): AiReadinessSources {
  return {
    settings: undefined,
    settingsFailed: false,
    models: undefined,
    modelsFailed: false,
    agents: undefined,
    agentsFailed: false,
    ...overrides,
  };
}
