import type { Ref } from "vue";

import { searchSpaceKnowledgeHybrid } from "../knowledge/queries/hybrid-search";

export interface SpaceSearchContext {
  activeSpaceId: Ref<string | undefined>;
}

export async function searchSpace(
  context: SpaceSearchContext,
  value: string,
  semantic = false,
  signal?: AbortSignal,
) {
  return value.trim() && context.activeSpaceId.value
    ? searchSpaceKnowledgeHybrid(context.activeSpaceId.value, value, {
        semantic,
        ...(signal ? { signal } : {}),
      })
    : { matches: [], semantic: "empty" as const };
}
