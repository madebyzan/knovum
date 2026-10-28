import { routingCapabilities } from "../../shared/contracts/generated/contract-values";
import type { RoutingPurpose } from "../../shared/contracts/generated/RoutingPurpose";
import type { TaskTarget } from "../../shared/contracts/generated/TaskTarget";
import { createAppError } from "../errors/app-error";
import { useAiSettingsStore } from "./ai-settings-store";
import type { AiModelSettings } from "./schema/model-settings";
import type { ResolvedAiModelTarget, ResolvedAiTaskTarget } from "./schema/models";

const DEFAULT_TARGET_FIELD_BY_PURPOSE: Record<RoutingPurpose, keyof AiModelSettings["defaults"]> = {
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

/** The routing default that assigns an executor to `purpose`; Settings names its route by it. */
export function routingDefaultsKey(purpose: RoutingPurpose): keyof AiModelSettings["defaults"] {
  return DEFAULT_TARGET_FIELD_BY_PURPOSE[purpose];
}

export function defaultExecutionTarget(
  settings: AiModelSettings,
  purpose: RoutingPurpose,
): TaskTarget | null {
  const target = settings.defaults[routingDefaultsKey(purpose)] ?? settings.defaults.defaultTarget;
  return target ? { ...target } : null;
}

const AI_TARGET_UNCONFIGURED = "ai_target_unconfigured";

function resolveModelTarget(
  settings: AiModelSettings,
  purpose: RoutingPurpose,
  target: Extract<TaskTarget, { type: "model" }>,
): ResolvedAiModelTarget | undefined {
  const connection = settings.connections.find(({ id }) => id === target.connectionId);
  const model = connection?.models.find(({ id }) => id === target.modelId);
  if (!connection || !model?.capabilities.includes(routingCapabilities[purpose])) return undefined;
  return { type: "model", connection, model };
}

/**
 * The executor `purpose` runs on: `target` when given, otherwise the routed default. Undefined when
 * that executor is missing, cannot serve `purpose` or is a desktop Agent that is not enabled.
 */
export function resolveExecutionTarget(
  settings: AiModelSettings,
  purpose: RoutingPurpose,
  target?: TaskTarget,
): ResolvedAiTaskTarget | undefined {
  const selected = target ?? defaultExecutionTarget(settings, purpose);
  if (!selected) return undefined;
  if (selected.type === "model") return resolveModelTarget(settings, purpose, selected);
  return settings.desktopAgents.some(({ kind, enabled }) => kind === selected.runtimeKind && enabled)
    ? selected
    : undefined;
}

export async function loadAiExecutionTarget(
  purpose: RoutingPurpose,
  target?: TaskTarget,
): Promise<ResolvedAiTaskTarget> {
  const store = useAiSettingsStore();
  await store.ensureLoaded();
  if (store.loadError) throw store.loadError;
  const resolved = resolveExecutionTarget(store.settings, purpose, target);
  if (!resolved)
    throw createAppError(AI_TARGET_UNCONFIGURED, `No usable AI executor is configured for ${purpose}`);
  return resolved;
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
