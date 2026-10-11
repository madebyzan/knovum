import { z } from "zod";

import type { CreationLibrary as PersistedCreationLibrary } from "../../../shared/contracts/generated/CreationLibrary";
import { generationPresetSchema } from "./models";

export const creationLibrarySchema = z
  .object({
    generationPresets: z.array(generationPresetSchema),
  })
  .strict()
  .refine(
    (library) =>
      new Set(library.generationPresets.map(({ id }) => id)).size === library.generationPresets.length,
    {
      message: "Duplicate generation preset id",
      path: ["generationPresets"],
    },
  ) satisfies z.ZodType<PersistedCreationLibrary>;

export type CreationLibrary = z.infer<typeof creationLibrarySchema>;

export const DEFAULT_CREATION_LIBRARY: CreationLibrary = {
  generationPresets: [],
};
