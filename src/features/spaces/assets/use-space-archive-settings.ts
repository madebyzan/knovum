import pLimit from "p-limit";
import { storeToRefs } from "pinia";
import { computed } from "vue";

import type { ArchiveSettingsContract } from "../../../shared/contracts/archive-settings";
import { useSpaceStore } from "../session/space-store";

const permanentDeletions = pLimit(2);

export function useSpaceArchiveSettings(): ArchiveSettingsContract {
  const store = useSpaceStore();
  const { spaces } = storeToRefs(store);
  const archivedSpaces = computed(() =>
    spaces.value.filter((space) => space.archivedAt !== null).map(({ id, title }) => ({ id, title })),
  );
  return {
    spaces: archivedSpaces,
    initialize: (inboxTitle) => store.initialize(inboxTitle),
    restore: (spaceId) => store.unarchiveSpace(spaceId),
    permanentlyDelete: (spaceId) => store.permanentlyDeleteSpace(spaceId),
    permanentlyDeleteAll: async () => {
      const ids = archivedSpaces.value.map(({ id }) => id);
      await Promise.all(ids.map((id) => permanentDeletions(() => store.permanentlyDeleteSpace(id))));
    },
  };
}
