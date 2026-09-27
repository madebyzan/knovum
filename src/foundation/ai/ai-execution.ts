import { routingCapabilities } from "../../shared/contracts/generated/contract-values";
import { useAiSettingsStore } from "./ai-settings-store";
import { encodeExecutionTarget } from "./execution-target-codec";
import type { AiModelSettings } from "./integration-settings";
import type { AiRoutingPurpose, ExecutionTarget, ResolvedAiModelTarget } from "./schema/models";

const defaultTargetKey: Record<AiRoutingPurpose, keyof AiModelSettings["defaults"]> = {
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
  target: ExecutionTarget;
}

export function defaultExecutionTarget(
  settings: AiModelSettings,
  purpose: AiRoutingPurpose,
): ExecutionTarget | null {
  const target = settings.defaults[defaultTargetKey[purpose]];
  return target ? { ...target } : null;
}

function resolveModelTarget(
  settings: AiModelSettings,
  purpose: AiRoutingPurpose,
  target: ExecutionTarget,
): ResolvedAiModelTarget {
  const connection = settings.connections.find(({ id }) => id === target.connectionId);
  if (!connection) throw new Error(`AI connection is unavailable: ${target.connectionId}`);
  const model = connection.models.find(({ id }) => id === target.modelId);
  if (!model) throw new Error(`AI model is unavailable: ${target.modelId}`);
  const capability = routingCapabilities[purpose];
  if (!model.capabilities.includes(capability)) {
    throw new Error(`AI model ${model.name} does not support ${capability}`);
  }
  return { type: "model", connection, model };
}

export async function loadAiExecutionTarget(
  purpose: AiRoutingPurpose,
  target?: ExecutionTarget,
): Promise<ResolvedAiModelTarget> {
  const store = useAiSettingsStore();
  await store.ensureLoaded();
  const settings = store.settings;
  const selected = target ?? defaultExecutionTarget(settings, purpose);
  if (!selected) throw new Error(`AI execution target is not configured for ${purpose}`);
  return resolveModelTarget(settings, purpose, selected);
}

export function aiExecutionTargetPayload(target: ResolvedAiModelTarget, temperature = 0.2) {
  return {
    type: "model" as const,
    connectionId: target.connection.id,
    modelId: target.model.id,
    temperature,
  };
}

export function availableExecutionTargets(
  settings: AiModelSettings,
  purpose: AiRoutingPurpose,
): ExecutionTargetOption[] {
  const capability = routingCapabilities[purpose];
  return settings.connections.flatMap((connection) =>
    connection.models
      .filter((model) => model.capabilities.includes(capability))
      .map((model) => {
        const target: ExecutionTarget = { type: "model", connectionId: connection.id, modelId: model.id };
        return { value: encodeExecutionTarget(target), label: `${model.name} · ${connection.name}`, target };
      }),
  );
}
