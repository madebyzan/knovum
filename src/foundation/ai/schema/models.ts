import { z } from "zod";

import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import type { AiModelDefinition } from "../../../shared/contracts/generated/AiModelDefinition";
import {
  providerKinds,
  executionCapabilities,
  routingPurposes,
} from "../../../shared/contracts/generated/contract-values";

export const aiProviderSchema = z.enum(providerKinds);
export const aiCapabilitySchema = z.enum(executionCapabilities);
export const aiRoutingPurposeSchema = z.enum(routingPurposes);

export const aiModelDefinitionSchema = z
  .object({
    id: z.string().trim().min(1),
    name: z.string().trim().min(1),
    model: z.string().trim().min(1),
    capabilities: z.array(aiCapabilitySchema).min(1),
  })
  .strict();

export const aiConnectionSettingsSchema = z
  .object({
    id: z.string().trim().min(1),
    name: z.string().trim().min(1),
    provider: aiProviderSchema,
    baseUrl: z.string().trim().min(1),
    models: z.array(aiModelDefinitionSchema),
  })
  .strict()
  .superRefine((connection, context) => {
    if (URL.canParse(connection.baseUrl) === false) {
      context.addIssue({ code: "custom", path: ["baseUrl"], message: "Invalid AI provider URL" });
    }
    if (new Set(connection.models.map(({ id }) => id)).size !== connection.models.length) {
      context.addIssue({ code: "custom", path: ["models"], message: "Duplicate AI model id" });
    }
    if (new Set(connection.models.map(({ model }) => model)).size !== connection.models.length) {
      context.addIssue({
        code: "custom",
        path: ["models"],
        message: "Duplicate AI provider model identifier",
      });
    }
  });

export const executionTargetSchema = z
  .object({ type: z.literal("model"), connectionId: z.string().min(1), modelId: z.string().min(1) })
  .strict();

export const aiDefaultsSchema = z
  .object({
    writing: executionTargetSchema.nullable(),
    knowledge: executionTargetSchema.nullable(),
    visualUnderstanding: executionTargetSchema.nullable(),
    translation: executionTargetSchema.nullable(),
    imageGeneration: executionTargetSchema.nullable(),
    videoGeneration: executionTargetSchema.nullable(),
  })
  .strict();

export const generationPresetSchema = z
  .object({
    id: z.string().min(1),
    name: z.string().min(1),
    actionId: z.string().min(1),
    target: executionTargetSchema,
    parameters: z.record(z.string(), z.union([z.string(), z.number(), z.boolean()])),
  })
  .strict();

export type { ProviderKind as AiProvider } from "../../../shared/contracts/generated/ProviderKind";
export type { ExecutionCapability as AiModelCapability } from "../../../shared/contracts/generated/ExecutionCapability";
export type { RoutingPurpose as AiRoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";
export type { AiModelDefinition } from "../../../shared/contracts/generated/AiModelDefinition";
export type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
export type { ExecutionTargetRef as ExecutionTarget } from "../../../shared/contracts/generated/ExecutionTargetRef";
export type { AiRoutingDefaults } from "../../../shared/contracts/generated/AiRoutingDefaults";
export type GenerationPreset = z.infer<typeof generationPresetSchema>;

export interface ResolvedAiModelTarget {
  type: "model";
  connection: AiConnectionSettings;
  model: AiModelDefinition;
}

export const aiConnectionSecretKey = (connectionId: string) =>
  `integrations.ai.connections.${connectionId}.api-key`;
