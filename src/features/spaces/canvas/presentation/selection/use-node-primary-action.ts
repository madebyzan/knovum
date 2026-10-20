import { Sparkles } from "@lucide/vue";
import { computed, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import { OPEN_ACTION_BY_NODE_KIND } from "../canvas-node-open-actions";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

/**
 * The action bar leads with what the node is for, so a selected node is never represented by an
 * overflow menu alone. A kind with an open surface opens it; a node the conversation can read is
 * asked about instead. A node with neither — a credential, whose own card carries its editor —
 * keeps the bar to its access actions rather than showing a decorative one.
 */
export function useNodePrimaryAction(
  node: Readonly<Ref<SpaceCanvasNode | undefined>>,
  actions: ReturnType<typeof useCanvasSelectionActions>,
) {
  const { t } = useI18n();
  const openAction = computed(() => {
    const kind = node.value?.data.kind;
    return kind && actions.state.value.canOpen ? OPEN_ACTION_BY_NODE_KIND[kind] : undefined;
  });
  const asksInstead = computed(() => !openAction.value && actions.state.value.canAsk);
  const primary = computed(() => {
    if (openAction.value)
      return { label: t(openAction.value.label), icon: openAction.value.icon, run: actions.open };
    return asksInstead.value
      ? { label: t("spaces.conversation.askSelection"), icon: Sparkles, run: actions.ask }
      : undefined;
  });
  return { primary, asksInstead };
}
