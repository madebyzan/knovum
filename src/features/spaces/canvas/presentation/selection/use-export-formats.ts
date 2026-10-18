import { queryOptions, useQuery } from "@tanstack/vue-query";
import { computed, type Ref } from "vue";

import type { CanvasSelectionExport } from "../use-canvas-selection-export";

/**
 * Asks what the selection can be saved as, once the formats are being looked
 * at. Nothing is asked of the engine while the menu stays closed.
 */
export function useExportFormats(
  target: Readonly<Ref<CanvasSelectionExport | undefined>>,
  open: Readonly<Ref<boolean>>,
) {
  return useQuery(
    computed(() =>
      queryOptions({
        queryKey: ["spaces", "export-formats", target.value?.key ?? "none"] as const,
        queryFn: () => target.value?.formats() ?? [],
        enabled: open.value && target.value !== undefined,
        staleTime: 0,
        retry: false,
        refetchOnWindowFocus: false,
        refetchOnReconnect: false,
      }),
    ),
  );
}
