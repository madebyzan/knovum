import { z } from "zod";

export const MAX_CANVAS_Z_INDEX = 0xffff_ffff;
export const CANVAS_TYPOGRAPHY_SCALE_MIN = 0.25;
export const CANVAS_TYPOGRAPHY_SCALE_MAX = 4;

const finiteNumber = z.number().finite();
export const CANVAS_TEXT_COLORS = ["ink", "muted", "red", "blue", "green", "purple"] as const;
export const CANVAS_FILL_PRESETS = ["yellow", "green", "blue", "pink", "gray"] as const;
const canvasNodeAppearanceSchema = z
  .object({
    fill: z.enum(CANVAS_FILL_PRESETS).optional(),
    customFill: z
      .string()
      .regex(/^#[\da-f]{6}$/iu)
      .optional(),
    textColor: z.enum(CANVAS_TEXT_COLORS).optional(),
    fontSize: z.number().finite().min(2.5).max(576).optional(),
    typographyScale: z
      .number()
      .finite()
      .min(CANVAS_TYPOGRAPHY_SCALE_MIN)
      .max(CANVAS_TYPOGRAPHY_SCALE_MAX)
      .optional(),
    fontWeight: z.enum(["regular", "bold"]).optional(),
    fontStyle: z.enum(["normal", "italic"]).optional(),
    textDecoration: z.enum(["none", "underline"]).optional(),
    textAlign: z.enum(["left", "center", "right"]).optional(),
    verticalAlign: z.enum(["top", "middle", "bottom"]).optional(),
    background: z.string().optional(),
    backgroundStyle: z.enum(["cover", "ratio", "repeat"]).optional(),
  })
  .strict()
  .refine(({ fill, customFill }) => !(fill && customFill), {
    message: "fill and customFill are mutually exclusive",
  });
const canvasNodeSizingModeSchema = z.enum(["content", "frame"]);
const canvasFrameSchema = z
  .object({
    x: finiteNumber,
    y: finiteNumber,
    width: finiteNumber.positive(),
    height: finiteNumber.positive(),
    rotation: finiteNumber,
  })
  .strict();
const canvasNodeLayoutSchema = z
  .object({
    nodeId: z.string().min(1),
    frame: canvasFrameSchema,
    sizingMode: canvasNodeSizingModeSchema,
    appearance: canvasNodeAppearanceSchema.optional(),
    fileSubpath: z.string().startsWith("#").optional(),
    zIndex: z.number().int().nonnegative().max(MAX_CANVAS_Z_INDEX),
  })
  .strict();
export const CANVAS_RELATION_COLORS = ["neutral", "blue", "green", "amber", "red", "purple"] as const;
export const CANVAS_RELATION_OVERRIDE_COLORS = ["blue", "green", "amber", "red", "purple"] as const;
const canvasRelationStyleSchema = z
  .object({
    relationId: z.string().min(1),
    color: z.enum(CANVAS_RELATION_OVERRIDE_COLORS),
  })
  .strict();
export const canvasDocumentSchema = z
  .object({
    version: z.literal(1),
    viewport: z.object({ x: finiteNumber, y: finiteNumber, zoom: finiteNumber.positive() }).strict(),
    nodes: z.array(canvasNodeLayoutSchema).max(100_000),
    relationStyles: z.array(canvasRelationStyleSchema).max(100_000).optional(),
  })
  .strict()
  .superRefine(validateCanvasReferences);

export type CanvasNodeFrame = z.infer<typeof canvasFrameSchema>;
export type CanvasNodeSizingMode = z.infer<typeof canvasNodeSizingModeSchema>;
export type CanvasNodeLayoutV1 = z.infer<typeof canvasNodeLayoutSchema>;
export type CanvasNodeAppearance = z.infer<typeof canvasNodeAppearanceSchema>;
export type CanvasDocumentV1 = z.infer<typeof canvasDocumentSchema>;
export type CanvasRelationColor = (typeof CANVAS_RELATION_COLORS)[number];
export type CanvasRelationOverrideColor = (typeof CANVAS_RELATION_OVERRIDE_COLORS)[number];

export interface CanvasNodeFrameUpdate {
  nodeId: string;
  frame: CanvasNodeFrame;
  sizingMode?: CanvasNodeSizingMode;
}

export function parseCanvasDocument(source: unknown): CanvasDocumentV1 {
  return canvasDocumentSchema.parse(source);
}

function validateCanvasReferences(document: CanvasDocumentV1, context: z.RefinementCtx) {
  const nodes = new Map<string, CanvasNodeLayoutV1>();
  for (const [index, node] of document.nodes.entries()) {
    if (nodes.has(node.nodeId)) {
      context.addIssue({
        code: "custom",
        path: ["nodes", index, "nodeId"],
        message: "nodeId must be unique",
      });
    }
    nodes.set(node.nodeId, node);
  }
  const relationIds = new Set<string>();
  for (const [index, relation] of (document.relationStyles ?? []).entries()) {
    if (relationIds.has(relation.relationId)) {
      context.addIssue({
        code: "custom",
        path: ["relationStyles", index, "relationId"],
        message: "relationId must be unique",
      });
    }
    relationIds.add(relation.relationId);
  }
}

export function serializeCanvasAppearance(appearance: CanvasNodeAppearance): Record<string, string | number> {
  const serialized: Record<string, string | number> = {};
  for (const [key, value] of Object.entries(appearance)) {
    if (value !== undefined) serialized[key] = value;
  }
  return serialized;
}
