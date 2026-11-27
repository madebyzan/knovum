<script setup lang="ts">
import { Check } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { MENU_ITEM_BASE_CLASSES } from "../../../../shared/ui/ui-classes";
import { nodeIcon } from "../../canvas/presentation/node-icon";
import type { SpaceAiReferenceCandidate } from "../tiptap-space-references";

defineProps<{
  candidates: readonly SpaceAiReferenceCandidate[];
  attachedIds: ReadonlySet<string>;
  selectedIndex: number;
}>();

const emit = defineEmits<{
  highlight: [index: number];
  select: [candidate: SpaceAiReferenceCandidate];
}>();

const { t } = useI18n();
const ITEM_CLASS = `${MENU_ITEM_BASE_CLASSES} w-full px-2.5 text-left text-fg-primary hover:bg-surface-control-hover data-[highlighted=true]:bg-surface-control-hover`;
</script>

<template>
  <div data-testid="tiptap-ai-mention-list" class="border-t border-stroke-divider p-1.5 select-none">
    <p class="px-2.5 pt-0.5 pb-1 text-caption-sm font-medium text-fg-tertiary">
      {{ t("spaces.richtext.ai.panel.addReference") }}
    </p>
    <div v-if="candidates.length" class="max-h-52 overflow-y-auto">
      <button
        v-for="(item, index) in candidates"
        :key="item.nodeId"
        type="button"
        :data-highlighted="index === selectedIndex ? 'true' : undefined"
        :class="ITEM_CLASS"
        @mousedown.prevent
        @mouseenter="emit('highlight', index)"
        @click="emit('select', item)"
      >
        <component
          :is="nodeIcon(item.kind)"
          class="size-3.5 shrink-0 text-fg-secondary"
          :stroke-width="1.6"
        />
        <span class="min-w-0 flex-1 truncate">{{ item.title }}</span>
        <Check
          v-if="attachedIds.has(item.nodeId)"
          class="size-3.5 shrink-0 text-fg-accent"
          :stroke-width="1.75"
        />
      </button>
    </div>
    <p v-else class="px-2.5 py-3 text-center text-caption text-fg-tertiary">
      {{ t("spaces.richtext.ai.panel.noReferences") }}
    </p>
  </div>
</template>
