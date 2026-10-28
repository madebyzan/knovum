import type { ExternalAgentDescriptor } from "../../shared/contracts/generated/ExternalAgentDescriptor";
import type { ExternalAgentInstallation } from "../../shared/contracts/generated/ExternalAgentInstallation";
import type { ExternalAgentKind } from "../../shared/contracts/generated/ExternalAgentKind";
import type { LocalModelAvailability } from "../../shared/contracts/generated/LocalModelAvailability";
import type { LocalModelCapability } from "../../shared/contracts/generated/LocalModelCapability";
import type { LocalModelId } from "../../shared/contracts/generated/LocalModelId";
import type { RoutingPurpose } from "../../shared/contracts/generated/RoutingPurpose";
import type { TaskTarget } from "../../shared/contracts/generated/TaskTarget";
import { availableExecutionTargets, availableModelTargets, resolveExecutionTarget } from "./ai-execution";
import type { AiSettingsPane } from "./ai-settings-navigation";
import type { AiModelSettings } from "./schema/model-settings";

/**
 * What a feature needs from AI before it can run. A routed requirement executes on the provider or
 * desktop Agent assigned to its purpose, or on the executor the feature itself chose for it; a model
 * choice lets the person pick among the configured models that can serve it. A local requirement
 * executes on a model this computer downloads: the one Settings selected for a capability, or the
 * one the feature itself chose.
 */
export type AiRequirement =
  | { readonly kind: "routed"; readonly purpose: RoutingPurpose; readonly target?: TaskTarget | undefined }
  | { readonly kind: "modelChoice"; readonly purpose: RoutingPurpose }
  | { readonly kind: "localCapability"; readonly capability: LocalModelCapability }
  | { readonly kind: "localModel"; readonly model: LocalModelId };

/**
 * Whether a feature's requirements are met. A missing routed executor blocks the feature until
 * Settings assigns one, and a desktop Agent whose command-line entry this computer lacks blocks it
 * until that is installed. A local model that is not downloaded yet does not block it: the feature
 * prepares the model when it starts, so the person only needs to know what that costs and where to
 * follow it. A source that cannot be read leaves the state unknown, which blocks too.
 */
export type AiReadiness =
  | { readonly state: "checking" }
  | { readonly state: "ready" }
  | {
      readonly state: "download";
      readonly models: readonly LocalModelAvailability[];
      readonly bytes: number;
    }
  | {
      readonly state: "configure";
      readonly purpose: RoutingPurpose;
      readonly pane: Extract<AiSettingsPane, "aiProviders" | "aiRouting">;
    }
  | { readonly state: "install"; readonly agent: DesktopAgentPresence }
  | { readonly state: "unavailable"; readonly source: AiReadinessSource | "platform" };

/** A desktop Agent as this computer has it: launchable only through its command-line entry. */
export interface DesktopAgentPresence {
  readonly kind: ExternalAgentKind;
  readonly name: string;
  readonly installed: boolean;
}

/** A desktop Agent launches through the command-line entry its detection found, never its app. */
export function desktopAgentPresence(
  descriptors: readonly ExternalAgentDescriptor[],
  installations: readonly ExternalAgentInstallation[],
): DesktopAgentPresence[] {
  return descriptors.map(({ kind, displayName }) => ({
    kind,
    name: displayName,
    installed: Boolean(installations.find((installation) => installation.kind === kind)?.executablePath),
  }));
}

/** What a requirement is decided against. */
export type AiReadinessSource = "settings" | "models" | "agents";

export interface AiReadinessSources {
  /** Absent until the AI settings have loaded. */
  readonly settings: AiModelSettings | undefined;
  readonly settingsFailed: boolean;
  /** Absent until the local model catalog has loaded. */
  readonly models: readonly LocalModelAvailability[] | undefined;
  readonly modelsFailed: boolean;
  /** Absent until the desktop Agents of this computer have been detected. */
  readonly agents: readonly DesktopAgentPresence[] | undefined;
  readonly agentsFailed: boolean;
}

/** The state that decides first when a feature has several requirements: whatever stops it. */
const PRECEDENCE: readonly AiReadiness["state"][] = [
  "unavailable",
  "configure",
  "install",
  "checking",
  "download",
  "ready",
];

