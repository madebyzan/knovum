import { z } from "zod";

import {
  structureLayoutSchema,
  MAX_STRUCTURE_NODES,
  structureNodeSchema,
} from "../model/structure/structure-contract";

export const packageStructureNodesSchema = z.array(structureNodeSchema).max(MAX_STRUCTURE_NODES);
export const packageLayoutsSchema = z.array(structureLayoutSchema).max(MAX_STRUCTURE_NODES);
export const packageRelationStyleSchema = z
  .object({
    relationId: z.string().min(1).max(128),
    color: z.enum(["blue", "green", "amber", "red", "purple"]),
  })
  .strict();
export const packageRelationStylesSchema = z.array(packageRelationStyleSchema).max(MAX_STRUCTURE_NODES);

export type PackageRelationStyle = z.infer<typeof packageRelationStyleSchema>;
