import { z } from "zod";

import { groundedMessageSchema } from "../knowledge/model/grounded-message";
import { groundedThreadSchema } from "../knowledge/model/grounded-thread";
import { SPACE_PACKAGE_FORMAT, SPACE_PACKAGE_VERSION } from "./contract";
import { nodeSchema, packageTimeSchema, relationSchema, sceneSchema, spaceSchema } from "./core-schema";
import { derivationSchema } from "./derivation-schema";
import { eventSchema, objectTypeSchema, provenanceSchema } from "./knowledge-contract-schema";
import { chunkSchema } from "./knowledge-schema";
import {
  packageLayoutsSchema,
  packageRelationStylesSchema,
  packageStructureNodesSchema,
} from "./structure-schema";
import { workflowRunSchema } from "./workflow-run-schema";

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
    creationWorkflowRuns: z.array(workflowRunSchema).max(100_000),
    chunks: z.array(chunkSchema).max(1_000_000),
    aiThreads: z.array(groundedThreadSchema).max(100_000),
    aiMessages: z.array(groundedMessageSchema).max(1_000_000),
    objectTypes: z.array(objectTypeSchema).max(10_000),
    provenance: z.array(provenanceSchema).max(1_000_000),
    knowledgeEvents: z.array(eventSchema).max(2_000_000),
  })
  .strict();

export type SpacePackageManifest = z.infer<typeof manifestSchema>;
