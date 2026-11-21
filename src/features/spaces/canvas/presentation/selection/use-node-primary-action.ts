import { computed, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import { resolveNodeOpenAction } from "../canvas-node-open-actions";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

/** Resolves the node kind's primary open/edit/preview action for its floating action bar. */
export function useNodePrimaryAction(
  node: Readonly<Ref<SpaceCanvasNode | undefined>>,
  actions: ReturnType<typeof useCanvasSelectionActions>,
) {
  const { t } = useI18n();
  const primary = computed(() => {
    const kind = node.value?.data.kind;
    const openAction = kind && actions.state.value.canOpen ? resolveNodeOpenAction(kind) : undefined;
    return openAction ? { label: t(openAction.label), icon: openAction.icon, run: actions.open } : undefined;
  });
  return { primary };
}
