import { ask } from "@tauri-apps/plugin-dialog";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useSpaceStore } from "../session/space-store";
import type { SpaceDeletedNode } from "./deleted-node";
import { permanentlyDeleteItems, restoreDeletedNode } from "./deleted-node-actions";

export function useRecentlyDeletedActions() {
  const store = useSpaceStore();
  const { t } = useI18n();
  const busy = shallowRef(false);

  async function restore(item: SpaceDeletedNode) {
    if (busy.value) return;
    busy.value = true;
    try {
      await restoreDeletedNode(item.spaceId, item.id);
      if (store.activeSpaceId === item.spaceId) await store.refreshActiveSpace();
    } catch (error) {
      toast.error(t("spaces.navigation.restoreItemFailed"), { description: String(error) });
    } finally {
      busy.value = false;
    }
  }

  async function remove(items: readonly SpaceDeletedNode[], scope?: string) {
    if (busy.value || !items.length) return;
    const selected = [...items];
    busy.value = true;
    try {
      const confirmed = await ask(
        scope === undefined
          ? t("spaces.navigation.permanentlyDeleteItemDescription", { title: selected[0]!.title })
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
      await permanentlyDeleteItems(selected);
      toast.success(t("spaces.navigation.itemsPermanentlyDeleted", { count: selected.length }));
    } catch (error) {
      toast.error(t("spaces.navigation.deleteItemFailed"), { description: String(error) });
    } finally {
      busy.value = false;
    }
  }

  return { busy, restore, remove };
}
