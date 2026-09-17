import { storeToRefs } from "pinia";
import { computed } from "vue";

import type { ArchiveSettingsContract } from "../../../shared/contracts/archive-settings";
import { useSpaceStore } from "../session/space-store";

export function createSpaceArchiveSettings(): ArchiveSettingsContract {
  const store = useSpaceStore();
  const { spaces } = storeToRefs(store);
  const archivedSpaces = computed(() =>
    spaces.value.filter((space) => space.archivedAt !== null).map(({ id, title }) => ({ id, title })),
  );
  return {
    spaces: archivedSpaces,
    initialize: (inboxTitle) => store.initialize(inboxTitle),
    restore: (id) => store.unarchiveSpace(id),
    permanentlyDelete: (id) => store.permanentlyDeleteSpace(id),
    permanentlyDeleteAll: async () => {
      const ids = archivedSpaces.value.map(({ id }) => id);
      await Promise.all(ids.map((id) => store.permanentlyDeleteSpace(id)));
    },
  };
}
