import { z } from "zod";

import { creationWorkflowSchema, generationPresetSchema } from "./ai-models";

export const creationLibrarySchema = z
  .object({
    generationPresets: z.array(generationPresetSchema),
    creationWorkflows: z.array(creationWorkflowSchema),
  })
  .strict();

export type CreationLibrary = z.infer<typeof creationLibrarySchema>;

export const defaultCreationLibrary: CreationLibrary = {
  generationPresets: [],
  creationWorkflows: [],
};
