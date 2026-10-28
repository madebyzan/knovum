import { queryClient } from "../query/query-client";
import { localModelsQueryOptions } from "../runtime/use-local-models-query";
import {
  aiSourcesOf,
  canStartWith,
  desktopAgentPresence,
  resolveAiReadiness,
  type AiReadinessSources,
  type AiRequirement,
} from "./ai-readiness";
import { announceAiReadiness } from "./ai-readiness-prompt";
import { useAiSettingsStore } from "./ai-settings-store";
import { agentDescriptorsQueryOptions, agentInstallationsQueryOptions } from "./use-execution-catalog-query";

/**
 * Decides, at the moment a feature starts, whether it may run, and tells the person why not or what
 * running costs first. Reads current state rather than a cached projection, because a download, an
 * installation or a Settings change may have happened since the control was drawn.
 */
export async function admitAiRequirements(requirements: readonly AiRequirement[]): Promise<boolean> {
  const reads = aiSourcesOf(requirements, undefined);
  const [settings, models] = await Promise.all([
    reads.has("settings") ? loadSettings() : unread<AiReadinessSources["settings"]>(),
    reads.has("models") ? loadModels() : unread<AiReadinessSources["models"]>(),
  ]);
  // Which executor a route resolves to is only known once the settings are.
  const agents = aiSourcesOf(requirements, settings.value).has("agents")
    ? await loadAgents()
    : await unread<AiReadinessSources["agents"]>();
  const readiness = resolveAiReadiness(requirements, {
    settings: settings.value,
    settingsFailed: settings.failed,
    models: models.value,
    modelsFailed: models.failed,
    agents: agents.value,
    agentsFailed: agents.failed,
  });
  announceAiReadiness(readiness);
  return canStartWith(readiness);
}

interface SourceRead<T> {
  value: T | undefined;
  failed: boolean;
}

async function loadSettings(): Promise<SourceRead<AiReadinessSources["settings"]>> {
  const store = useAiSettingsStore();
  await store.ensureLoaded();
  return store.loadError ? { value: undefined, failed: true } : { value: store.settings, failed: false };
}

function loadModels(): Promise<SourceRead<AiReadinessSources["models"]>> {
  return fresh(() => queryClient.fetchQuery({ ...localModelsQueryOptions, staleTime: 0, retry: false }));
}

function loadAgents(): Promise<SourceRead<AiReadinessSources["agents"]>> {
  return fresh(async () => {
    const [descriptors, installations] = await Promise.all([
      queryClient.fetchQuery(agentDescriptorsQueryOptions),
      queryClient.fetchQuery({ ...agentInstallationsQueryOptions, staleTime: 0, retry: false }),
    ]);
    return desktopAgentPresence(descriptors, installations);
  });
}

/** Admission answers a click, so a source that cannot be read is reported at once, not retried. */
async function fresh<T>(read: () => Promise<T>): Promise<SourceRead<T>> {
  try {
    return { value: await read(), failed: false };
  } catch {
    return { value: undefined, failed: true };
  }
}

function unread<T>(): Promise<SourceRead<T>> {
  return Promise.resolve({ value: undefined, failed: false });
}
