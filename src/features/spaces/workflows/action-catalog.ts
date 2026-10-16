import { z } from "zod";

import { aiCapabilitySchema, aiRoutingPurposeSchema } from "../../../foundation/ai/schema/models";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import {
  spaceActionExecutionModes,
  spaceActionImpacts,
  spaceActionInputRoles,
  spaceActionOutputArities,
  spaceActionOutputContracts,
} from "../contracts/generated/contract-values";
import type { SpaceActionDefinition } from "../contracts/generated/SpaceActionDefinition";

const spaceActionDefinitionSchema = z
  .object({
    id: z.string().min(1),
    version: z.number().int().positive(),
    impact: z.enum(spaceActionImpacts),
    executionMode: z.enum(spaceActionExecutionModes),
    inputConsistency: z.literal("pinned_revision"),
    executionCapability: aiCapabilitySchema.nullable(),
    routingPurpose: aiRoutingPurposeSchema.nullable(),
    inputRoles: z.array(z.enum(spaceActionInputRoles)),
    collectionInput: z
      .object({ kind: z.string(), role: z.string(), maxItems: z.number().int().positive() })
      .nullable(),
    parametersSchema: z.record(z.string(), z.unknown()),
    outputContract: z.enum(spaceActionOutputContracts),
    outputArity: z.enum(spaceActionOutputArities),
  })
  .strict() satisfies z.ZodType<SpaceActionDefinition>;

let catalog: Promise<SpaceActionDefinition[]> | undefined;

export function listSpaceActions(): Promise<SpaceActionDefinition[]> {
  catalog ??= invokeCommand<unknown>("plugin:spaces|list_space_actions").then((value) =>
    z.array(spaceActionDefinitionSchema).parse(value),
  );
  return catalog;
}
