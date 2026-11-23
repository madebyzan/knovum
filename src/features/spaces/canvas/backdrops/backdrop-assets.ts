import type { CanvasBackdrop } from "../../../../shared/contracts/generated/CanvasBackdrop";
import auroraSilkBackdrop from "./aurora-silk.webp";
import duskTideBackdrop from "./dusk-tide.webp";
import morningMistBackdrop from "./morning-mist.webp";
import quietCanopyBackdrop from "./quiet-canopy.webp";
import summerGlowBackdrop from "./summer-glow.webp";

export type VisibleCanvasBackdrop = Exclude<CanvasBackdrop, "none">;

const BACKDROP_ASSET_BY_PRESET: Readonly<Record<VisibleCanvasBackdrop, string>> = {
  "morning-mist": morningMistBackdrop,
  "summer-glow": summerGlowBackdrop,
  "aurora-silk": auroraSilkBackdrop,
  "quiet-canopy": quietCanopyBackdrop,
  "dusk-tide": duskTideBackdrop,
};

export function getCanvasBackdropAsset(backdrop: VisibleCanvasBackdrop): string {
  return BACKDROP_ASSET_BY_PRESET[backdrop];
}
