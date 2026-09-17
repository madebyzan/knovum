import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import { aiCapabilitySchema, aiRoutingPurposeSchema } from "./ai-models";

const knowledgeActionDefinitionSchema = z
  .object({
    id: z.string().min(1),
    version: z.number().int().positive(),
    executionCapability: aiCapabilitySchema.nullable(),
    routingPurpose: aiRoutingPurposeSchema.nullable(),
    inputRoles: z.array(z.string().min(1)).min(1),
    parameters: z.array(z.object({ key: z.string().min(1), kind: z.enum(["text", "number"]) }).strict()),
    outputContract: z.string().min(1),
  })
  .strict();

export type KnowledgeActionDefinition = z.infer<typeof knowledgeActionDefinitionSchema>;

let definitions: Promise<KnowledgeActionDefinition[]> | undefined;

export function loadKnowledgeActions(): Promise<KnowledgeActionDefinition[]> {
  definitions ??= invoke<unknown>("plugin:spaces|list_knowledge_actions").then((value) =>
    z.array(knowledgeActionDefinitionSchema).parse(value),
  );
  return definitions;
}
