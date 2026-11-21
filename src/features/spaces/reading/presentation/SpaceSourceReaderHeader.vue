<script setup lang="ts">
import { ArrowUpRight, LocateFixed, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { openSpaceAsset } from "../../assets/asset-operations";
import { nodeIcon } from "../../canvas/presentation/node-icon";
import { selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";

const props = defineProps<{ node: SpaceCanvasNode }>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();

async function openExternally() {
  const data = props.node.data;
  if (!isSpaceManagedAssetPayload(data)) return;
  try {
    await openSpaceAsset(data.assetPath);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}
</script>

<template>
  <header
    data-reader-ignore-selection
    class="flex h-10 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider bg-surface-panel px-3 select-none"
  >
    <div class="flex min-w-0 items-center gap-2">
      <component
        :is="nodeIcon(node.data.kind)"
        class="size-4 shrink-0 text-fg-secondary"
        :stroke-width="1.6"
      />
      <h2 class="truncate text-label font-semibold text-fg-primary" :title="node.data.title">
        {{ node.data.title }}
      </h2>
    </div>
    <div class="flex shrink-0 items-center gap-0.5">
      <UiIconButton
        :label="t('spaces.canvas.revealInCanvas')"
        size="sm"
        tooltip
        @click="selectAndRevealSpaceNode(node.id)"
      >
        <LocateFixed class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        v-if="isSpaceManagedAssetPayload(node.data)"
        :label="t('spaces.importExport.openExternally')"
        size="sm"
        tooltip
        @click="openExternally"
      >
        <ArrowUpRight class="size-3.5" />
      </UiIconButton>
      <UiIconButton :label="t('common.close')" size="sm" tooltip @click="emit('close')">
        <X class="size-3.5" />
      </UiIconButton>
    </div>
  </header>
</template>
