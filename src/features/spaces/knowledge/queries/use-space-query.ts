import { useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";

function spaceIdValue(spaceId: MaybeRefOrGetter<string | null | undefined>) {
  return toValue(spaceId) ?? "none";
}

export function useSpaceQuery<T>(
  spaceId: MaybeRefOrGetter<string | null | undefined>,
  keyFactory: (spaceId: string) => readonly unknown[],
  fetcher: (spaceId: string) => Promise<T>,
  extraEnabled?: () => boolean,
) {
  return useQuery<T>(
    computed(() => {
      const id = spaceIdValue(spaceId);
      return {
        queryKey: keyFactory(id),
        queryFn: () => fetcher(id),
        enabled: Boolean(toValue(spaceId)) && (extraEnabled ? extraEnabled() : true),
      };
    }),
  );
}
