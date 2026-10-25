<script setup lang="ts">
import { SquareSplitHorizontal } from "@lucide/vue";
import { computed, shallowRef, toRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import UiDialog from "../../../../../shared/ui/UiDialog.vue";
import { spaceAssetSource } from "../../../assets/node-preview-source";
import { useDerivationComparison } from "../../../knowledge-derivations/comparison/use-derivation-comparison";
import SpaceImageComparisonViewer from "../../../knowledge-derivations/presentation/SpaceImageComparisonViewer.vue";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";

/** Offered only on a derived picture that lays over its input; on any other image it is absent. */
const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload>; compact: boolean }>();
const { t } = useI18n();
const open = shallowRef(false);
const { available, input, error } = useDerivationComparison(toRef(props, "node"), open);
const before = computed(() => input.value && spaceAssetSource(input.value.path, input.value.revision));
const after = computed(() => spaceAssetSource(props.node.data.assetPath, props.node.data.sourceRevision));
</script>

<template>
  <template v-if="available">
    <SpaceCanvasActionTrigger
      :label="t('spaces.media.imageComparison.title')"
      :icon="SquareSplitHorizontal"
      :compact="compact"
      @invoke="open = true"
    />
    <UiDialog
      v-model:open="open"
      :title="t('spaces.media.imageComparison.title')"
      :description="t('spaces.media.imageComparison.hint')"
      size="wide"
      class="h-[calc(100vh-5rem)] max-w-[calc(100vw-5rem)]"
      body-class="relative overflow-hidden p-0"
    >
      <SpaceImageComparisonViewer v-if="before" :before="before" :after="after" />
      <p v-else class="grid size-full place-items-center text-caption text-fg-secondary">
        {{ error ? getErrorMessage(error) : t("spaces.media.imageComparison.preparing") }}
      </p>
    </UiDialog>
  </template>
</template>
