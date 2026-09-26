<script setup lang="ts">
import {
  Bookmark,
  Boxes,
  Check,
  ChevronDown,
  CircleHelp,
  Database,
  FileText,
  FlaskConical,
  GitBranch,
  Layers,
  Lightbulb,
  Shapes,
  Sparkles,
  Tag,
} from "@lucide/vue";
import {
  SelectContent,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectViewport,
} from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import {
  INPUT_BASE_CLASSES,
  POPUP_ANIMATE_CLASSES,
  POPOVER_SURFACE_CLASSES,
} from "../../../../shared/ui/ui-classes";

const model = defineModel<string>({ required: true });
defineProps<{ disabled?: boolean }>();
const { t } = useI18n();

const ICON_MAP = {
  Shapes: { icon: Shapes, labelKey: "spaces.knowledge.icons.general" },
  Database: { icon: Database, labelKey: "spaces.knowledge.icons.database" },
  FileText: { icon: FileText, labelKey: "spaces.knowledge.icons.document" },
  Bookmark: { icon: Bookmark, labelKey: "spaces.knowledge.icons.bookmark" },
  Tag: { icon: Tag, labelKey: "spaces.knowledge.icons.tag" },
  Layers: { icon: Layers, labelKey: "spaces.knowledge.icons.layers" },
  Boxes: { icon: Boxes, labelKey: "spaces.knowledge.icons.container" },
  Sparkles: { icon: Sparkles, labelKey: "spaces.knowledge.icons.intelligence" },
  FlaskConical: { icon: FlaskConical, labelKey: "spaces.knowledge.icons.experiment" },
  GitBranch: { icon: GitBranch, labelKey: "spaces.knowledge.icons.decision" },
  CircleHelp: { icon: CircleHelp, labelKey: "spaces.knowledge.icons.question" },
  Lightbulb: { icon: Lightbulb, labelKey: "spaces.knowledge.icons.idea" },
} as const;

type IconName = keyof typeof ICON_MAP;

const activeEntry = computed(() => {
  const name = (model.value as IconName) in ICON_MAP ? (model.value as IconName) : "Shapes";
  return ICON_MAP[name];
});
</script>

<template>
  <SelectRoot v-model="model" :disabled="disabled">
    <SelectTrigger
      :aria-label="t('spaces.knowledge.icons.select')"
      :class="[
        INPUT_BASE_CLASSES,
        'flex h-(--knovum-control-height-md) w-full min-w-32 items-center justify-between gap-2 px-2.5 text-label data-[state=open]:border-focus-ring',
      ]"
    >
      <div class="flex min-w-0 flex-1 items-center gap-2 text-left">
        <component
          :is="activeEntry.icon"
          class="size-(--knovum-icon-size-sm) shrink-0 text-foreground-secondary"
          :stroke-width="1.6"
        />
        <span class="truncate">{{ t(activeEntry.labelKey) }}</span>
      </div>
      <ChevronDown class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary" />
    </SelectTrigger>
    <SelectPortal>
      <SelectContent
        position="popper"
        :side-offset="6"
        :collision-padding="8"
        avoid-collisions
        :class="[
          POPUP_ANIMATE_CLASSES,
          POPOVER_SURFACE_CLASSES,
          'z-50 max-h-60 max-w-xs min-w-(--reka-select-trigger-width) overflow-y-auto p-1 duration-100 ease-standard',
        ]"
      >
        <SelectViewport class="space-y-0.5">
          <SelectItem
            v-for="(iconOption, key) in ICON_MAP"
            :key="key"
            :value="key"
            class="relative flex h-(--knovum-compact-item-height) cursor-default items-center rounded-control pr-2 pl-7 text-label outline-none data-highlighted:bg-control-hover"
          >
            <SelectItemIndicator class="absolute left-2"
              ><Check class="size-(--knovum-icon-size-xs)"
            /></SelectItemIndicator>
            <div class="flex min-w-0 flex-1 items-center gap-2">
              <component
                :is="iconOption.icon"
                class="size-(--knovum-icon-size-sm) shrink-0 text-foreground-secondary"
                :stroke-width="1.6"
              />
              <SelectItemText class="min-w-0 flex-1 truncate">{{ t(iconOption.labelKey) }}</SelectItemText>
            </div>
          </SelectItem>
        </SelectViewport>
      </SelectContent>
    </SelectPortal>
  </SelectRoot>
</template>
