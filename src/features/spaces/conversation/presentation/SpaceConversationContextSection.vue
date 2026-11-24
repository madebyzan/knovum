<script setup lang="ts">
import { Check, Plus } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { nodeIcon } from "../../canvas/presentation/node-icon";
import type { SpaceQuickContextItem } from "../conversation-empty-guide";

defineProps<{
  items: readonly SpaceQuickContextItem[];
  totalCount: number;
}>();

const emit = defineEmits<{ toggle: [item: SpaceQuickContextItem] }>();
const { t } = useI18n();
</script>

<template>
  <section v-if="items.length > 0" class="border-t border-stroke-divider/70 py-3">
    <div class="mb-1 flex items-center justify-between gap-2">
      <h3 class="text-caption font-medium text-fg-tertiary">
        {{ t("spaces.conversation.empty.contextHeading") }}
      </h3>
      <span class="text-caption-sm text-fg-tertiary/80 tabular-nums">{{ totalCount }}</span>
    </div>
    <div class="space-y-0.5">
      <button
        v-for="item in items"
        :key="item.nodeId"
        type="button"
        class="group -mx-1.5 flex h-interaction w-[calc(100%+0.75rem)] items-center gap-2 rounded-md px-1.5 text-left text-label transition-colors duration-100 hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :class="item.attached ? 'text-fg-primary' : 'text-fg-secondary hover:text-fg-primary'"
        :title="item.title"
        :aria-pressed="item.attached"
        @click="emit('toggle', item)"
      >
        <span
          class="grid size-4.5 shrink-0 place-items-center overflow-hidden rounded-[4px] bg-surface-control text-fg-tertiary ring-1 ring-stroke-divider/60 ring-inset"
          aria-hidden="true"
        >
          <img
            v-if="item.previewSource"
            :src="item.previewSource"
            alt=""
            class="size-full object-cover"
            loading="lazy"
            decoding="async"
          />
          <component :is="nodeIcon(item.nodeKind)" v-else class="size-3" :stroke-width="1.55" />
        </span>
        <span class="min-w-0 flex-1 truncate">{{ item.title }}</span>
        <Check v-if="item.attached" class="size-3.5 shrink-0 text-fg-accent" :stroke-width="1.85" />
        <Plus
          v-else
          class="size-3.5 shrink-0 text-fg-tertiary opacity-0 transition-opacity duration-100 group-hover:opacity-100 group-focus-visible:opacity-100"
          :stroke-width="1.6"
        />
      </button>
    </div>
  </section>
</template>
