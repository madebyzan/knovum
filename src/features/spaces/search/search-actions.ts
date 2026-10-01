import type { Ref } from "vue";

import { searchSpaceKnowledgeHybrid } from "../knowledge/queries/hybrid-search";

export interface SpaceSearchContext {
  activeSpaceId: Ref<string | undefined>;
}

interface SpaceSearchInput {
  context: SpaceSearchContext;
  value: string;
  semantic?: boolean;
  signal?: AbortSignal | undefined;
}

export async function searchSpace({ context, value, semantic = false, signal }: SpaceSearchInput) {
  return value.trim() && context.activeSpaceId.value
    ? searchSpaceKnowledgeHybrid(context.activeSpaceId.value, value, {
        semantic,
        ...(signal ? { signal } : {}),
      })
    : { matches: [], semantic: "empty" as const };
}
