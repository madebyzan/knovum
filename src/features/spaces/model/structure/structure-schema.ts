import { z } from "zod";

export const MAX_STRUCTURE_ITEMS = 100_000;
export const MAX_STRUCTURE_DEPTH = 64;
export const itemIdSchema = z.string().min(1).max(128).regex(/^\S+$/u);
export const containerTitleSchema = z
  .string()
  .min(1)
  .max(512)
  .refine((title) => title === title.trim());
const base = {
  id: itemIdSchema,
  parentId: itemIdSchema.nullable(),
  order: z
    .number()
    .int()
    .nonnegative()
    .max(MAX_STRUCTURE_ITEMS - 1),
  locked: z.boolean(),
};
export const structureItemSchema = z.discriminatedUnion("kind", [
  z.object({ ...base, kind: z.literal("content"), nodeId: itemIdSchema }).strict(),
  z.object({ ...base, kind: z.literal("group"), title: containerTitleSchema }).strict(),
  z.object({ ...base, kind: z.literal("section"), title: containerTitleSchema }).strict(),
]);
export const spaceStructureSchema = z
  .object({
    version: z.literal(1),
    items: z.array(structureItemSchema).max(MAX_STRUCTURE_ITEMS),
  })
  .strict();
export type StructureItem = z.infer<typeof structureItemSchema>;
export type SpaceStructure = z.infer<typeof spaceStructureSchema>;
