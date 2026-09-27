import { z } from "zod";

import { SECTION_MINIMUM_SIZE } from "../section-metrics";

export const MAX_STRUCTURE_NODES = 100_000;
export const MAX_STRUCTURE_DEPTH = 64;
export const MAX_WORLD_COORDINATE = 1_000_000_000;
export const MAX_WORLD_SCALE = 1_000_000;

const structureNodeId = z.string().min(1).max(128);
const coordinate = z.number().finite().min(-MAX_WORLD_COORDINATE).max(MAX_WORLD_COORDINATE);
const positiveSize = z.number().finite().positive().max(MAX_WORLD_COORDINATE);
const structureBase = {
  id: structureNodeId,
  parentId: structureNodeId.nullable(),
  order: z.number().int().nonnegative().max(MAX_STRUCTURE_NODES),
  locked: z.boolean(),
};

export const structureNodeSchema = z.discriminatedUnion("kind", [
  z.object({ ...structureBase, kind: z.literal("content"), nodeId: structureNodeId }).strict(),
  z.object({ ...structureBase, kind: z.literal("group"), title: z.string().trim().min(1).max(512) }).strict(),
  z
    .object({ ...structureBase, kind: z.literal("section"), title: z.string().trim().min(1).max(512) })
    .strict(),
]);

export const localTransformSchema = z
  .object({
    x: coordinate,
    y: coordinate,
    rotation: coordinate,
    scale: z
      .number()
      .finite()
      .min(1 / MAX_WORLD_SCALE)
      .max(MAX_WORLD_SCALE),
  })
  .strict();

const size = z.object({ width: positiveSize, height: positiveSize }).strict();
const sectionSize = z
  .object({
    width: positiveSize.min(SECTION_MINIMUM_SIZE.width),
    height: positiveSize.min(SECTION_MINIMUM_SIZE.height),
  })
  .strict();
const layoutBase = { structureNodeId };
const contentAppearance = z
  .object({
    fill: z.enum(["yellow", "green", "blue", "pink", "gray"]).optional(),
    customFill: z
      .string()
      .regex(/^#[\da-f]{6}$/iu)
      .optional(),
    textColor: z.enum(["ink", "muted", "red", "blue", "green", "purple"]).optional(),
    fontSize: z.number().finite().min(2.5).max(576).optional(),
    typographyScale: z.number().finite().min(0.25).max(4).optional(),
    fontWeight: z.enum(["regular", "bold"]).optional(),
    fontStyle: z.enum(["normal", "italic"]).optional(),
    textDecoration: z.enum(["none", "underline"]).optional(),
    textAlign: z.enum(["left", "center", "right"]).optional(),
    verticalAlign: z.enum(["top", "middle", "bottom"]).optional(),
    background: z.string().optional(),
    backgroundStyle: z.enum(["cover", "ratio", "repeat"]).optional(),
  })
  .strict()
  .refine(({ fill, customFill }) => !(fill && customFill));
const sectionAppearance = z
  .object({
    fill: z.enum(["yellow", "green", "blue", "pink", "gray"]).optional(),
    customFill: z
      .string()
      .regex(/^#[\da-f]{6}$/iu)
      .optional(),
    borderStyle: z.enum(["solid", "dashed"]).optional(),
    background: z.string().optional(),
    backgroundStyle: z.enum(["cover", "ratio", "repeat"]).optional(),
  })
  .strict()
  .refine(({ fill, customFill }) => !(fill && customFill));

export const structureLayoutSchema = z.discriminatedUnion("kind", [
  z
    .object({
      ...layoutBase,
      kind: z.literal("content"),
      transform: localTransformSchema,
      size,
      sizeAuthority: z.enum(["content", "frame"]),
      appearance: contentAppearance.optional(),
      fileSubpath: z.string().startsWith("#").optional(),
    })
    .strict(),
  z.object({ ...layoutBase, kind: z.literal("group"), transform: localTransformSchema }).strict(),
  z
    .object({
      ...layoutBase,
      kind: z.literal("section"),
      position: z.object({ x: coordinate, y: coordinate }).strict(),
      size: sectionSize,
      collapsed: z.boolean(),
      appearance: sectionAppearance.optional(),
    })
    .strict(),
]);

export type StructureNodeId = z.infer<typeof structureNodeId>;
export type StructureNode = z.infer<typeof structureNodeSchema>;
export type StructureLayout = z.infer<typeof structureLayoutSchema>;
export type LocalTransform = z.infer<typeof localTransformSchema>;
