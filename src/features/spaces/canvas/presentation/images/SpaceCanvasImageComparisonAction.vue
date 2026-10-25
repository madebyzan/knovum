<script setup lang="ts">
import { SquareSplitHorizontal } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { computed, shallowRef, toRef } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import { spaceAssetSource } from "../../../assets/node-preview-source";
import { useDerivationComparison } from "../../../knowledge-derivations/comparison/use-derivation-comparison";
import SpaceImageComparisonViewer from "../../../knowledge-derivations/presentation/SpaceImageComparisonViewer.vue";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";

/**
 * Reading what a derivation did makes nothing, so it sits with the node's other ways of being read
 * (details, export), not among the actions that make something. Offered only on a derived picture
 * that lays over its input; on any other image it is absent.
 */
const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const { t } = useI18n();
const open = shallowRef(false);
const { available, input, error } = useDerivationComparison(toRef(props, "node"), open);
const before = computed(() => input.value && spaceAssetSource(input.value.path, input.value.revision));
const after = computed(() => spaceAssetSource(props.node.data.assetPath, props.node.data.sourceRevision));
</script>

<template>
  <template v-if="available">
    <ToolbarButton as-child>
      <UiIconButton :label="t('spaces.media.imageComparison.title')" tooltip @click="open = true">
        <SquareSplitHorizontal class="size-4.5" :stroke-width="1.6" />
      </UiIconButton>
    </ToolbarButton>
    <SpaceImageComparisonViewer v-model:open="open" :before="before" :after="after" :error="error" />
  </template>
</template>
