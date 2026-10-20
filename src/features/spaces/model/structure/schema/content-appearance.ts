import { z } from "zod";

export const CANVAS_TYPOGRAPHY_SCALE_MIN = 0.25;
export const CANVAS_TYPOGRAPHY_SCALE_MAX = 4;
export const CANVAS_TEXT_COLORS = ["ink", "muted", "red", "blue", "green", "purple"] as const;
export const CANVAS_FILL_PRESETS = ["yellow", "green", "blue", "pink", "gray"] as const;

/** The same content appearance is validated in stored structure and canvas projections. */
export const contentAppearanceSchema = z
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
