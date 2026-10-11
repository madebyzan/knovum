import { Info } from "@lucide/vue";
import { computed, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import { NODE_OPEN_ICONS, NODE_OPEN_LABELS } from "../canvas-node-open-actions";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

export function useNodePrimaryAction(
  node: Readonly<Ref<SpaceCanvasNode | undefined>>,
  actions: ReturnType<typeof useCanvasSelectionActions>,
) {
  const { t } = useI18n();
  const primaryLabel = computed(() => {
    const kind = node.value?.data.kind;
    return t(kind && actions.state.value.canOpen ? NODE_OPEN_LABELS[kind] : "spaces.inspector.title");
  });
  const primaryIcon = computed(() => {
    const kind = node.value?.data.kind;
    return kind && actions.state.value.canOpen ? NODE_OPEN_ICONS[kind] : Info;
  });
  const showPrimary = computed(() => {
    const kind = node.value?.data.kind;
    return Boolean(
      kind && actions.state.value.canOpen && NODE_OPEN_LABELS[kind] !== "spaces.inspector.title",
    );
  });
  function runPrimaryAction() {
    if (actions.state.value.canOpen) actions.open();
    else actions.inspect();
  }
  return { primaryLabel, primaryIcon, showPrimary, runPrimaryAction };
}
