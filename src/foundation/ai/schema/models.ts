import { z } from "zod";

import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import type { AiModelDefinition } from "../../../shared/contracts/generated/AiModelDefinition";
import {
  providerKinds,
  executionCapabilities,
  chatModelProtocols,
} from "../../../shared/contracts/generated/contract-values";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import { externalAgentKindSchema } from "./external-agents";

export const aiProviderSchema = z.enum(providerKinds);
export const aiCapabilitySchema = z.enum(executionCapabilities);
const INVALID_PROVIDER_URL = "Invalid AI provider URL";

export const aiConnectionBaseUrlSchema = z
  .string()
  .trim()
  .min(1)
  .superRefine((value, context) => {
    if (!URL.canParse(value)) {
      context.addIssue({ code: "custom", message: INVALID_PROVIDER_URL });
      return;
    }
    const url = new URL(value);
    if (
      (url.protocol !== "http:" && url.protocol !== "https:") ||
      !url.hostname ||
      url.username.length > 0 ||
      url.password.length > 0
    ) {
      context.addIssue({ code: "custom", message: INVALID_PROVIDER_URL });
    }
  });

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
    baseUrl: aiConnectionBaseUrlSchema,
    models: z.array(aiModelDefinitionSchema),
  })
  .strict()
  .superRefine((connection, context) => {
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

export interface ResolvedAiModelTarget {
  type: "model";
  connection: AiConnectionSettings;
  model: AiModelDefinition;
}

export function aiConnectionSecretKey(connectionId: string) {
  return `integrations.ai.connections.${connectionId}.api-key`;
}

export type ResolvedAiTaskTarget = ResolvedAiModelTarget | Extract<TaskTargetRef, { type: "agent" }>;