export function resolveAiReadiness(
  requirements: readonly AiRequirement[],
  sources: AiReadinessSources,
): AiReadiness {
  const states = requirements.map((requirement) => requirementReadiness(requirement, sources));
  const decisive = PRECEDENCE.find((state) => states.some((candidate) => candidate.state === state));
  if (decisive !== "download") return states.find(({ state }) => state === decisive) ?? { state: "ready" };
  const models = [
    ...new Map(
      states
        .flatMap((state) => (state.state === "download" ? state.models : []))
        .map((model) => [model.id, model]),
    ).values(),
  ];
  return {
    state: "download",
    models,
    bytes: models.reduce((total, model) => total + remainingBytes(model), 0),
  };
}

/** A feature may start while its local models are still to be downloaded; it prepares them first. */
export function canStartWith(readiness: AiReadiness): boolean {
  return readiness.state === "ready" || readiness.state === "download";
}

/**
 * The sources the requirements are decided against. Agent detection is read only for a routed
 * requirement that `settings` resolve to a desktop Agent, so features that run on a provider never
 * pay for it; until the settings are known, that cannot be told yet.
 */
export function aiSourcesOf(
  requirements: readonly AiRequirement[],
  settings: AiModelSettings | undefined,
): ReadonlySet<AiReadinessSource> {
  return new Set(
    requirements.flatMap((requirement): AiReadinessSource[] => {
      switch (requirement.kind) {
        case "routed":
          return settings &&
            resolveExecutionTarget(settings, requirement.purpose, requirement.target)?.type === "agent"
            ? ["settings", "agents"]
            : ["settings"];
        case "modelChoice":
          return ["settings"];
        case "localCapability":
        case "localModel":
          return ["models"];
      }
    }),
  );
}

function requirementReadiness(requirement: AiRequirement, sources: AiReadinessSources): AiReadiness {
  switch (requirement.kind) {
    case "routed":
      return routedReadiness(requirement, sources);
    case "modelChoice":
      return modelChoiceReadiness(requirement.purpose, sources);
    case "localCapability":
    case "localModel":
      return localReadiness(requirement, sources);
  }
}

function routedReadiness(
  { purpose, target }: Extract<AiRequirement, { kind: "routed" }>,
  sources: AiReadinessSources,
): AiReadiness {
  if (sources.settingsFailed) return { state: "unavailable", source: "settings" };
  const { settings } = sources;
  if (!settings) return { state: "checking" };
  const executor = resolveExecutionTarget(settings, purpose, target);
  if (!executor) {
    // A compatible executor that exists only needs assigning; otherwise one has to be added first.
    const pane = availableExecutionTargets(settings, purpose).length > 0 ? "aiRouting" : "aiProviders";
    return { state: "configure", purpose, pane };
  }
  if (executor.type === "model") return { state: "ready" };
  if (sources.agentsFailed) return { state: "unavailable", source: "agents" };
  if (!sources.agents) return { state: "checking" };
  const agent = sources.agents.find(({ kind }) => kind === executor.runtimeKind);
  if (!agent) return { state: "unavailable", source: "platform" };
  return agent.installed ? { state: "ready" } : { state: "install", agent };
}

function modelChoiceReadiness(purpose: RoutingPurpose, sources: AiReadinessSources): AiReadiness {
  if (sources.settingsFailed) return { state: "unavailable", source: "settings" };
  if (!sources.settings) return { state: "checking" };
  return availableModelTargets(sources.settings, purpose).length > 0
    ? { state: "ready" }
    : { state: "configure", purpose, pane: "aiProviders" };
}

function localReadiness(
  requirement: Extract<AiRequirement, { kind: "localCapability" | "localModel" }>,
  sources: AiReadinessSources,
): AiReadiness {
  if (sources.modelsFailed) return { state: "unavailable", source: "models" };
  if (!sources.models) return { state: "checking" };
  const model = sources.models.find((candidate) =>
    requirement.kind === "localModel"
      ? candidate.id === requirement.model
      : candidate.capability === requirement.capability && (candidate.selected || candidate.fixed),
  );
  if (!model) return { state: "unavailable", source: "platform" };
  return model.verified
    ? { state: "ready" }
    : { state: "download", models: [model], bytes: remainingBytes(model) };
}

/** A download resumes from what is already on disk. */
function remainingBytes(model: LocalModelAvailability) {
  return Math.max(0, model.totalBytes - model.installedBytes);
}
