import { z } from "zod";

export const aiProviderSchema = z.enum([
  "openai",
  "openai-compatible",
  "openrouter",
  "deepseek",
  "anthropic",
  "gemini",
  "qwen",
  "zhipu",
  "siliconflow",
  "kling",
  "grok",
  "groq",
  "mistral",
  "together",
  "stepfun",
  "moonshot",
  "doubao",
  "minimax",
  "ollama",
]);

export const aiCapabilitySchema = z.enum([
  "text-generation",
  "vision-understanding",
  "image-generation",
  "video-generation",
]);

export const aiRoutingPurposeSchema = z.enum([
  "writing",
  "knowledge",
  "visual-understanding",
  "translation",
  "image-generation",
  "video-generation",
]);

const aiModelDefinitionSchema = z
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
    models: z.array(aiModelDefinitionSchema).min(1),
  })
  .strict()
  .superRefine((connection, context) => {
    if (URL.canParse(connection.baseUrl) === false) {
      context.addIssue({ code: "custom", path: ["baseUrl"], message: "Invalid AI provider URL" });
    }
    if (new Set(connection.models.map(({ id }) => id)).size !== connection.models.length) {
      context.addIssue({ code: "custom", path: ["models"], message: "Duplicate AI model id" });
    }
  });

const aiModelTargetSchema = z
  .object({ type: z.literal("model"), connectionId: z.string().min(1), modelId: z.string().min(1) })
  .strict();

const executionTargetSchema = aiModelTargetSchema;

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

export type AiProvider = z.infer<typeof aiProviderSchema>;
export type AiModelCapability = z.infer<typeof aiCapabilitySchema>;
export type AiRoutingPurpose = z.infer<typeof aiRoutingPurposeSchema>;
export type AiModelDefinition = z.infer<typeof aiModelDefinitionSchema>;
export type AiConnectionSettings = z.infer<typeof aiConnectionSettingsSchema>;
export type AiModelTarget = z.infer<typeof aiModelTargetSchema>;
export type ExecutionTarget = z.infer<typeof executionTargetSchema>;
export type AiRoutingDefaults = z.infer<typeof aiDefaultsSchema>;
export type GenerationPreset = z.infer<typeof generationPresetSchema>;

export interface ResolvedAiModelTarget {
  type: "model";
  connection: AiConnectionSettings;
  model: AiModelDefinition;
}

export const aiConnectionSecretKey = (connectionId: string) =>
  `integrations.ai.connections.${connectionId}.api-key`;
