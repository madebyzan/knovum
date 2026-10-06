import { z } from "zod";

import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import type { AiModelDefinition } from "../../../shared/contracts/generated/AiModelDefinition";
import {
  providerKinds,
  executionCapabilities,
  routingPurposes,
  chatModelProtocols,
} from "../../../shared/contracts/generated/contract-values";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import { externalAgentKindSchema } from "./external-agents";

export const aiProviderSchema = z.enum(providerKinds);
export const aiCapabilitySchema = z.enum(executionCapabilities);
export const aiRoutingPurposeSchema = z.enum(routingPurposes);

const builtinAgentModelSettingsSchema = z
  .object({
    protocol: z.enum(chatModelProtocols),
    contextWindow: z.number().int().min(8192).max(2_000_000),
    maxOutputTokens: z.number().int().positive(),
  })
  .strict()
  .refine((profile) => profile.maxOutputTokens < profile.contextWindow, {
    message: "Builtin Agent output limit must be smaller than its context window",
  });

export const aiModelDefinitionSchema = z
  .object({
    id: z.string().trim().min(1),
    name: z.string().trim().min(1),
    model: z.string().trim().min(1),
    capabilities: z.array(aiCapabilitySchema).min(1),
    builtinAgent: builtinAgentModelSettingsSchema.nullable().exactOptional(),
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

const modelTargetSchema = z
  .object({ type: z.literal("model"), connectionId: z.string().min(1), modelId: z.string().min(1) })
  .strict();

const executionTargetSchema = z.discriminatedUnion("type", [
  modelTargetSchema,
  z.object({ type: z.literal("agent"), runtimeKind: externalAgentKindSchema }).strict(),
]);

export const executionSourceSchema = z.discriminatedUnion("type", [
  z
    .object({
      type: z.literal("model"),
      provider: z.string().trim().min(1).max(128),
      model: z.string().trim().min(1).max(256),
    })
    .strict(),
  z.object({ type: z.literal("agent"), runtimeKind: externalAgentKindSchema }).strict(),
]);

export const aiDefaultsSchema = z
  .object({
    defaultTarget: executionTargetSchema.nullable().default(null),
    writing: executionTargetSchema.nullable().default(null),
    knowledge: executionTargetSchema.nullable().default(null),
    visualUnderstanding: executionTargetSchema.nullable().default(null),
    translation: executionTargetSchema.nullable().default(null),
    imageGeneration: executionTargetSchema.nullable().default(null),
    videoGeneration: executionTargetSchema.nullable().default(null),
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

export type GenerationPreset = z.infer<typeof generationPresetSchema>;

export interface ResolvedAiModelTarget {
  type: "model";
  connection: AiConnectionSettings;
  model: AiModelDefinition;
}

export const aiConnectionSecretKey = (connectionId: string) =>
  `integrations.ai.connections.${connectionId}.api-key`;

export type ResolvedAiTaskTarget = ResolvedAiModelTarget | Extract<TaskTargetRef, { type: "agent" }>;
