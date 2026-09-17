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

const model = defineModel<string>({ required: true });
defineProps<{ disabled?: boolean }>();
const { t } = useI18n();

const ICON_MAP = {
  Shapes: { icon: Shapes, labelKey: "spaces.knowledge.knowledgeIcons.general" },
  Database: { icon: Database, labelKey: "spaces.knowledge.knowledgeIcons.database" },
  FileText: { icon: FileText, labelKey: "spaces.knowledge.knowledgeIcons.document" },
  Bookmark: { icon: Bookmark, labelKey: "spaces.knowledge.knowledgeIcons.bookmark" },
  Tag: { icon: Tag, labelKey: "spaces.knowledge.knowledgeIcons.tag" },
  Layers: { icon: Layers, labelKey: "spaces.knowledge.knowledgeIcons.layers" },
  Boxes: { icon: Boxes, labelKey: "spaces.knowledge.knowledgeIcons.container" },
  Sparkles: { icon: Sparkles, labelKey: "spaces.knowledge.knowledgeIcons.intelligence" },
  FlaskConical: { icon: FlaskConical, labelKey: "spaces.knowledge.knowledgeIcons.experiment" },
  GitBranch: { icon: GitBranch, labelKey: "spaces.knowledge.knowledgeIcons.decision" },
  CircleHelp: { icon: CircleHelp, labelKey: "spaces.knowledge.knowledgeIcons.question" },
  Lightbulb: { icon: Lightbulb, labelKey: "spaces.knowledge.knowledgeIcons.idea" },
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
      :aria-label="t('spaces.knowledge.knowledgeIcons.select')"
      class="flex h-(--knovum-control-height-md) w-full min-w-32 items-center justify-between gap-2 rounded-control border border-border bg-raised px-2.5 text-label transition-colors duration-100 outline-none hover:border-border-strong focus-visible:border-focus-ring focus-visible:outline-none"
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
        class="z-50 max-h-60 max-w-xs min-w-(--reka-select-trigger-width) overflow-y-auto rounded-panel border border-border material-overlay p-1 shadow-popover"
      >
        <SelectViewport class="space-y-0.5">
          <SelectItem
            v-for="(item, key) in ICON_MAP"
            :key="key"
            :value="key"
            class="relative flex h-(--knovum-compact-item-height) cursor-default items-center rounded-control pr-2 pl-7 text-label outline-none data-highlighted:bg-control-hover"
          >
            <SelectItemIndicator class="absolute left-2"
              ><Check class="size-(--knovum-icon-size-xs)"
            /></SelectItemIndicator>
            <div class="flex min-w-0 flex-1 items-center gap-2">
              <component
                :is="item.icon"
                class="size-(--knovum-icon-size-sm) shrink-0 text-foreground-secondary"
                :stroke-width="1.6"
              />
              <SelectItemText class="min-w-0 flex-1 truncate">{{ t(item.labelKey) }}</SelectItemText>
            </div>
          </SelectItem>
        </SelectViewport>
      </SelectContent>
    </SelectPortal>
  </SelectRoot>
</template>
