import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { errorMessage } from "../../../foundation/errors/error-format";
import { confirmAction } from "../../../shared/ui/confirm-action";
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
      toast.error(t("spaces.trash.structure.restoreFailed"), { description: errorMessage(error) });
    } finally {
      busy.value = false;
    }
  }

  async function remove(deletedRoots: readonly SpaceDeletedStructureRoot[], scope?: string) {
    if (busy.value || !deletedRoots.length) return;
    const selected = [...deletedRoots];
    busy.value = true;
    try {
      const confirmed = await confirmAction({
        title: t(scope === undefined ? "spaces.trash.deletePermanently" : "spaces.trash.deleteAll.title"),
        description:
          scope === undefined
            ? t("spaces.trash.structure.deleteDescription", { title: selected[0]!.title })
            : t("spaces.trash.deleteAll.description", { count: selected.length, scope }),
        confirmLabel: t("spaces.trash.deletePermanently"),
        danger: true,
      });
      if (!confirmed) return;
      await permanentlyDeleteStructure(selected, scope !== undefined);
      toast.success(t("spaces.trash.structure.deletedNotice", { count: selected.length }));
    } catch (error) {
      toast.error(t("spaces.trash.structure.deleteFailed"), { description: errorMessage(error) });
    } finally {
      busy.value = false;
    }
  }

  return { busy, restore, remove };
}
