<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useCanvasSelectionProjection } from "../../canvas/editor-session";
import { useContextDock } from "../../canvas/use-context-dock";
import { knowledgeLocationLabel } from "../../knowledge/model/location";
import { useSpaceStore } from "../../session/space-store";
import SpaceKnowledgeAssistant from "../knowledge/SpaceKnowledgeAssistant.vue";
import SpaceContextHeader from "./SpaceContextHeader.vue";
import SpaceInspectorDetails from "./SpaceInspectorDetails.vue";
import SpaceSourcePeek from "./SpaceSourcePeek.vue";

const store = useSpaceStore();
const { inspectorMode, sourcePeek } = storeToRefs(store);
const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
const { t } = useI18n();
const dock = useContextDock();

watch(
  () => store.workspaceMode,
  (mode) => {
    if (mode === "library") dock.closeDock();
  },
);

watch(
  () => store.activeSpaceId,
  () => {
    if (store.sourcePeek) store.closeSource();
  },
);

const title = computed(() => {
  if (inspectorMode.value === "ask") return t("spaces.assistant.title");
  if (inspectorMode.value === "source") return sourcePeek.value?.nodeTitle ?? t("spaces.inspector.source");
  if (selectedNodes.value.length === 1) return t("spaces.navigation.details");
  return selectedNodes.value.length
    ? t("spaces.inspector.itemsSelected", { count: selectedNodes.value.length })
    : t("spaces.knowledge.knowledgeHealth");
});
const subtitle = computed(() =>
  inspectorMode.value === "source" && sourcePeek.value
    ? knowledgeLocationLabel(sourcePeek.value.locator)
    : undefined,
);
const panelClasses = computed(() =>
  inspectorMode.value === "inspect"
    ? "mt-2 max-h-[calc(100%-0.5rem)] rounded-workspace border border-window-hairline"
    : "h-full rounded-r-workspace border-y border-r border-l border-window-hairline",
);
</script>

<template>
  <aside
    :data-inspector-mode="inspectorMode"
    class="pointer-events-auto flex w-full min-w-0 overflow-hidden bg-panel select-none"
    :class="panelClasses"
  >
    <section class="flex max-h-full min-w-0 flex-1 flex-col overflow-hidden">
      <SpaceContextHeader
        :title="title"
        v-bind="subtitle === undefined ? {} : { subtitle }"
        :back="inspectorMode === 'source'"
        @back="store.closeSource()"
        @close="dock.closeDock()"
      />
      <div class="min-h-0 flex-1 overflow-hidden select-text">
        <SpaceInspectorDetails v-if="inspectorMode === 'inspect'" class="h-full" />
        <SpaceKnowledgeAssistant v-else-if="inspectorMode === 'ask'" class="h-full" />
        <SpaceSourcePeek v-if="inspectorMode === 'source' && sourcePeek" class="h-full" />
      </div>
    </section>
  </aside>
</template>
