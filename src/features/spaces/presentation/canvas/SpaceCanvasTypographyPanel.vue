<script setup lang="ts">
import {
  AlignCenter,
  AlignCenterVertical,
  AlignEndVertical,
  AlignLeft,
  AlignRight,
  AlignStartVertical,
  Bold,
  Check,
  Italic,
  Underline,
} from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import { canvasBaseFontSize, effectiveCanvasFontSize } from "../../canvas/document/canvas-typography";
import { DEFAULT_CANVAS_TEXT_FONT_SIZE } from "../../canvas/node-size";
import type { CanvasNodeAppearance } from "../../canvas/schema/document";
import { CANVAS_FONT_SIZES } from "./canvas-typography-options";
import { nodeActionLabelClass, nodeActionLabelState } from "./node-action-label";

const props = defineProps<{ kind: "note" | "text"; appearance?: CanvasNodeAppearance; compact: boolean }>();
const emit = defineEmits<{ update: [patch: Partial<CanvasNodeAppearance>] }>();
const { t } = useI18n();
const typographyScale = computed(() => props.appearance?.typographyScale ?? 1);
const fontSize = computed(() =>
  effectiveCanvasFontSize(props.appearance?.fontSize ?? DEFAULT_CANVAS_TEXT_FONT_SIZE, typographyScale.value),
);
const textAlign = computed(() => props.appearance?.textAlign ?? (props.kind === "text" ? "center" : "left"));
const verticalAlign = computed(
  () => props.appearance?.verticalAlign ?? (props.kind === "text" ? "middle" : "top"),
);
const horizontalOptions = [
  { value: "left", icon: AlignLeft },
  { value: "center", icon: AlignCenter },
  { value: "right", icon: AlignRight },
] as const;
const verticalOptions = [
  { value: "top", icon: AlignStartVertical },
  { value: "middle", icon: AlignCenterVertical },
  { value: "bottom", icon: AlignEndVertical },
] as const;
</script>

<template>
  <UiPopover
    side="top"
    align="center"
    :side-offset="6"
    :trigger-tooltip="compact ? t('spaces.canvas.typography') : undefined"
    class="p-0.5"
  >
    <template #trigger>
      <button
        type="button"
        class="inline-flex h-(--knovum-control-height-md) min-w-8 items-center justify-center gap-1.5 rounded-md px-2.5 text-label-md font-medium text-fg-secondary transition-colors duration-100 outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus"
        :aria-label="t('spaces.canvas.typography')"
      >
        Aa
        <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{
          t("spaces.canvas.typography")
        }}</span>
      </button>
    </template>

    <div class="flex items-center gap-0.5">
      <div class="flex items-center gap-0.5">
        <UiIconButton
          :label="t('spaces.richtext.bold')"
          size="sm"
          :active="appearance?.fontWeight === 'bold'"
          @click="emit('update', { fontWeight: appearance?.fontWeight === 'bold' ? 'regular' : 'bold' })"
        >
          <Bold class="size-3.5" :stroke-width="1.7" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.richtext.italic')"
          size="sm"
          :active="appearance?.fontStyle === 'italic'"
          @click="emit('update', { fontStyle: appearance?.fontStyle === 'italic' ? 'normal' : 'italic' })"
        >
          <Italic class="size-3.5" :stroke-width="1.7" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.richtext.underline')"
          size="sm"
          :active="appearance?.textDecoration === 'underline'"
          @click="
            emit('update', {
              textDecoration: appearance?.textDecoration === 'underline' ? 'none' : 'underline',
            })
          "
        >
          <Underline class="size-3.5" :stroke-width="1.7" />
        </UiIconButton>
      </div>
      <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
      <UiDropdownMenu side="top" align="center" :side-offset="5" :modal="false" class="min-w-20">
        <template #trigger>
          <button
            type="button"
            class="h-(--knovum-control-height-sm) min-w-10 rounded-md px-1.5 text-caption-md font-medium text-fg-secondary tabular-nums transition-colors duration-100 outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus"
            :aria-label="t('spaces.canvas.fontSize')"
          >
            {{ fontSize }}
          </button>
        </template>
        <UiDropdownMenuItem
          v-for="size in CANVAS_FONT_SIZES"
          :key="size"
          @select="emit('update', { fontSize: canvasBaseFontSize(size, typographyScale) })"
        >
          <Check class="size-3" :class="fontSize === size ? 'opacity-100' : 'opacity-0'" />
          <span class="tabular-nums">{{ size }} pt</span>
        </UiDropdownMenuItem>
      </UiDropdownMenu>
      <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
      <UiIconButton
        v-for="option in horizontalOptions"
        :key="option.value"
        :label="t(`spaces.canvas.textAlignments.${option.value}`)"
        size="sm"
        :active="textAlign === option.value"
        @click="emit('update', { textAlign: option.value })"
      >
        <component :is="option.icon" class="size-3.5" :stroke-width="1.7" />
      </UiIconButton>
      <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
      <UiIconButton
        v-for="option in verticalOptions"
        :key="option.value"
        :label="t(`spaces.canvas.verticalAlignments.${option.value}`)"
        size="sm"
        :active="verticalAlign === option.value"
        @click="emit('update', { verticalAlign: option.value })"
      >
        <component :is="option.icon" class="size-3.5" :stroke-width="1.7" />
      </UiIconButton>
    </div>
  </UiPopover>
</template>
