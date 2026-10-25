<script setup lang="ts">
import { SquareSplitHorizontal } from "@lucide/vue";
import { shallowRef, toRef } from "vue";
import { useI18n } from "vue-i18n";

import { useDerivationComparison } from "../../../knowledge-derivations/comparison/use-derivation-comparison";
import SpaceDerivationComparisonDialog from "../../../knowledge-derivations/presentation/SpaceDerivationComparisonDialog.vue";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";

/** Offered only on a derived picture that lays over its input; on any other image it is absent. */
const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload>; compact: boolean }>();
const { t } = useI18n();
const open = shallowRef(false);
const { available, input, error } = useDerivationComparison(toRef(props, "node"), open);
</script>

<template>
  <template v-if="available">
    <SpaceCanvasActionTrigger
      :label="t('spaces.media.imageComparison.title')"
      :icon="SquareSplitHorizontal"
      :compact="compact"
      @invoke="open = true"
    />
    <SpaceDerivationComparisonDialog v-model:open="open" :output="node.data" :input="input" :error="error" />
  </template>
</template>
