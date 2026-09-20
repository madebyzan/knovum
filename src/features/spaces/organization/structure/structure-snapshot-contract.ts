import { z } from "zod";

import { structureLayoutSchema } from "../../canvas/contract/structure-layout";
import { spaceStructureSchema } from "../../model/structure/structure-schema";

export const structureSnapshotSchema = z
  .object({ structure: spaceStructureSchema, layout: structureLayoutSchema })
  .strict();
export type StructureSnapshot = z.infer<typeof structureSnapshotSchema>;
