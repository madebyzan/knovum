import { z } from "zod";

import { itemIdSchema, MAX_STRUCTURE_ITEMS } from "../../model/structure/structure-schema";
import { canvasNodeAppearanceSchema } from "./canvas-appearance";

export const WORLD_COORDINATE_LIMIT = 1_000_000_000_000;
export const WORLD_SCALE_MIN = 0.000001;
export const WORLD_SCALE_MAX = 1_000_000;
const coordinate = z.number().finite().min(-WORLD_COORDINATE_LIMIT).max(WORLD_COORDINATE_LIMIT);
export const structurePointSchema = z.object({ x: coordinate, y: coordinate }).strict();
export const structureSizeSchema = z
  .object({
    width: z.number().finite().positive().max(WORLD_COORDINATE_LIMIT),
    height: z.number().finite().positive().max(WORLD_COORDINATE_LIMIT),
  })
  .strict();
export const structureTransformSchema = z
  .object({
    x: coordinate,
    y: coordinate,
    rotation: z.number().finite().min(-180).lt(180),
    scale: z.number().finite().min(WORLD_SCALE_MIN).max(WORLD_SCALE_MAX),
  })
  .strict();
export const structureItemLayoutSchema = z.discriminatedUnion("kind", [
  z
    .object({
      kind: z.literal("content"),
      itemId: itemIdSchema,
      transform: structureTransformSchema,
      size: structureSizeSchema,
      sizeAuthority: z.enum(["content", "frame"]),
      appearance: canvasNodeAppearanceSchema.optional(),
      fileSubpath: z.string().startsWith("#").optional(),
    })
    .strict(),
  z.object({ kind: z.literal("group"), itemId: itemIdSchema, transform: structureTransformSchema }).strict(),
  z
    .object({
      kind: z.literal("section"),
      itemId: itemIdSchema,
      position: structurePointSchema,
      size: structureSizeSchema,
      collapsed: z.boolean(),
      appearance: canvasNodeAppearanceSchema.optional(),
    })
    .strict(),
]);
export const structureLayoutSchema = z
  .object({
    version: z.literal(1),
    items: z.array(structureItemLayoutSchema).max(MAX_STRUCTURE_ITEMS),
  })
  .strict();
export type StructurePoint = z.infer<typeof structurePointSchema>;
export type StructureSize = z.infer<typeof structureSizeSchema>;
export type StructureTransform = z.infer<typeof structureTransformSchema>;
export type StructureItemLayout = z.infer<typeof structureItemLayoutSchema>;
export type StructureLayout = z.infer<typeof structureLayoutSchema>;
export type StructureBounds = StructurePoint & StructureSize;
