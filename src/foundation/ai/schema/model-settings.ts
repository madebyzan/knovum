import { z } from "zod";

import type { AiSettings } from "../../../shared/contracts/generated/AiSettings";
import { routingCapabilities } from "../../../shared/contracts/generated/contract-values";
import type { ExecutionCapability } from "../../../shared/contracts/generated/ExecutionCapability";
import type { RoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import { desktopAgentSettingsSchema } from "./agents";
import { aiConnectionSettingsSchema, aiDefaultsSchema } from "./models";

type AiModelSettingsShape = AiSettings;
type SettingsRefinement = z.core.$RefinementCtx<AiModelSettingsShape>;

export const aiModelSettingsSchema = z
  .object({
    connections: z.array(aiConnectionSettingsSchema),
    desktopAgents: z.array(desktopAgentSettingsSchema),
    defaults: aiDefaultsSchema,
  })
  .strict()
  .superRefine((settings, context) => {
    rejectDuplicateConnections(settings, context);
    rejectDuplicateDesktopAgents(settings, context);
    rejectInvalidDefaultTargets(settings, context);
  });

export type AiModelSettings = z.infer<typeof aiModelSettingsSchema>;

export const defaultAiModelSettings: AiModelSettings = {
  connections: [],
  desktopAgents: [],
  defaults: aiDefaultsSchema.parse({}),
};

function rejectDuplicateConnections(settings: AiModelSettingsShape, context: SettingsRefinement) {
  if (new Set(settings.connections.map(({ id }) => id)).size !== settings.connections.length) {
    context.addIssue({ code: "custom", path: ["connections"], message: "Duplicate AI connection id" });
  }
}

function rejectDuplicateDesktopAgents(settings: AiModelSettingsShape, context: SettingsRefinement) {
  if (new Set(settings.desktopAgents.map(({ kind }) => kind)).size !== settings.desktopAgents.length) {
    context.addIssue({ code: "custom", path: ["desktopAgents"], message: "Duplicate desktop agent" });
  }
}

function rejectInvalidDefaultTargets(settings: AiModelSettingsShape, context: SettingsRefinement) {
  const targets: ReadonlyArray<readonly [TaskTargetRef | null, RoutingPurpose | undefined]> = [
    [settings.defaults.defaultTarget, undefined],
    [settings.defaults.writing, "writing"],
    [settings.defaults.knowledge, "knowledge"],
    [settings.defaults.visualUnderstanding, "visual-understanding"],
    [settings.defaults.translation, "translation"],
    [settings.defaults.imageGeneration, "image-generation"],
    [settings.defaults.videoGeneration, "video-generation"],
  ];

  for (const [target, purpose] of targets) {
    if (!target) continue;
    if (target.type === "agent") {
      if (!settings.desktopAgents.some(({ kind, enabled }) => kind === target.runtimeKind && enabled))
        context.addIssue({ code: "custom", message: "Default desktop Agent is not enabled" });
      continue;
    }
    const capability = purpose ? routingCapabilities[purpose] : undefined;
    if (!resolvesToCapability(settings.connections, target, capability)) {
      context.addIssue({ code: "custom", message: `Invalid ${capability} model target` });
    }
  }
}

function resolvesToCapability(
  connections: AiModelSettingsShape["connections"],
  target: Extract<TaskTargetRef, { type: "model" }>,
  capability: ExecutionCapability | undefined,
) {
  const connection = connections.find(({ id }) => id === target.connectionId);
  const model = connection?.models.find(({ id }) => id === target.modelId);
  return Boolean(model && (!capability || model.capabilities.includes(capability)));
}
