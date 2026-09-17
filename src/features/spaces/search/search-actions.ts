import type { Ref } from "vue";

import { searchSpaceKnowledgeHybrid } from "../knowledge/queries/hybrid-search";

export interface SpaceSearchContext {
  activeSpaceId: Ref<string | undefined>;
}

export async function searchSpace(context: SpaceSearchContext, value: string) {
  return value.trim() && context.activeSpaceId.value
    ? searchSpaceKnowledgeHybrid(context.activeSpaceId.value, value)
    : { matches: [], semantic: "empty" as const };
}
