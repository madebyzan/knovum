import { z } from "zod";

import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import { validateStructureSnapshot } from "../structure-validation";
import { structureLayoutSchema, MAX_STRUCTURE_NODES, structureNodeSchema } from "./structure";

const relationStyleSchema = z
  .object({
    relationId: z.string().min(1).max(128),
    color: z.enum(["blue", "green", "amber", "red", "purple"]),
  })
  .strict();

const spaceEditSnapshotSchema = z
  .object({
    editRevision: z.number().int().nonnegative(),
    structure: z.array(structureNodeSchema).max(MAX_STRUCTURE_NODES),
    layouts: z.array(structureLayoutSchema).max(MAX_STRUCTURE_NODES),
    relationStyles: z.array(relationStyleSchema).max(MAX_STRUCTURE_NODES),
  })
  .strict()
  .superRefine((snapshot, context) => {
    try {
      validateStructureSnapshot(snapshot.structure, snapshot.layouts);
    } catch (error) {
      context.addIssue({ code: "custom", message: getErrorMessage(error) });
    }
    const relationIds = new Set<string>();
    for (const [index, style] of snapshot.relationStyles.entries()) {
      if (relationIds.has(style.relationId))
        context.addIssue({
          code: "custom",
          path: ["relationStyles", index, "relationId"],
          message: "Duplicate relation style",
        });
      relationIds.add(style.relationId);
    }
  });

export type SpaceEditSnapshot = z.infer<typeof spaceEditSnapshotSchema>;

export function parseSpaceEditSnapshot(source: unknown): SpaceEditSnapshot {
  return spaceEditSnapshotSchema.parse(source);
}
