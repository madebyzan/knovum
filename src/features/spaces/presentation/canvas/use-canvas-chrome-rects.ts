import { useElementBounding } from "@vueuse/core";
import { computed, useTemplateRef } from "vue";

import type { SurfaceRect } from "./selection-surface-geometry";

/** Reserve only the space occupied by fixed canvas controls, in canvas viewport pixels. */
export function useCanvasChromeRects() {
  const chromeRoot = useTemplateRef<HTMLElement>("chromeRoot");
  const topChrome = useTemplateRef<HTMLElement>("topChrome");
  const focusChrome = useTemplateRef<HTMLElement>("focusChrome");
  const viewChrome = useTemplateRef<HTMLElement>("viewChrome");
  const root = useElementBounding(chromeRoot);
  const controls = [topChrome, focusChrome, viewChrome].map((element) => useElementBounding(element));
  const chromeRects = computed<SurfaceRect[]>(() =>
    controls
      .filter(({ width, height }) => width.value > 0 && height.value > 0)
      .map(({ x, y, width, height }) => ({
        left: x.value - root.x.value - 6,
        top: y.value - root.y.value - 6,
        right: x.value - root.x.value + width.value + 6,
        bottom: y.value - root.y.value + height.value + 6,
      })),
  );
  return { chromeRects };
}
