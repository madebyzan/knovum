import { today, getLocalTimeZone } from "@internationalized/date";
import { useQuery } from "@tanstack/vue-query";
import { useIntervalFn } from "@vueuse/core";
import { computed, shallowRef } from "vue";

import { loadTokenUsageReport } from "../../../foundation/ai/token-usage";

export function useTokenUsage() {
  const to = shallowRef(today(getLocalTimeZone()));
  useIntervalFn(() => {
    const current = today(getLocalTimeZone());
    if (to.value.compare(current) !== 0) to.value = current;
  }, 60_000);
  const interval = computed(() => ({
    fromDate: to.value.subtract({ days: 364 }).toString(),
    toDate: to.value.toString(),
  }));
  return useQuery({
    queryKey: computed(() => ["ai", "token-usage", interval.value.fromDate, interval.value.toDate]),
    queryFn: () => loadTokenUsageReport(interval.value.fromDate, interval.value.toDate),
    staleTime: 30_000,
  });
}
