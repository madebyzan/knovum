<script setup lang="ts">
import { ChevronRight } from "@lucide/vue";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { SUBMENU_CONTENT_CLASSES } from "../../../../../shared/ui/ui-classes";
import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceDerivationMenuItems from "../../../knowledge-derivations/presentation/SpaceDerivationMenuItems.vue";
import { hasVisibleCreationActions } from "../../../knowledge-derivations/presentation/visible-derivation-actions";
import SpaceCanvasMenuTrigger from "../SpaceCanvasMenuTrigger.vue";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "./node-action-label";
import SpaceCanvasCreateIcon from "./SpaceCanvasCreateIcon.vue";

/**
 * Primary creation menu for the active canvas selection (document, image, video). Renders as a
 * first-class action-bar dropdown by default and degrades into an overflow submenu when minimal.
 */
const props = defineProps<{
  derivations: CreationIntentDecisions;
  compact?: boolean | undefined;
  submenu?: boolean | undefined;
}>();
const emit = defineEmits<{ invoke: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const hasActions = computed(() => hasVisibleCreationActions(props.derivations));
</script>

<template>
  <DropdownMenuSub v-if="hasActions && submenu">
    <DropdownMenuSubTrigger
      class="flex h-interaction cursor-default items-center gap-2 rounded-md px-2 text-label text-fg-primary outline-none data-[highlighted]:bg-surface-control-hover"
    >
      <SpaceCanvasCreateIcon class="size-3.5 shrink-0" />
      <span class="min-w-0 flex-1">{{ t("spaces.creation.shared.createAction") }}</span>
      <ChevronRight class="size-3 text-fg-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="SUBMENU_CONTENT_CLASSES">
        <SpaceDerivationMenuItems :derivations="derivations" @invoke="emit('invoke', $event)" />
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
  <UiDropdownMenu v-else-if="hasActions" side="top" align="center" :side-offset="6" class="min-w-44">
    <template #trigger>
      <SpaceCanvasMenuTrigger :label="t('spaces.creation.shared.createFromSelection')">
        <SpaceCanvasCreateIcon class="size-4.5 shrink-0" />
        <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(Boolean(compact))]">{{
          t("spaces.creation.shared.createAction")
        }}</span>
      </SpaceCanvasMenuTrigger>
    </template>
    <SpaceDerivationMenuItems :derivations="derivations" @invoke="emit('invoke', $event)" />
  </UiDropdownMenu>
</template>
