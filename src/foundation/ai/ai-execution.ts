import { agentTargetSchema, type AgentDescriptor, type AgentInstallation } from "./agent-models";
import type { AiModelTarget, AiRoutingPurpose, ExecutionTarget, ResolvedAiModelTarget } from "./ai-models";
import { useAiSettingsStore } from "./ai-settings-store";
import { routingPolicy } from "./execution-routing";
import type { AiModelSettings } from "./integration-settings";

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
  executorType: "model" | "agent";
  capabilitySource: "model-native" | "agent-workflow";
  availability: "ready" | "unverified" | "unavailable";
  disabled: boolean;
  disabledReason?: string | undefined;
  statusDetail?: string | undefined;
}

export type ResolvedAiExecutionTarget =
  | ResolvedAiModelTarget
  | { type: "agent"; agent: AiModelSettings["desktopAgents"][number] };

function copyExecutionTarget(target: ExecutionTarget): ExecutionTarget {
  return target.type === "agent"
    ? { type: "agent", kind: target.kind }
    : {
        type: "model",
        connectionId: target.connectionId,
        modelId: target.modelId,
      };
}

export function defaultExecutionTarget(
  settings: AiModelSettings,
  purpose: AiRoutingPurpose,
): ExecutionTarget | null {
  const target = settings.defaults[defaultTargetKey[purpose]];
  return target ? copyExecutionTarget(target) : null;
}

function resolveModelTarget(
  settings: AiModelSettings,
  purpose: AiRoutingPurpose,
  target: AiModelTarget,
): ResolvedAiModelTarget {
  const connection = settings.connections.find(({ id }) => id === target.connectionId);
  if (!connection) throw new Error(`AI connection is unavailable: ${target.connectionId}`);
  const model = connection.models.find(({ id }) => id === target.modelId);
  if (!model) throw new Error(`AI model is unavailable: ${target.modelId}`);
  const capability = routingPolicy(purpose).modelCapability;
  if (!model.capabilities.includes(capability)) {
    throw new Error(`AI model ${model.name} does not support ${capability}`);
  }
  return { type: "model", connection, model };
}

function resolveExecutionTarget(
  settings: AiModelSettings,
  purpose: AiRoutingPurpose,
  selected: ExecutionTarget,
): ResolvedAiExecutionTarget {
  if (selected.type === "model") return resolveModelTarget(settings, purpose, selected);
  const agent = settings.desktopAgents.find(({ kind }) => kind === selected.kind);
  if (!agent) throw new Error(`Desktop agent is not enabled: ${selected.kind}`);
  return { type: "agent", agent };
}

export async function loadAiExecutionTarget(
  purpose: AiRoutingPurpose,
  target?: ExecutionTarget,
): Promise<ResolvedAiExecutionTarget> {
  const store = useAiSettingsStore();
  await store.ensureLoaded();
  const settings = store.settings;
  const selected = target ?? defaultExecutionTarget(settings, purpose);
  if (!selected) throw new Error(`AI execution target is not configured for ${purpose}`);
  return resolveExecutionTarget(settings, purpose, selected);
}

export function aiExecutionTargetPayload(target: ResolvedAiExecutionTarget, temperature = 0.2) {
  if (target.type === "agent") return { type: "agent" as const, kind: target.agent.kind };
  return {
    type: "model" as const,
    connectionId: target.connection.id,
    modelId: target.model.id,
    temperature,
  };
}

function modelOptions(settings: AiModelSettings, purpose: AiRoutingPurpose): ExecutionTargetOption[] {
  const capability = routingPolicy(purpose).modelCapability;
  return settings.connections.flatMap((connection) =>
    connection.models
      .filter((model) => model.capabilities.includes(capability))
      .map((model) => ({
        value: `model:${connection.id}:${model.id}`,
        label: `${model.name} · ${connection.name}`,
        target: { type: "model" as const, connectionId: connection.id, modelId: model.id },
        executorType: "model" as const,
        capabilitySource: "model-native" as const,
        availability: "ready" as const,
        disabled: false,
      })),
  );
}

function agentAvailability(readiness: AgentInstallation | undefined) {
  if (!readiness) return "unverified" as const;
  if (!readiness.detected) return "unavailable" as const;
  return "unverified" as const;
}

function agentOptions(
  settings: AiModelSettings,
  descriptors: readonly AgentDescriptor[],
  installations: readonly AgentInstallation[],
): ExecutionTargetOption[] {
  return settings.desktopAgents.flatMap((agent) => {
    const descriptor = descriptors.find(({ kind }) => kind === agent.kind);
    if (!descriptor) return [];
    const readiness = installations.find(({ kind }) => kind === agent.kind);
    return [
      {
        value: `agent:${agent.kind}`,
        label: descriptor.displayName,
        target: { type: "agent" as const, kind: agent.kind },
        executorType: "agent" as const,
        capabilitySource: "agent-workflow" as const,
        availability: agentAvailability(readiness),
        disabled: false,
      },
    ];
  });
}

export function availableExecutionTargets(
  settings: AiModelSettings,
  purpose: AiRoutingPurpose,
  descriptors: readonly AgentDescriptor[],
  installations: readonly AgentInstallation[] = [],
): ExecutionTargetOption[] {
  return [...modelOptions(settings, purpose), ...agentOptions(settings, descriptors, installations)];
}

export const encodeExecutionTarget = (target: ExecutionTarget) =>
  target.type === "agent" ? `agent:${target.kind}` : `model:${target.connectionId}:${target.modelId}`;

export function decodeExecutionTarget(value: string): ExecutionTarget {
  if (value.startsWith("agent:")) {
    return agentTargetSchema.parse({ type: "agent", kind: value.slice("agent:".length) });
  }
  const [type, connectionId, modelId, ...rest] = value.split(":");
  if (type !== "model" || !connectionId || !modelId || rest.length) {
    throw new Error("Invalid AI execution target");
  }
  return { type: "model", connectionId, modelId };
}
