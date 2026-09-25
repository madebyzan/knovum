import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import { aiCapabilitySchema, aiRoutingPurposeSchema } from "./ai-models";

const spaceActionInputRoleSchema = z.enum(["document", "knowledge", "subtitle", "transcript", "video"]);

const spaceActionOutputContractSchema = z.enum([
  "answer",
  "document",
  "image",
  "knowledge_change_receipt",
  "knowledge_node",
  "presentation",
  "resource_receipt",
  "search_results",
  "space_edit_receipt",
  "subtitle",
  "video",
]);

const spaceActionDefinitionSchema = z
  .object({
    id: z.string().min(1),
    version: z.number().int().positive(),
    impact: z.enum(["read_only", "creates_resource", "mutates_resource"]),
    executionMode: z.enum(["immediate", "durable"]),
    approvalPolicy: z.enum(["task_grant", "exact_plan"]),
    inputConsistency: z.literal("pinned_revision"),
    executionCapability: aiCapabilitySchema.nullable(),
    routingPurpose: aiRoutingPurposeSchema.nullable(),
    inputRoles: z.array(spaceActionInputRoleSchema),
    collectionInput: z
      .object({ kind: z.string(), role: z.string(), maxItems: z.number().int().positive() })
      .nullable(),
    parametersSchema: z.record(z.string(), z.unknown()),
    outputContract: spaceActionOutputContractSchema,
  })
  .strict();

export type SpaceActionDefinition = z.infer<typeof spaceActionDefinitionSchema>;

let catalog: Promise<SpaceActionDefinition[]> | undefined;

export function listSpaceActions(): Promise<SpaceActionDefinition[]> {
  catalog ??= invoke<unknown>("plugin:spaces|list_space_actions").then((value) =>
    z.array(spaceActionDefinitionSchema).parse(value),
  );
  return catalog;
}
