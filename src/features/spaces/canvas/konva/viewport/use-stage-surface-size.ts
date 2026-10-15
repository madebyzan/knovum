import { useDebounceFn } from "@vueuse/core";
import { reactive, shallowRef, watch, type Reactive } from "vue";

const SURFACE_STEP = 256;
const SHRINK_SETTLE_MS = 220;

function quantize(extent: number) {
  return Math.max(SURFACE_STEP, Math.ceil(extent / SURFACE_STEP) * SURFACE_STEP);
}

/**
 * Extent of the stage surface: the raster of every Konva layer, the grid, and the region culling admits.
 *
 * The surface covers the largest extent its host can take - the window - rather than the host itself, so
 * a work-area edge moving under a sidebar is a clip and never a resize. Konva reallocates and fully
 * repaints every layer's scene and hit canvas whenever the stage is resized, and culling against the host
 * mounts and unmounts node content as an edge sweeps across it; bounding the surface by the window removes
 * both from panel motion and sash drags entirely. Only the window resizes the surface: it grows immediately,
 * which keeps newly exposed area painted, and shrinks once the window has settled. Quantizing keeps a
 * window drag to one reallocation per step.
 */
export function useStageSurfaceSize(extent: Reactive<{ height: number; width: number }>) {
  const width = shallowRef(quantize(extent.width));
  const height = shallowRef(quantize(extent.height));
  const shrink = useDebounceFn(() => {
    width.value = quantize(extent.width);
    height.value = quantize(extent.height);
  }, SHRINK_SETTLE_MS);

  watch([() => extent.width, () => extent.height], ([nextWidth, nextHeight]) => {
    width.value = Math.max(width.value, quantize(nextWidth));
    height.value = Math.max(height.value, quantize(nextHeight));
    void shrink();
  });

  return reactive({ width, height });
}
