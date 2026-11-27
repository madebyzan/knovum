<script setup lang="ts">
import { ShieldAlert, ShieldCheck } from "@lucide/vue";
import { computed, inject, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { workspaceSidebarKey } from "../../../../shared/contracts/workspace-sidebar";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useSpaceKnowledgeHealthQuery } from "../../knowledge/queries/use-space-knowledge-health-query";
import { useSpaceStore } from "../../session/space-store";
import SpaceOutline from "./SpaceOutline.vue";

const store = useSpaceStore();
const { t } = useI18n();
const sidebar = inject(workspaceSidebarKey)!;
const healthQuery = useSpaceKnowledgeHealthQuery(() => store.activeSpaceId);
const healthIssues = computed(() => healthQuery.data.value ?? []);
const hasCriticalHealth = computed(() => healthIssues.value.some((issue) => issue.severity === "critical"));

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
          :label="
            healthIssues.length
              ? `${t('spaces.knowledge.health.title')} (${healthIssues.length})`
              : t('spaces.knowledge.health.title')
          "
          size="sm"
          tooltip
          :class="[
            healthIssues.length ? 'h-5 w-auto min-w-5 gap-1 px-1' : 'size-5',
            healthIssues.length
              ? hasCriticalHealth
                ? 'text-fg-danger hover:bg-surface-danger-subtle'
                : 'text-fg-warning hover:bg-surface-warning-subtle'
              : 'text-fg-tertiary hover:text-fg-primary',
          ]"
          @click="store.openKnowledgeHealth()"
        >
          <ShieldAlert v-if="healthIssues.length" class="size-3.5" :stroke-width="1.65" />
          <ShieldCheck v-else class="size-3.5" :stroke-width="1.6" />
          <span v-if="healthIssues.length" class="text-caption-sm font-medium tabular-nums">
            {{ healthIssues.length }}
          </span>
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
