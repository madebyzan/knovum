<script setup lang="ts">
import { MessageCircle, SlidersHorizontal } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { useContextDock } from "../../canvas/use-context-dock";
import { useSpaceStore } from "../../session/space-store";

const store = useSpaceStore();
const { t } = useI18n();
const dock = useContextDock();
</script>

<template>
  <div
    role="tablist"
    :aria-label="t('spaces.sidebar.rightSidebarViews')"
    class="inline-flex h-7 items-center rounded-lg bg-surface-control p-0.5 text-label select-none"
  >
    <button
      role="tab"
      type="button"
      :aria-selected="store.rightSidebarView === 'ask'"
      class="inline-flex h-full items-center gap-1.5 rounded-md px-2.5 text-caption font-medium transition-colors"
      :class="
        store.rightSidebarView === 'ask'
          ? 'bg-surface-raised text-fg-primary shadow-xs'
          : 'text-fg-secondary hover:text-fg-primary'
      "
      @click="dock.openAsk()"
    >
      <MessageCircle class="size-3.5" :stroke-width="1.6" />
      <span>{{ t("spaces.conversation.title") }}</span>
    </button>
    <button
      role="tab"
      type="button"
      :aria-selected="store.rightSidebarView === 'inspect'"
      class="inline-flex h-full items-center gap-1.5 rounded-md px-2.5 text-caption font-medium transition-colors"
      :class="
        store.rightSidebarView === 'inspect'
          ? 'bg-surface-raised text-fg-primary shadow-xs'
          : 'text-fg-secondary hover:text-fg-primary'
      "
      @click="dock.openInspect()"
    >
      <SlidersHorizontal class="size-3.5" :stroke-width="1.6" />
      <span>{{ t("spaces.inspector.title") }}</span>
    </button>
  </div>
</template>
