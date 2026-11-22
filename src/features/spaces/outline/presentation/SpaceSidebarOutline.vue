<script setup lang="ts">
import { ShieldCheck } from "@lucide/vue";
import { inject, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { workspaceSidebarKey } from "../../../../shared/contracts/workspace-sidebar";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
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
      <div class="flex shrink-0 items-center justify-between pr-2.5 pb-1 pl-4.5">
        <h2 class="text-caption-sm font-medium text-fg-tertiary">
          {{ t("spaces.sidebar.outline.title") }}
        </h2>
        <UiIconButton
          :label="t('spaces.knowledge.health.title')"
          size="sm"
          tooltip
          class="size-5 text-fg-tertiary hover:text-fg-primary"
          @click="store.openKnowledgeHealth()"
        >
          <ShieldCheck class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
      </div>
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
