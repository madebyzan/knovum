<script setup lang="ts">
import { ToolbarRoot } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceCanvasSelectionPrimary from "./SpaceCanvasSelectionPrimary.vue";
import { useCanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{ derivations: CreationIntentDecisions }>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const actions = useCanvasSelectionActions();
const { t } = useI18n();
</script>
<template>
  <ToolbarRoot
    v-if="actions.visible.value"
    orientation="horizontal"
    :loop="false"
    :aria-label="t('spaces.navigation.selection')"
    class="flex h-7 cursor-default items-center gap-0.5 rounded-control border border-border bg-overlay select-none"
  >
    <UiBadge v-if="actions.state.value.roots.length > 1" size="sm" class="mx-1">{{
      actions.state.value.roots.length
    }}</UiBadge>
    <SpaceCanvasSelectionPrimary :actions="actions" />
    <SpaceCanvasSelectionMenu
      :actions="actions"
      :derivations="derivations"
      @invoke-derivation="emit('invokeDerivation', $event)"
    />
  </ToolbarRoot>
</template>
