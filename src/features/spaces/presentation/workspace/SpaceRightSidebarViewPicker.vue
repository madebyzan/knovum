<script setup lang="ts">
import { ChevronDown, Info, MessageCircle, Search } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import { useContextDock } from "../../canvas/use-context-dock";
import { useSpaceStore } from "../../session/space-store";

const store = useSpaceStore();
const { t } = useI18n();
const dock = useContextDock();
const currentView = computed(() => {
  switch (store.rightSidebarView) {
    case "inspect":
      return { label: t("spaces.inspector.title"), icon: Info };
    case "source":
      return { label: t("spaces.inspector.fields.source"), icon: Search };
    default:
      return { label: t("spaces.conversation.title"), icon: MessageCircle };
  }
});
const views = computed(() => {
  const items = [
    {
      id: "ask",
      label: t("spaces.conversation.title"),
      icon: MessageCircle,
      checked: store.rightSidebarView === "ask",
      onSelect: () => dock.openAsk(),
    },
    {
      id: "inspect",
      label: t("spaces.inspector.title"),
      icon: Info,
      checked: store.rightSidebarView === "inspect",
      onSelect: () => dock.openInspect(),
    },
  ];
  return items;
});
</script>

<template>
  <UiDropdownMenu :items="views" align="start">
    <template #trigger>
      <UiButton
        size="sm"
        variant="ghost"
        class="gap-1 px-1.5"
        :aria-label="`${t('spaces.sidebar.rightSidebarViews')}: ${currentView.label}`"
      >
        <component :is="currentView.icon" class="size-4" :stroke-width="1.55" />
        <span>{{ currentView.label }}</span>
        <ChevronDown class="size-3 text-fg-tertiary" :stroke-width="1.55" />
      </UiButton>
    </template>
  </UiDropdownMenu>
</template>
