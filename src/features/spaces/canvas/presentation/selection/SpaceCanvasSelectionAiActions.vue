<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiKnovumAiIcon from "../../../../../shared/ui/UiKnovumAiIcon.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import { hasVisibleCreationActions } from "../../../knowledge-derivations/presentation/visible-derivation-actions";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
import SpaceCanvasCreationMenu from "./SpaceCanvasCreationMenu.vue";

/**
 * Unified leading AI & Creation cluster shared by every content selection bar (single node, image,
 * and multi-selection): Ask Knovum always leads first, Create second, followed by an optional
 * divider before node-specific or spatial actions.
 */
const props = defineProps<{
  canAsk: boolean;
  derivations: CreationIntentDecisions;
  compact?: boolean | undefined;
  menuItem?: boolean | undefined;
  separator?: boolean | undefined;
}>();
const emit = defineEmits<{ ask: []; invokeDerivation: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const hasAny = computed(() => props.canAsk || hasVisibleCreationActions(props.derivations));
</script>

<template>
  <SpaceCanvasActionTrigger
    v-if="canAsk"
    :label="t('spaces.conversation.askSelection')"
    :icon="UiKnovumAiIcon"
    :compact="compact"
    :menu-item="menuItem"
    class="font-medium text-fg-primary"
    @invoke="emit('ask')"
  />
  <SpaceCanvasCreationMenu
    :derivations="derivations"
    :compact="compact"
    :submenu="menuItem"
    @invoke="emit('invokeDerivation', $event)"
  />
  <UiSeparator v-if="separator && hasAny && !menuItem" orientation="vertical" class="mx-0.5 h-4" />
</template>
