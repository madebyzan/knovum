import { today, getLocalTimeZone } from "@internationalized/date";
import { useQuery } from "@tanstack/vue-query";
import { computed, type Ref } from "vue";

import { loadTokenUsageReport } from "../../../foundation/ai/token-usage";

export type UsageRange = "year" | "quarter";

export function useTokenUsage(range: Ref<UsageRange>) {
  const interval = computed(() => {
    const to = today(getLocalTimeZone());
    const days = range.value === "year" ? 364 : 89;
    return { fromDate: to.subtract({ days }).toString(), toDate: to.toString() };
  });
  const query = useQuery({
    queryKey: computed(() => ["ai", "token-usage", interval.value.fromDate, interval.value.toDate]),
    queryFn: () => loadTokenUsageReport(interval.value.fromDate, interval.value.toDate),
    staleTime: 30_000,
  });

  return { ...query, interval };
}
