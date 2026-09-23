import { z } from "zod";

const normalized = z.number().min(0).max(1);

export const imageCropSchema = z
  .object({
    x: normalized,
    y: normalized,
    width: z.number().positive().max(1),
    height: z.number().positive().max(1),
  })
  .strict()
  .refine(({ x, width }) => x + width <= 1, "Crop exceeds the image width")
  .refine(({ y, height }) => y + height <= 1, "Crop exceeds the image height");

const imageEraserStrokeSchema = z
  .object({
    diameter: z.number().positive().max(0.5),
    points: z
      .array(normalized)
      .min(4)
      .max(4096)
      .refine((points) => points.length % 2 === 0),
  })
  .strict();

export const imageEditSchema = z
  .object({
    version: z.literal(1),
    crop: imageCropSchema,
    rotation: z.union([z.literal(0), z.literal(90), z.literal(180), z.literal(270)]),
    flipHorizontal: z.boolean(),
    flipVertical: z.boolean(),
    adjustments: z
      .object({
        brightness: z.number().min(-100).max(100),
        contrast: z.number().min(-100).max(100),
        saturation: z.number().min(-100).max(100),
      })
      .strict(),
    eraserStrokes: z.array(imageEraserStrokeSchema).max(500),
  })
  .strict();
