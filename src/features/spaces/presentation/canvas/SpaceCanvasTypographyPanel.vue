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
import type { CanvasNodeAppearance } from "../../canvas/contract/canvas-document";
import { canvasBaseFontSize, effectiveCanvasFontSize } from "../../canvas/document/canvas-typography";
import { DEFAULT_CANVAS_TEXT_FONT_SIZE } from "../../canvas/node-size";
import { CANVAS_FONT_SIZES } from "./canvas-typography-options";

const props = defineProps<{ kind: "note" | "text"; appearance?: CanvasNodeAppearance }>();
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
  <UiPopover side="top" align="center" :side-offset="6" class="p-0.5">
    <template #trigger>
      <button
        type="button"
        class="grid h-(--knovum-control-height-sm) min-w-8 place-items-center rounded-control px-1.5 text-body font-medium text-foreground-secondary transition-colors duration-100 outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring"
        :aria-label="t('spaces.canvas.typography')"
      >
        Aa
      </button>
    </template>

    <div class="flex items-center gap-0.5">
      <div class="flex items-center gap-0.5">
        <UiIconButton
          :label="t('editor.bold')"
          size="sm"
          :active="appearance?.fontWeight === 'bold'"
          @click="emit('update', { fontWeight: appearance?.fontWeight === 'bold' ? 'regular' : 'bold' })"
        >
          <Bold class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
        </UiIconButton>
        <UiIconButton
          :label="t('editor.italic')"
          size="sm"
          :active="appearance?.fontStyle === 'italic'"
          @click="emit('update', { fontStyle: appearance?.fontStyle === 'italic' ? 'normal' : 'italic' })"
        >
          <Italic class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
        </UiIconButton>
        <UiIconButton
          :label="t('editor.underline')"
          size="sm"
          :active="appearance?.textDecoration === 'underline'"
          @click="
            emit('update', {
              textDecoration: appearance?.textDecoration === 'underline' ? 'none' : 'underline',
            })
          "
        >
          <Underline class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
        </UiIconButton>
      </div>
      <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
      <UiDropdownMenu side="top" align="center" :side-offset="5" :modal="false" class="min-w-20">
        <template #trigger>
          <button
            type="button"
            class="h-(--knovum-control-height-sm) min-w-10 rounded-control px-1.5 text-caption font-medium text-foreground-secondary tabular-nums transition-colors duration-100 outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring"
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
          <Check
            class="size-(--knovum-icon-size-xs)"
            :class="fontSize === size ? 'opacity-100' : 'opacity-0'"
          />
          <span class="tabular-nums">{{ size }} pt</span>
        </UiDropdownMenuItem>
      </UiDropdownMenu>
      <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
      <UiIconButton
        v-for="item in horizontalOptions"
        :key="item.value"
        :label="t(`spaces.canvas.textAlignments.${item.value}`)"
        size="sm"
        :active="textAlign === item.value"
        @click="emit('update', { textAlign: item.value })"
      >
        <component :is="item.icon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
      </UiIconButton>
      <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
      <UiIconButton
        v-for="item in verticalOptions"
        :key="item.value"
        :label="t(`spaces.canvas.verticalAlignments.${item.value}`)"
        size="sm"
        :active="verticalAlign === item.value"
        @click="emit('update', { verticalAlign: item.value })"
      >
        <component :is="item.icon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
      </UiIconButton>
    </div>
  </UiPopover>
</template>
