<script setup lang="ts">
import { ChevronRight, FileOutput } from "@lucide/vue";
import {
  DropdownMenuPortal,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  ToolbarButton,
} from "reka-ui";
import { useI18n } from "vue-i18n";

import { SUBMENU_CONTENT_CLASSES } from "../../../../shared/ui/ui-classes";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import { CANVAS_ACTION_BUTTON_CLASSES } from "../canvas/canvas-action-classes";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "../canvas/node-action-label";
import SpaceDocumentConversionOptions from "./SpaceDocumentConversionOptions.vue";

defineProps<{ decisions: CreationIntentDecisions; isCompact?: boolean; isSubmenu?: boolean }>();
const emit = defineEmits<{ invoke: [actionId: CreationIntentId] }>();
const { t } = useI18n();
</script>

<template>
  <DropdownMenuSub v-if="isSubmenu">
    <DropdownMenuSubTrigger
      data-testid="document-conversion-trigger"
      class="flex h-interaction cursor-default items-center gap-2 rounded-md px-2 text-label text-fg-primary outline-none data-[highlighted]:bg-surface-control-hover"
    >
      <FileOutput class="size-3.5" />
      <span class="min-w-0 flex-1">{{ t("spaces.creation.document.convert.label") }}</span>
      <ChevronRight class="size-3 text-fg-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="SUBMENU_CONTENT_CLASSES">
        <SpaceDocumentConversionOptions :decisions="decisions" @invoke="emit('invoke', $event)" />
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
  <UiDropdownMenu v-else side="top" align="center" :side-offset="6" :modal="false">
    <template #trigger>
      <ToolbarButton as-child>
        <button
          type="button"
          data-testid="document-conversion-trigger"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :aria-label="t('spaces.creation.document.convert.label')"
          :title="isCompact ? t('spaces.creation.document.convert.label') : undefined"
        >
          <FileOutput class="size-4.5" :stroke-width="1.6" />
          <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(Boolean(isCompact))]">
            {{ t("spaces.creation.document.convert.label") }}
          </span>
        </button>
      </ToolbarButton>
    </template>
    <SpaceDocumentConversionOptions :decisions="decisions" @invoke="emit('invoke', $event)" />
  </UiDropdownMenu>
</template>
