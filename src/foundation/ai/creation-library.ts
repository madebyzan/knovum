import { z } from "zod";

import { generationPresetSchema } from "./ai-models";

export const creationLibrarySchema = z
  .object({
    generationPresets: z.array(generationPresetSchema),
  })
  .strict();

export type CreationLibrary = z.infer<typeof creationLibrarySchema>;

export const defaultCreationLibrary: CreationLibrary = {
  generationPresets: [],
};
