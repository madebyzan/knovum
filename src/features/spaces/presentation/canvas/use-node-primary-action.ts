import { Focus, Info } from "@lucide/vue";
import { computed, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { nodeOpenIcons, nodeOpenLabels } from "./canvas-node-open-actions";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

export function useNodePrimaryAction(
  node: Readonly<Ref<SpaceCanvasNode | undefined>>,
  actions: ReturnType<typeof useCanvasSelectionActions>,
) {
  const { t } = useI18n();
  const primaryLabel = computed(() => {
    if (actions.state.value.section && actions.state.value.content.length)
      return t("spaces.scenes.focus.selection");
    const kind = node.value?.data.kind;
    return t(kind && actions.state.value.canOpen ? nodeOpenLabels[kind] : "spaces.inspector.title");
  });
  const primaryIcon = computed(() => {
    if (actions.state.value.section && actions.state.value.content.length) return Focus;
    const kind = node.value?.data.kind;
    return kind && actions.state.value.canOpen ? nodeOpenIcons[kind] : Info;
  });
  const showPrimary = computed(() => {
    if (actions.state.value.section && actions.state.value.content.length) return true;
    const kind = node.value?.data.kind;
    return Boolean(kind && actions.state.value.canOpen && nodeOpenLabels[kind] !== "spaces.inspector.title");
  });
  function runPrimaryAction() {
    if (actions.state.value.section && actions.state.value.content.length) actions.focus();
    else if (actions.state.value.canOpen) actions.open();
    else actions.inspect();
  }
  return { primaryLabel, primaryIcon, showPrimary, runPrimaryAction };
}
