import { z } from "zod";

export const groundedScopeSchema = z.enum(["selection", "section", "space", "all-spaces"]);

export type GroundedScope = z.infer<typeof groundedScopeSchema>;
