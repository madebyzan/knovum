import { Aperture, Blend, CircleDotDashed, Contrast } from "@lucide/vue";

import type { ImageAdjustments } from "../../image-editing/image-edit";

export const imageAdjustmentGroups = [
  { id: "light", icon: Contrast, keys: ["exposure", "contrast", "highlights", "shadows", "whites", "blacks"] },
  { id: "color", icon: Blend, keys: ["vibrance", "saturation", "warmth", "tint"] },
  { id: "detail", icon: CircleDotDashed, keys: ["sharpness", "clarity", "grain"] },
  { id: "effects", icon: Aperture, keys: ["vignette", "softLight", "glow"] },
] as const satisfies ReadonlyArray<{
  id: string;
  icon: typeof Aperture;
  keys: readonly (keyof ImageAdjustments)[];
}>;

export type ImageAdjustmentGroupId = (typeof imageAdjustmentGroups)[number]["id"];
