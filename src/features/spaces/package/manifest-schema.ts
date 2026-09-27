import { z } from "zod";

import { relationSchema } from "../model/relations/relation-schema";
import { sceneSchema } from "../scenes/scene-schema";
import { packageAssistantConversationSchema, packageAssistantMessageSchema } from "./assistant-schema";
import { SPACE_PACKAGE_FORMAT, SPACE_PACKAGE_VERSION } from "./contract";
import { nodeSchema, packageTimeSchema, spaceSchema } from "./core-schema";
import { derivationSchema } from "./derivation-schema";
import {
  packageExternalAgentConversationSchema,
  packageExternalAgentTurnSchema,
} from "./external-conversation-schema";
import { eventSchema, objectTypeSchema, provenanceSchema } from "./knowledge-contract-schema";
import { chunkSchema } from "./knowledge-schema";
import {
  packageRunAttemptSchema,
  packageRunEventSchema,
  packageRunSchema,
  packageRunStepSchema,
} from "./run-schema";
import {
  packageLayoutsSchema,
  packageRelationStylesSchema,
  packageStructureNodesSchema,
} from "./structure-schema";

export const manifestSchema = z
  .object({
    format: z.literal(SPACE_PACKAGE_FORMAT),
    version: z.literal(SPACE_PACKAGE_VERSION),
    exportedAt: packageTimeSchema,
    space: spaceSchema,
    structureNodes: packageStructureNodesSchema,
    layouts: packageLayoutsSchema,
    relationStyles: packageRelationStylesSchema,
    nodes: z.array(nodeSchema).max(100_000),
    scenes: z.array(sceneSchema).max(10_000),
    relations: z.array(relationSchema).max(200_000),
    derivations: z.array(derivationSchema).max(100_000),
    runs: z.array(packageRunSchema).max(100_000),
    runSteps: z.array(packageRunStepSchema).max(1_000_000),
    runAttempts: z.array(packageRunAttemptSchema).max(2_000_000),
    runEvents: z.array(packageRunEventSchema).max(5_000_000),
    externalAgentConversations: z.array(packageExternalAgentConversationSchema).max(100_000),
    externalAgentTurns: z.array(packageExternalAgentTurnSchema).max(1_000_000),
    assistantConversations: z.array(packageAssistantConversationSchema).max(100_000),
    assistantMessages: z.array(packageAssistantMessageSchema).max(1_000_000),
    chunks: z.array(chunkSchema).max(1_000_000),
    objectTypes: z.array(objectTypeSchema).max(10_000),
    provenance: z.array(provenanceSchema).max(1_000_000),
    knowledgeEvents: z.array(eventSchema).max(2_000_000),
  })
  .strict();

export type SpacePackageManifest = z.infer<typeof manifestSchema>;
