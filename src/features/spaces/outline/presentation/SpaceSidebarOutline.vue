<script setup lang="ts">
import { inject, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { workspaceSidebarKey } from "../../../../shared/contracts/workspace-sidebar";
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
  <div class="flex min-h-0 flex-1 flex-col">
    <section
      v-if="store.editSnapshot.structure.length"
      class="flex min-h-0 flex-1 flex-col pt-4"
      :aria-label="t('spaces.sidebar.outline.title')"
    >
      <h2 class="shrink-0 px-4.5 pb-1 text-caption-sm font-medium text-fg-tertiary">
        {{ t("spaces.sidebar.outline.title") }}
      </h2>
      <div class="min-h-0 flex-1">
        <div
          ref="content"
          tabindex="-1"
          class="h-full pr-1 pb-3 pl-2.5 outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus focus-visible:ring-inset"
        >
          <SpaceOutline />
        </div>
      </div>
    </section>
  </div>
</template>
