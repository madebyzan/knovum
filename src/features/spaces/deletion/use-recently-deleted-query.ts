import { queryOptions, useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";

import { listRecentlyDeletedStructure } from "./deleted-structure-actions";
import { spaceDeletionQueryKeys } from "./query-keys";

export function useRecentlyDeletedQuery(
  spaceId: MaybeRefOrGetter<string | null>,
  enabled: MaybeRefOrGetter<boolean>,
) {
  return useQuery(
    computed(() =>
      queryOptions({
        queryKey: spaceDeletionQueryKeys.deleted(toValue(spaceId)),
        queryFn: () => listRecentlyDeletedStructure(toValue(spaceId)),
        enabled: toValue(enabled),
      }),
    ),
  );
}
