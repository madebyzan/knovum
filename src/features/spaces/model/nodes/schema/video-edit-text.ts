import { z } from "zod";

const hexColor = z.string().regex(/^#[0-9A-Fa-f]{6}$/);

const videoTextStyleSchema = z
  .object({
    family: z.string().min(1).max(128),
    weight: z.number().int().min(100).max(900),
    italic: z.boolean(),
    size: z.number().int().min(12).max(200),
    color: hexColor,
    align: z.enum(["left", "center", "right"]),
    decoration: z.enum(["plain", "outline", "shadow", "box"]),
    decorationStrength: z.number().int().min(0).max(20),
    decorationColor: hexColor,
  })
  .strict();

export const videoTextOverlaySchema = z
  .object({
    id: z.string().uuid(),
    startMs: z.number().int().nonnegative().max(86_400_000),
    endMs: z.number().int().positive().max(86_400_000),
    text: z
      .string()
      .min(1)
      .max(500)
      .refine((value) => !!value.trim(), "Text cannot be blank"),
    x: z.number().min(-0.5).max(1.5),
    y: z.number().min(-0.5).max(1.5),
    style: videoTextStyleSchema,
  })
  .strict()
  .refine(({ startMs, endMs }) => endMs - startMs >= 100, "Timed text needs at least 100 ms");

export const videoOutputColor = hexColor;
