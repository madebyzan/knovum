<script setup lang="ts">
import { ChevronRight } from "@lucide/vue";
import {
  DropdownMenuPortal,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  ToolbarButton,
} from "reka-ui";
import { computed, shallowRef, type Component } from "vue";
import { useI18n } from "vue-i18n";

import { useLocalModelsQuery } from "../../../../../foundation/runtime/use-local-models-query";
import { canRunAction, shouldShowAction } from "../../../../../shared/contracts/action-availability";
import type { LocalModelCapability } from "../../../../../shared/contracts/generated/LocalModelCapability";
import { POPOVER_SURFACE_CLASSES, POPUP_ANIMATE_CLASSES } from "../../../../../shared/ui/ui-classes";
import UiBadge from "../../../../../shared/ui/UiBadge.vue";
import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiPopover from "../../../../../shared/ui/UiPopover.vue";
import UiTooltip from "../../../../../shared/ui/UiTooltip.vue";
import type { ImageExecutionTarget } from "../../../contracts/generated/ImageExecutionTarget";
import {
  preferredVerifiedImageTarget,
  verifiedImageTargetOptions,
} from "../../../knowledge-derivations/image-model-targets";
import { useImageOperationCapabilities } from "../../../knowledge-derivations/use-image-operation-capabilities";
import { CANVAS_ACTION_BUTTON_CLASSES } from "../canvas-action-classes";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "../selection/node-action-label";
import type { ImageActionAvailability } from "./image-action-availability";
import SpaceCanvasImageActionTrigger from "./SpaceCanvasImageActionTrigger.vue";

const props = defineProps<{
  capability: LocalModelCapability;
  availability: ImageActionAvailability;
  reason: string;
  icon: Component;
  labelKey: string;
  compact?: boolean;
  menuItem?: boolean;
}>();
const emit = defineEmits<{ run: [execution?: ImageExecutionTarget] }>();
const { t } = useI18n();
const open = shallowRef(false);
const visible = computed(() => shouldShowAction(props.availability));
const capabilities = useImageOperationCapabilities(visible);
const localModels = useLocalModelsQuery(visible);
const options = computed(() =>
  verifiedImageTargetOptions(
    capabilities.data.value?.targets ?? [],
    localModels.data.value ?? [],
    props.capability,
  ),
);
const multiple = computed(() => canRunAction(props.availability) && options.value.length > 1);
const defaultExecution = computed(() => preferredVerifiedImageTarget(options.value)?.execution);

function pick(execution: ImageExecutionTarget) {
  open.value = false;
  emit("run", execution);
}
</script>

<template>
  <template v-if="visible">
    <DropdownMenuSub v-if="menuItem && multiple">
      <DropdownMenuSubTrigger
        class="flex h-(--knovum-control-height-lg) w-full cursor-default items-center gap-2 rounded-md px-2 text-left text-label text-fg-primary outline-none data-highlighted:bg-surface-control-hover"
      >
        <component :is="icon" class="size-3.5" />
        <span class="min-w-0 flex-1">{{ t(labelKey) }}</span>
        <ChevronRight class="size-3 text-fg-tertiary" />
      </DropdownMenuSubTrigger>
      <DropdownMenuPortal>
        <DropdownMenuSubContent
          :side-offset="4"
          :align-offset="-4"
          :class="[
            POPOVER_SURFACE_CLASSES,
            POPUP_ANIMATE_CLASSES,
            'z-50 w-max max-w-80 min-w-52 p-1 shadow-popover outline-none',
          ]"
        >
          <UiDropdownMenuItem
            v-for="(option, index) in options"
            :key="index"
            class="justify-between gap-3"
            @select="emit('run', option.execution)"
          >
            <span class="truncate">{{ option.label }}</span>
            <UiBadge v-if="option.selected" size="sm" class="shrink-0">{{
              t("ai.modelChoice.default")
            }}</UiBadge>
          </UiDropdownMenuItem>
        </DropdownMenuSubContent>
      </DropdownMenuPortal>
    </DropdownMenuSub>
    <UiPopover
      v-else-if="!menuItem && multiple"
      v-model:open="open"
      side="bottom"
      align="start"
      class="w-max max-w-80 min-w-52 p-1"
    >
      <template #trigger>
        <ToolbarButton as-child>
          <button
            type="button"
            :class="CANVAS_ACTION_BUTTON_CLASSES"
            :aria-label="t(labelKey)"
            :title="compact ? t(labelKey) : undefined"
          >
            <component :is="icon" class="size-4.5" :stroke-width="1.6" />
            <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(Boolean(compact))]">{{
              t(labelKey)
            }}</span>
          </button>
        </ToolbarButton>
      </template>
      <UiTooltip v-for="(option, index) in options" :key="index" :text="option.label" disabled>
        <button
          type="button"
          class="flex h-(--knovum-control-height-lg) w-full items-center justify-between gap-3 rounded-md px-2 text-left text-label outline-none hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus"
          @click="pick(option.execution)"
        >
          <span class="truncate font-medium text-fg-primary">{{ option.label }}</span>
          <UiBadge v-if="option.selected" size="sm" class="shrink-0">{{
            t("ai.modelChoice.default")
          }}</UiBadge>
        </button>
      </UiTooltip>
    </UiPopover>
    <SpaceCanvasImageActionTrigger
      v-else
      :availability="availability"
      :reason="reason"
      :icon="icon"
      :label-key="labelKey"
      :compact="compact"
      :menu-item="menuItem"
      @begin="emit('run', defaultExecution)"
    />
  </template>
</template>
