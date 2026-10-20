import { routingCapabilities } from "../../shared/contracts/generated/contract-values";
import type { RoutingPurpose } from "../../shared/contracts/generated/RoutingPurpose";
import type { TaskTarget } from "../../shared/contracts/generated/TaskTarget";
import { useAiSettingsStore } from "./ai-settings-store";
import type { AiModelSettings } from "./schema/model-settings";
import type { ResolvedAiModelTarget, ResolvedAiTaskTarget } from "./schema/models";

const DEFAULT_TARGET_KEY: Record<RoutingPurpose, keyof AiModelSettings["defaults"]> = {
  writing: "writing",
  knowledge: "knowledge",
  "visual-understanding": "visualUnderstanding",
  translation: "translation",
  "image-generation": "imageGeneration",
  "video-generation": "videoGeneration",
};

export interface ExecutionTargetOption {
  value: string;
  label: string;
  target: TaskTarget;
}

export function defaultExecutionTarget(
  settings: AiModelSettings,
  purpose: RoutingPurpose,
): TaskTarget | null {
  const target = settings.defaults[DEFAULT_TARGET_KEY[purpose]] ?? settings.defaults.defaultTarget;
  return target ? { ...target } : null;
}

function resolveModelTarget(
  settings: AiModelSettings,
  purpose: RoutingPurpose,
  target: Extract<TaskTarget, { type: "model" }>,
): ResolvedAiModelTarget {
  const connection = settings.connections.find(({ id }) => id === target.connectionId);
  if (!connection) throw new Error(`AI connection is unavailable: ${target.connectionId}`);
  const model = connection.models.find(({ id }) => id === target.modelId);
  if (!model) throw new Error(`AI model is unavailable: ${target.modelId}`);
  const capability = routingCapabilities[purpose];
  if (!model.capabilities.includes(capability))
    throw new Error(`AI model ${model.name} does not support ${capability}`);
  return { type: "model", connection, model };
}

export async function loadAiExecutionTarget(
  purpose: RoutingPurpose,
  target?: TaskTarget,
): Promise<ResolvedAiTaskTarget> {
  const store = useAiSettingsStore();
  await store.ensureLoaded();
  if (store.loadError) throw store.loadError;
  const settings = store.settings;
  const selected = target ?? defaultExecutionTarget(settings, purpose);
  if (!selected) throw new Error(`AI execution method is not selected for ${purpose}`);
  if (selected.type === "model") return resolveModelTarget(settings, purpose, selected);
  if (!settings.desktopAgents.some(({ kind, enabled }) => kind === selected.runtimeKind && enabled))
    throw new Error("Desktop Agent is not enabled");
  return selected;
}

export function aiExecutionTargetPayload(target: ResolvedAiTaskTarget, temperature = 0.2) {
  return target.type === "agent"
    ? { ...target }
    : {
        type: "model" as const,
        connectionId: target.connection.id,
        modelId: target.model.id,
        temperature,
      };
}

export function availableModelTargets(
  settings: Pick<AiModelSettings, "connections">,
  purpose?: RoutingPurpose,
): (Omit<ExecutionTargetOption, "target"> & { target: Extract<TaskTarget, { type: "model" }> })[] {
  return settings.connections.flatMap((connection) =>
    connection.models
      .filter((model) => !purpose || model.capabilities.includes(routingCapabilities[purpose]))
      .map((model) => {
        const target: TaskTarget = { type: "model", connectionId: connection.id, modelId: model.id };
        return { value: JSON.stringify(target), label: `${model.name} · ${connection.name}`, target };
      }),
  );
}

export function availableExecutionTargets(
  settings: Pick<AiModelSettings, "connections" | "desktopAgents">,
  purpose?: RoutingPurpose,
): ExecutionTargetOption[] {
  const agents = settings.desktopAgents
    .filter(({ enabled }) => enabled)
    .map(({ kind }) => {
      const target: TaskTarget = { type: "agent", runtimeKind: kind };
      return { value: JSON.stringify(target), label: kind, target };
    });
  return [...availableModelTargets(settings, purpose), ...agents];
}
