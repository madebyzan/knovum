import { queryOptions, useQuery } from "@tanstack/vue-query";
import { computed, type Ref } from "vue";

import type { ExportTarget } from "./target";

/**
 * Asks what a target can be saved as, once the formats are being looked at.
 * Nothing is asked of the engine while the menu stays closed.
 */
export function useExportFormats(
  target: Readonly<Ref<ExportTarget | undefined>>,
  open: Readonly<Ref<boolean>>,
) {
  return useQuery(
    computed(() =>
      queryOptions({
        queryKey: ["spaces", "export-formats", target.value?.key ?? "none"] as const,
        queryFn: () => target.value?.formats() ?? { formats: [], unavailable: null },
        enabled: open.value && target.value !== undefined,
        staleTime: 0,
        retry: false,
        refetchOnWindowFocus: false,
        refetchOnReconnect: false,
      }),
    ),
  );
}
