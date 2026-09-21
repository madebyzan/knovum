import { ask } from "@tauri-apps/plugin-dialog";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useSpaceStore } from "../session/space-store";
import type { SpaceDeletedStructureRoot } from "./deleted-structure";
import { permanentlyDeleteStructure, restoreDeletedStructure } from "./deleted-structure-actions";

export function useRecentlyDeletedActions() {
  const store = useSpaceStore();
  const { t } = useI18n();
  const busy = shallowRef(false);

  async function restore(deletedRoot: SpaceDeletedStructureRoot, wholeBatch = false) {
    if (busy.value) return;
    busy.value = true;
    try {
      await restoreDeletedStructure(deletedRoot, wholeBatch);
      if (store.activeSpaceId === deletedRoot.spaceId) {
        await store.refreshActiveSpace();
        store.recordCommittedSpaceEdit();
      }
    } catch (error) {
      toast.error(t("spaces.navigation.restoreStructureFailed"), { description: String(error) });
    } finally {
      busy.value = false;
    }
  }

  async function remove(deletedRoots: readonly SpaceDeletedStructureRoot[], scope?: string) {
    if (busy.value || !deletedRoots.length) return;
    const selected = [...deletedRoots];
    busy.value = true;
    try {
      const confirmed = await ask(
        scope === undefined
          ? t("spaces.navigation.permanentlyDeleteStructureDescription", { title: selected[0]!.title })
          : t("spaces.navigation.deleteAllDescription", { count: selected.length, scope }),
        {
          title: t(
            scope === undefined ? "spaces.navigation.deletePermanently" : "spaces.navigation.deleteAllTitle",
          ),
          kind: "warning",
          okLabel: t("spaces.navigation.deletePermanently"),
          cancelLabel: t("common.cancel"),
        },
      );
      if (!confirmed) return;
      await permanentlyDeleteStructure(selected, scope !== undefined);
      toast.success(t("spaces.navigation.structureNodesPermanentlyDeleted", { count: selected.length }));
    } catch (error) {
      toast.error(t("spaces.navigation.permanentlyDeleteStructureFailed"), { description: String(error) });
    } finally {
      busy.value = false;
    }
  }

  return { busy, restore, remove };
}
