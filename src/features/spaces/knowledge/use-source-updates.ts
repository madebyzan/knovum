import { watchDebounced } from "@vueuse/core";
import { computed } from "vue";

import { queryClient } from "../../../foundation/query/query-client";
import { useSpaceStore } from "../session/space-store";
import { sourceUpdateCounts } from "./model/health";
import { SPACE_KNOWLEDGE_QUERY_KEYS } from "./queries/query-options";
import { useSpaceKnowledgeHealthQuery } from "./queries/use-space-knowledge-health-query";

/** Edits land in bursts; the check runs once they settle. */
const SETTLE_MS = 800;

/**
 * How many sources of each item on the canvas have changed since the item was kept. The person
 * decides what to do about it; nothing here changes the item.
 */
export function useSourceUpdateCounts() {
  const store = useSpaceStore();
  const health = useSpaceKnowledgeHealthQuery(() => store.activeSpaceId);
  const sources = computed(() => new Map(store.nodes.map((node) => [node.id, node.data])));
  // A source changing elsewhere — an edit, a re-import, a transcript — moves its revision, which is
  // what makes items kept from it stale; the check runs again then.
  const revisions = computed(() =>
    store.nodes
      .map(({ id, data }) => `${id}:${data.sourceRevision ?? ""}:${data.knowledgeRevision ?? ""}`)
      .join(" "),
  );
  watchDebounced(
    revisions,
    () => {
      if (!store.activeSpaceId) return;
      void queryClient.invalidateQueries({
        queryKey: SPACE_KNOWLEDGE_QUERY_KEYS.health(store.activeSpaceId),
      });
    },
    { debounce: SETTLE_MS },
  );
  return computed(() => sourceUpdateCounts(health.data.value ?? [], sources.value));
}
