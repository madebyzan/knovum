<script setup lang="ts">
import { inject, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { workspaceSidebarKey } from "../../../../shared/contracts/workspace-sidebar";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import { useSpaceStore } from "../../session/space-store";
import SpaceOutline from "./SpaceOutline.vue";

const store = useSpaceStore();
const { t } = useI18n();
const sidebar = inject(workspaceSidebarKey)!;

const content = useTemplateRef<HTMLElement>("content");

watch(
  () => store.outlineFocusRequest,
  async () => {
    sidebar.reveal();
    await nextTick();
    content.value?.focus({ preventScroll: true });
  },
);
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col pt-4" :aria-label="t('spaces.navigation.outline')">
    <h2 class="shrink-0 px-4.5 pb-1 text-micro font-medium text-foreground-tertiary">
      {{ t("spaces.navigation.outline") }}
    </h2>
    <UiScrollArea class="min-h-0 flex-1">
      <div
        ref="content"
        tabindex="-1"
        class="px-2.5 pb-3 outline-none focus-visible:ring-2 focus-visible:ring-focus-ring focus-visible:ring-inset"
      >
        <SpaceOutline />
      </div>
    </UiScrollArea>
  </section>
</template>
